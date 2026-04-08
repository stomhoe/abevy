use ::game_common::game_common_components::*;
use ::tilemap_shared::*;
#[allow(unused_imports)]
use bevy::prelude::*;

use common::PORTAL_INIT;
use common::common_components::HashId;

use crate::{
    run_suitable_pos_search_logic, terrain::{
        operation_list::operation_list_resources::OperationListEntityMap,
        terrprobe::{terrprobe_components::*, terrprobe_resources::*},
        terrprobe::{terrprobe_messages::TerrProbeJob, terrprobe_systems::SearchParams},
    }, tile::{tile_components::*, tile_resources::*}, tilemap_resources::MassCollectedTiles
};

#[allow(unused_parens)]
pub fn map_portal_tiles(
    mut cmd: Commands,
    mut portals_templ_query: Query<(Entity, &TileStrId, &mut PortalSeri),
        (With<Templ>, common::AnyDisabling, Changed<PortalSeri>),
    >,
    tiles_map: Res<TileEntityMap>,
) {
    info!("Mapping portal tiles");
    portals_templ_query.iter_mut().for_each(|(ent, str_id, portal_seri)| {
        let Ok(tile_ent) = tiles_map.0.get_cloned(&portal_seri.oe_tile) else {
            error!(
                target: PORTAL_INIT,
                "Portal tile {} to '{}' references unknown oe_tile '{}'",
                str_id,
                portal_seri.dest_dimension,
                portal_seri.oe_tile
            );
            return;
        };
        info!(
            target: PORTAL_INIT,
            "Mapping portal tile '{}' to destination dimension '{}'",
            str_id,
            portal_seri.dest_dimension
        );
        let mut sampled_offsets = Vec::with_capacity(portal_seri.offset_pos_destinations.len().max(1));
        for (weight, (x, y)) in &portal_seri.offset_pos_destinations {
            sampled_offsets.push((GlobalTilePos::new(*x as i32, *y as i32), *weight));
        }
        if sampled_offsets.is_empty() {
            sampled_offsets.push((GlobalTilePos::default(), 1.0));
        }

        cmd.entity(ent).insert(PortalRecipe {
            dest_dimension: Entity::PLACEHOLDER,
            oe_portal_tile: tile_ent,
            terrprobe_ent: Entity::PLACEHOLDER,
            one_way: portal_seri.one_way,
            sampler: GlobalTilePosWeightedSampler::new(&sampled_offsets),
        });
    });
}

