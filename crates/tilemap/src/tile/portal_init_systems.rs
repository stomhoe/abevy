use ::game_common::game_common_components::*;
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use sprite_shared::AcZ;
use ::tilemap_shared::*;
#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::{
    platform::collections::HashSet,
};
use common::PORTAL_INIT;

use crate::{
    run_suitable_pos_search_logic, terrain::{
        terrprobe::{terrprobe_components::*, terrprobe_resources::*},
        terrprobe::{terrprobe_messages::TerrProbeJob, terrprobe_systems::SearchParams},
    }, tile::{tile_components::*, tile_resources::*}, tilemap_resources::MassCollectedTiles
};

#[allow(unused_parens)]
pub fn map_portal_tiles(
    mut cmd: Commands,
    mut portals_ezero_query: Query<(Entity, &TileStrId, &mut PortalSeri),
        (With<EntityZero>, common::AnyDisabling, Changed<PortalSeri>),
    >,
    tiles_map: Res<TileEntityMap>,
) {
    info!("Mapping portal tiles");
    portals_ezero_query.iter_mut().for_each(|(ent, str_id, portal_seri)| {
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
    mut portal_recipes: Query<(Entity, &mut PortalRecipe, Option<&PortalSeri>)>,
    dimension_query: Query<Option<&DimensionRootOplist>>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
) {
    for (ezero_portal, mut recipe, portal_seri_opt) in portal_recipes.iter_mut() {
        if recipe.dest_dimension == Entity::PLACEHOLDER {
            continue;
        }
        let Ok(root_oplist) = dimension_query.get(recipe.dest_dimension) else {
            error!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} references a Dimension that doesn't exist.", recipe.dest_dimension);
            continue;
        };
        let Some(root_oplist) = root_oplist else {
            error!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} references a Dimension that has no DimensionRootOplist.", recipe.dest_dimension);
            continue;
        };
        if recipe.terrprobe_ent == Entity::PLACEHOLDER
            && let Some(portal_seri) = portal_seri_opt
            && let Ok(ent) = terrprobe_entity_map.0.get_cloned(&portal_seri.oe_terrprobe)
        {
            recipe.terrprobe_ent = ent;
        }

        if recipe.terrprobe_ent != Entity::PLACEHOLDER {
            debug!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} is valid with root oplist {:?} and terrprobe {:?}.", recipe.dest_dimension, root_oplist, recipe.terrprobe_ent);
            cmd.entity(ezero_portal).try_insert(AwaitingStartSearch);
        } else {
            let terrprobe_id = portal_seri_opt.map(|p| p.oe_terrprobe.as_str()).unwrap_or("<missing PortalSeri>");
            error!(target: PORTAL_INIT, "PortalRecipe references missing terrprobe '{}' for dest_dimension entity {:?}", terrprobe_id, recipe.dest_dimension);
            cmd.entity(ezero_portal).try_remove::<AwaitingStartSearch>();
        }
    }
}
#[allow(unused_parens)]
pub fn instantiate_portal(
    mut cmd: Commands,
    portals: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            &EntityZeroRef,
            Has<AwaitingStartSearch>,
            Option<&SearchingForSuitablePos>,
        ),
        (Without<EntityZero>, With<Tile>),
    >,
    ezero_query: Query<(&TileStrId, Option<&PortalRecipe>), (With<EntityZero>,)>,
    mut mass_collected: ResMut<MassCollectedTiles>,
    mut register_pos: ResMut<ImportantRegisteredPositions>,
    terrprobe_query: Query<&TerrProbeTempl>,
    mut search_params: SearchParams,
) {
    let make_search_request = |_cmd: &mut Commands,
                               portal_ent: Entity,
                               global_pos: GlobalTilePos,
                               ezero_ref: EntityZeroRef|
     -> Option<TerrProbeJob> {
        let Ok((str_id, portal_recipe_opt)) = ezero_query.get(ezero_ref.0) else {
            error!(target: PORTAL_INIT, "Portal tile entity {:?} references an EntityZero {:?} which no longer exists.", portal_ent, ezero_ref.0);
            return None;
        };
        let Some(portal_recipe) = portal_recipe_opt else {
            error!(target: PORTAL_INIT, "Portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", portal_ent, ezero_ref.0);
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
        let probe = probe_template.to_probe(probe_template_ent, DimensionRef(portal_recipe.dest_dimension), global_pos);
        Some(probe)
    };

    let mut handle_success_event = |cmd: &mut Commands,
                                    portal_ent: Entity,
                                    my_pos: GlobalTilePos,
                                    dim_ref: DimensionRef,
                                    ezero_ref: EntityZeroRef,
                                    found_pos: GlobalTilePos,
                                    _sampled_val: f32,
                                    _is_last: bool|
     -> bool {
        let Ok((str_id, portal_recipe_opt)) = ezero_query.get(ezero_ref.0) else {
            error!(target: PORTAL_INIT, "SuitablePosFound but portal tile entity {:?} references an EntityZero {:?} which no longer exists.", portal_ent, ezero_ref.0);
            return false;
        };
        let Some(portal_recipe) = portal_recipe_opt else {
            error!(target: PORTAL_INIT, "SuitablePosFound but portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", portal_ent, ezero_ref.0);
            return false;
        };
        let portal_recipe = portal_recipe.clone();

        info!(target: PORTAL_INIT, "Found suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}, found pos: {:?}", str_id, portal_ent, dim_ref.0, my_pos, portal_recipe.dest_dimension, found_pos);

        let oe_dim_ref = DimensionRef(portal_recipe.dest_dimension);

        let oe_portal_tileref = EntityZeroRef(portal_recipe.oe_portal_tile);
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

        cmd.entity(oe_portal)
            .try_remove::<(AwaitingStartSearch)>()
            .try_insert(DeleteOtherTiles {
                spared_z: HashSet::from_iter(vec![AcZ::new(-900.0)]),
                extra_radius: 2,
                ..Default::default()
            });

        if !portal_recipe.one_way {
            cmd.entity(oe_portal).try_insert(PortalTo::new(portal_ent, portal_recipe.sampler.clone()));
        }

        debug!(target: PORTAL_INIT, "Instantiated oe-portal '{}' at position {:?} in dimension {:?}", oe_portal, found_pos, portal_recipe.dest_dimension);
        true
    };

    let handle_pending_failure = |portal_ent: Entity,
                                  global_pos: GlobalTilePos,
                                  dim_ref: DimensionRef,
                                  tile_ref: EntityZeroRef,
                                  failed_filter_ent: Entity| {
        let Ok((str_id, portal_template)) = ezero_query.get(tile_ref.0) else {
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
        make_search_request: make_search_request,
        handle_success_event: handle_success_event,
        handle_pending_failure: handle_pending_failure,
    );
}