#[allow(unused_parens)]
pub fn validate_portal_recipes(
    mut cmd: Commands,
    mut portal_recipes: Query<(Entity, &mut PortalRecipe, Option<&PortalSeri>), With<Templ>>,
    dimension_query: Query<&DimensionRootOplist>,
    oplist_map: Res<OperationListEntityMap>,
    oplist_hash_query: Query<(Entity, &HashId), With<crate::terrain::operation_list::operation_list_components::OperationList>>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
) {
    for (templ_portal, mut recipe, portal_seri_opt) in portal_recipes.iter_mut() {
        if recipe.dest_dimension == Entity::PLACEHOLDER {
            continue;
        }
        let Ok(root_oplist) = dimension_query.get(recipe.dest_dimension) else {
            error!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} references a Dimension that doesn't exist or has no DimensionRootOplist.", recipe.dest_dimension);
            continue;
        };
        let Some(root_oplist_ent) = oplist_map
            .0
            .get_cloned(root_oplist.0)
            .ok()
            .or_else(|| {
                oplist_hash_query
                    .iter()
                    .find_map(|(oplist_ent, &oplist_hash)| (oplist_hash == root_oplist.0).then_some(oplist_ent))
            })
        else {
            error!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} references missing root oplist hash {:?}", recipe.dest_dimension, root_oplist.0);
            continue;
        };
        if recipe.terrprobe_ent == Entity::PLACEHOLDER
            && let Some(portal_seri) = portal_seri_opt
            && let Ok(ent) = terrprobe_entity_map.0.get_cloned(&portal_seri.oe_terrprobe)
        {
            recipe.terrprobe_ent = ent;
        }

        if recipe.terrprobe_ent != Entity::PLACEHOLDER {
            debug!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} is valid with root oplist {:?} (entity {:?}) and terrprobe {:?}.", recipe.dest_dimension, root_oplist, root_oplist_ent, recipe.terrprobe_ent);
            cmd.entity(templ_portal).try_insert(AwaitingStartSearch);
        } else {
            let terrprobe_id = portal_seri_opt.map(|p| p.oe_terrprobe.as_str()).unwrap_or("<missing PortalSeri>");
            error_once!(target: PORTAL_INIT, "PortalRecipe references missing terrprobe '{}' for dest_dimension entity {:?}", terrprobe_id, recipe.dest_dimension);
            cmd.entity(templ_portal).try_remove::<AwaitingStartSearch>();
        }
    }
}
#[allow(unused_parens)]
pub fn start_portal_search(
    mut cmd: Commands,
    portals: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            &TemplEntiRef,
            Option<&PortalTo>,
            Option<&SearchingForSuitablePos>,
        ),
        (Without<Templ>, With<Tile>, With<AwaitingStartSearch>),
    >,
    templ_query: Query<(&TileStrId, Option<&PortalRecipe>), (With<Templ>,)>,
    terrprobe_query: Query<&TerrProbeTempl>,
    dimension_hash_query: Query<&HashId, With<Dimension>>,
    mut search_params: SearchParams,
) {
    let make_search_request = |_cmd: &mut Commands,
                               portal_ent: Entity,
                               global_pos: GlobalTilePos,
                               templ_ref: TemplEntiRef|
     -> Option<TerrProbeJob> {
        let Ok((str_id, portal_recipe_opt)) = templ_query.get(templ_ref.0) else {
            error!(target: PORTAL_INIT, "Portal tile entity {:?} references an EntityZero {:?} which no longer exists.", portal_ent, templ_ref.0);
            return None;
        };
        let Some(portal_recipe) = portal_recipe_opt else {
            error!(target: PORTAL_INIT, "Portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", portal_ent, templ_ref.0);
            return None;
        };
        let probe_template_ent = portal_recipe.terrprobe_ent;
        if probe_template_ent == Entity::PLACEHOLDER {
            error!(target: PORTAL_INIT, "Portal tile '{}' has no terrprobe_ent resolved", str_id);
            return None;
        }
        let Ok(probe_template) = terrprobe_query.get(probe_template_ent) else {
            error!(target: PORTAL_INIT, "TerrainProbe template entity {:?} missing TerrProbeTempl", probe_template_ent);
            return None;
        };
        let Ok(&dest_dim_hash) = dimension_hash_query.get(portal_recipe.dest_dimension) else {
            error!(target: PORTAL_INIT, "Portal tile '{}' destination dimension entity {:?} is missing HashId", str_id, portal_recipe.dest_dimension);
            return None;
        };
        let probe = probe_template.to_probe(probe_template_ent, DimensionRef(dest_dim_hash), global_pos);
        Some(probe)
    };

    for (portal_ent, dim_ref, global_pos, templ_ref, portal_to, searching_for) in portals.iter() {
        if portal_to.is_some() {
            cmd.entity(portal_ent).try_remove::<AwaitingStartSearch>();
            continue;
        }
        if searching_for.is_some() {
            continue;
        }
        cmd.entity(portal_ent).try_remove::<AwaitingStartSearch>();

        let Some(mut probe) = make_search_request(&mut cmd, portal_ent, *global_pos, *templ_ref) else {
            continue;
        };
        if probe.requester == Entity::PLACEHOLDER {
            probe.requester = portal_ent;
        }
        let requester = probe.requester;

        info!(
            target: PORTAL_INIT,
            "Starting suitable-pos search for portal tile entity {:?} at position {:?}",
            portal_ent,
            global_pos
        );

        cmd.entity(portal_ent).try_insert(SearchingForSuitablePos {
            requester,
            collect_all_successes: probe.collect_all_successes,
        });
        search_params
            .requester_collect_all
            .insert(requester, probe.collect_all_successes);
        search_params
            .requester_had_success
            .insert(requester, false);
        search_params
            .min_result_distance_by_requester
            .insert(requester, probe.min_result_distance as u64);
        search_params.pos_searches_msgs_to_write.push(probe);
        search_params
            .pending_by_requester
            .entry(requester)
            .or_default()
            .push((portal_ent, *global_pos, *dim_ref, *templ_ref));
    }
    search_params.write_pos_searches();
}

#[allow(unused_parens)]
pub fn resolve_portal_search_results(
    mut cmd: Commands,
    portals: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            &TemplEntiRef,
            Option<&SearchingForSuitablePos>,
        ),
        (Without<Templ>, With<Tile>),
    >,
    templ_query: Query<(&TileStrId, Option<&PortalRecipe>), (With<Templ>,)>,
    dimension_hash_query: Query<&HashId, With<Dimension>>,
    mut mass_collected: ResMut<MassCollectedTiles>,
    mut register_pos: ResMut<ImportantRegisteredPositions>,
    mut search_params: SearchParams,
) {
    let mut handle_success_event = |cmd: &mut Commands,
                                    portal_ent: Entity,
                                    my_pos: GlobalTilePos,
                                    dim_ref: DimensionRef,
                                    templ_ref: TemplEntiRef,
                                    found_pos: GlobalTilePos,
                                    _sampled_val: f32,
                                    _is_last: bool|
     -> bool {
        let Ok((str_id, portal_recipe_opt)) = templ_query.get(templ_ref.0) else {
            error!(target: PORTAL_INIT, "SuitablePosFound but portal tile entity {:?} references an EntityZero {:?} which no longer exists.", portal_ent, templ_ref.0);
            return false;
        };
        let Some(portal_recipe) = portal_recipe_opt else {
            error!(target: PORTAL_INIT, "SuitablePosFound but portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", portal_ent, templ_ref.0);
            return false;
        };
        let portal_recipe = portal_recipe.clone();

        info!(target: PORTAL_INIT, "Found suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}, found pos: {:?}", str_id, portal_ent, dim_ref.0, my_pos, portal_recipe.dest_dimension, found_pos);

        let Ok(&dest_dim_hash) = dimension_hash_query.get(portal_recipe.dest_dimension) else {
            error!(target: PORTAL_INIT, "Failed to resolve destination dimension hash for portal tile {}", str_id);
            return false;
        };
        let oe_dim_ref = DimensionRef(dest_dim_hash);

        let oe_portal_tileref = TemplEntiRef(portal_recipe.oe_portal_tile);
        debug!(target: PORTAL_INIT, "OE Portal TileRef: {:?}", oe_portal_tileref);

        let oe_portal = mass_collected.clonespawn_and_push_tile(
            cmd,
            oe_portal_tileref,
            found_pos,
            oe_dim_ref,
        );
        register_pos.exempt_entity_from_mindist_checks(oe_portal);

        cmd.entity(portal_ent)
            .try_insert(PortalTo::new(oe_portal, portal_recipe.sampler.clone()))
            .try_remove::<(SearchingForSuitablePos, AwaitingStartSearch)>();

        if !portal_recipe.one_way {
            cmd.entity(oe_portal).try_insert(PortalTo::new(portal_ent, portal_recipe.sampler.clone()));
        }

        debug!(target: PORTAL_INIT, "Instantiated oe-portal '{}' at position {:?} in dimension {:?}", oe_portal, found_pos, portal_recipe.dest_dimension);
        true
    };

    let handle_pending_failure = |portal_ent: Entity,
                                  global_pos: GlobalTilePos,
                                  dim_ref: DimensionRef,
                                  tile_ref: TemplEntiRef,
                                  failed_filter_ent: Entity| {
        let Ok((str_id, portal_template)) = templ_query.get(tile_ref.0) else {
            error!(target: PORTAL_INIT, "SearchFailed for studied_op_ent {:?} portal tile entity {:?} references an EntityZero {:?} which no longer exists or has no StrId.", failed_filter_ent, portal_ent, tile_ref.0);
            return;
        };
        let Some(portal_template) = portal_template else {
            error!(target: PORTAL_INIT, "SearchFailed for studied_op_ent {:?} portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", failed_filter_ent, portal_ent, tile_ref.0);
            return;
        };

        error!(target: PORTAL_INIT, "Failed to find suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}", str_id, portal_ent, dim_ref.0, global_pos, portal_template.dest_dimension);
    };

    run_suitable_pos_search_logic!(
        target: PORTAL_INIT,
        searched_entity_label: "portal tile",
        cmd: cmd,
        searching_entities: portals,
        search_params: search_params,
        handle_success_event: handle_success_event,
        handle_pending_failure: handle_pending_failure,
    );
}
