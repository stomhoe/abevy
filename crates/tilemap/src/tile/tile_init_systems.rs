
use ::game_common::{game_common_components::*, };
use ::sprite_shared::{sprite_scale_offset::Offset2D, *};
use ::tilemap_shared::*;
#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::HashSet,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use color_sampler::{ColorSamplerEntityMap, ColorSamplerRef,};
use common::{PORTAL_INIT, TILE_INIT, common_components::*, common_tag_components::TagSet};
use sprite::sprite_components::SpriteConfig;
use sprite_animation_shared::AcAnimationProgresses;

use crate::{
    terrain_gen::{terrgen_messages::*, terrgen_operaton_list_components::OperationList, terrgen_search::{AwaitingStartSearch, SearchParams}},
    tile::{
        tile_components::*,
        tile_resources::*,
        tile_shader::{tile_shader_components::*, tile_shader_resources::*},
    },
    tilemap_resources::*,
};
use std::mem::take;

#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands,
    seris_handles: Res<TileSerisHandles>,
    mut assets: ResMut<Assets<TileSeri>>,
    shader_map: Res<TileShaderEntityMap>,
    tiling_map: Res<TileEntityMap>,
    color_map: Res<ColorSamplerEntityMap>,
    egui_tiles_holder_query: Query<Entity, With<EguiTilesHolder>>,
) {
    if !tiling_map.0.0.is_empty() {
        return;
    }
    let holder = if let Ok(first_holder) = egui_tiles_holder_query.single() {
        first_holder
    } else {
        cmd.spawn((EguiTilesHolder,)).id()
    };

    let egui_portal_holder = cmd.spawn((PortalsZeroEguiHolder, ChildOf(holder))).id();

    let mut res_tile_cats = TileEntsWithinTag::default();

    seris_handles.handles.iter().for_each(|handle| {
        let Some(seri) = assets.get_mut(handle) else { return; };

        let str_id = match TileStrId::new_with_result(seri.id.clone(), Tile::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create TileStrId for tile '{}': {}", seri.id, err);
                return;
            }
        };
        let my_z = AcZ(seri.z);
        let tile_enti = cmd.spawn((
            Tile, Replicated, str_id.clone(), //PROBLEMA: EL DISABLED HACE Q EL DESPAWNONEXIT NO FUNCIONE
            Prefix::trunc("Tile"),
            my_z.clone(),
            EntityZero,
            AddHashIdFromStrId,
            ChildOf(holder),
            AssetScoped,
            SizeInTiles::new(seri.size_in_tiles),
            //SparedFromHotReloading,
        )).id();

        if let Some(tags) = &seri.tags {
            let mut tag_set = TagSet::default();
            for tag_string in tags {
                let tag_str = tag_string.trim();
                if tag_str.is_empty() { continue; }
                let tag = Tag::trunc(tag_str);
                tag_set.insert(tag.clone());
                res_tile_cats.0.entry(tag).or_default().insert(tile_enti);
            }
            cmd.entity(tile_enti).insert(tag_set);
        }

        let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
        let color = Color::srgba_u8(r, g, b, a);

        if ! seri.name.is_empty() {
            cmd.entity(tile_enti).insert(DisplayName(seri.name.clone()));
        }
        if seri.portal.is_some() {
            cmd.entity(tile_enti).insert(Persisted);
        }
        if seri.img_paths.is_empty() {
            warn!("Tile '{}' has no img_paths entries", str_id);
        }

        if let Some(ref mut adj_retex_config) = seri.adj_retex {
            cmd.entity(tile_enti).insert(AdjRetexConfig::new(take(adj_retex_config)));
        }

        if let Some(ref color_map_str) = seri.color_map {
            if !color_map_str.is_empty() {
                match color_map.0.get_cloned(color_map_str) {
                    Ok(color_sampler_ent) => {
                        cmd.entity(tile_enti).insert(ColorSamplerRef(color_sampler_ent));
                    }
                    Err(_err) => {
                        error!("Tile '{}': Weighted color sampler with id '{}' not found in ColorSamplerEntityMap", str_id, color_map_str);
                    }
                }
            }
        }
        if seri.randflipx == Some(true) {
            cmd.entity(tile_enti).insert(FlipHorizontallyBasedOnHash);
        }
        if let Some(portal) = &mut seri.portal {
            cmd.entity(tile_enti).insert((take(portal), ChildOf(egui_portal_holder)));
        }

        if let Some(ws) = seri.walk_speed {
            cmd.entity(tile_enti).insert(WalkSpeedMultIfOnTop(ws));
        } else{
            cmd.entity(tile_enti).insert(WalkSpeedMultIfOnTop(1.0));
        }

        if seri.blocks_projectiles == Some(true) {
            cmd.entity(tile_enti).insert(BlocksProjectiles);
        }



        if seri.sprite != Some(true) {
            cmd.entity(tile_enti).insert(TileImagePaths(take(&mut seri.img_paths)));

            if let Some(shader_str) = &seri.shader {
                if shader_str.len() > 2 {
                    let Ok(shader_ent) = shader_map.0.get_cloned(shader_str) else {
                        error!("Tile '{}' references shader {} not found in TileShaderEntityMap", str_id, shader_str);
                        return;
                    };
                    cmd.entity(tile_enti).insert(TileShaderRef(shader_ent));
                } else if shader_str.len() > 0 {
                    warn!("Tile {} shader {} is too short for a shader", str_id, shader_str);
                }
            }
            if let Some(y_sort_origin) = seri.y_sort {
                cmd.entity(tile_enti).insert(YSortOrigin(seri.offset.unwrap_or_default().1 + y_sort_origin - 10.0));
            }

            cmd.entity(tile_enti).insert_if_new((TileColor::from(color), ));
        } else {
            cmd.entity(tile_enti).insert((
                Transform::from_translation(Vec2::splat(f32::INFINITY).extend(0.)),
                Visibility::default(),
                SpriteTile
            ));
            let mut sprite_cfgs = Vec::new();
            let mut processing_as_sprite_cfgs = None;

            let len = seri.img_paths.len();
            for (key, path) in seri.img_paths.iter_mut() {
                let path_holder = ImagePathHolder::new(path.clone());
                let spritecfg_str_id_present = !key.trim().is_empty();

                if path_holder.is_err() && spritecfg_str_id_present
                && processing_as_sprite_cfgs != Some(false) {
                    sprite_cfgs.reserve(len);
                    sprite_cfgs.push(take(key));
                    processing_as_sprite_cfgs = Some(true);
                } else if processing_as_sprite_cfgs != Some(true) {
                    let path_holder = path_holder.unwrap();

                    let child_sprite = cmd.spawn((
                        TileChildSprite,
                        ChildOf(tile_enti),
                        BaseHolderRef{ base: tile_enti },
                        StrId::trunc(format!("{}", path_holder).replace("texture/", "")),
                        EntityZero,
                        path_holder,
                        Replicated,
                        my_z.clone(),
                    )).id();

                    if let Some(offset) = seri.offset {
                        cmd.entity(child_sprite).insert(Offset2D::from(offset));
                    }
                    if let Some(y_sort_origin) = seri.y_sort {
                        cmd.entity(child_sprite).insert(YSortOrigin(seri.offset.unwrap_or_default().1 + y_sort_origin - 10.0));
                    }
                    processing_as_sprite_cfgs = Some(false);
                }
            }
            if !sprite_cfgs.is_empty() {
                let sprite_cfgs_str_ids = SampleSpritesFromStrIds::new(sprite_cfgs);
                cmd.entity(tile_enti).insert(sprite_cfgs_str_ids);
            }
        }
    });
    cmd.insert_resource(res_tile_cats);
}

#[allow(unused_parens)]
pub fn init_childrensprite(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    ezero_img_path: Query<(Option<&ImagePathHolder>, Has<SpriteConfig>), (With<EntityZero>,)>,
    childrensprite_query: Query<
        (Entity, AnyOf<(&ImagePathHolder, &EntityZeroRef)>),
        (
            Without<AcAnimationProgresses>,
            Or<(Changed<ImagePathHolder>, Changed<EntityZeroRef>)>,
            With<TileChildSprite>,
            Without<Sprite>,
            Without<TilemapId>,
            Without<Children>,
            Without<TileShader>,
            common::AnyDisabling,
        ),
    >,
) {
    let mut to_insert = Vec::new();
    for (entity, (image_path_holder, ezero_ref)) in childrensprite_query.iter() {
        if let Some(img_path_holder) = image_path_holder {
            trace!(target: "childrensprite_init","Inserting Sprite for entity {:?} with direct ImagePathHolder: {:?}", entity, img_path_holder.path());
            to_insert.push((
                entity,
                Sprite {
                    image: asset_server.load(img_path_holder.path()),
                    ..Default::default()
                },
            ));
        } else if let Some(ezero_ref) = ezero_ref {
            let Ok((img_path_holder, is_ezero_a_spriteconfig)) = ezero_img_path.get(ezero_ref.0)
            else {
                error!(target: "childrensprite_init","Entity {:?} has EntityZeroRef {:?} but the referenced entity doesn't exist", entity, ezero_ref.0);
                continue;
            };
            if is_ezero_a_spriteconfig {
                continue;
            }
            let Some(img_path_holder) = img_path_holder else {
                error!(target: "childrensprite_init","Entity {:?} has EntityZeroRef {:?} but the referenced entity has no ImagePathHolder", entity, ezero_ref.0);
                continue;
            };

            trace!(target: "childrensprite_init","Inserting Sprite for entity {:?} via EntityZeroRef {:?}, path: {:?}", entity, ezero_ref.0, img_path_holder.path());
            to_insert.push((
                entity,
                Sprite {
                    image: asset_server.load(img_path_holder.path()),
                    ..Default::default()
                },
            ));
        } else {
            error!(target: "childrensprite_init","Entity {:?} has neither ImagePathHolder nor EntityZeroRef", entity);
        }
    }
    cmd.try_insert_batch(to_insert);
}

#[allow(unused_parens)]
pub fn add_handles(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    ezero_id_query: Query<
        (Entity, &TileStrId, &TileImagePaths),
        (
            With<EntityZero>,
            Without<TileHashIdsHandles>,
            Changed<TileImagePaths>,
        ),
    >,
) {
    let mut comps = Vec::new();
    for (enti, str_id, tile_image_paths) in ezero_id_query.iter() {
        let tile_handles = TileHashIdsHandles::from_paths(&asset_server, tile_image_paths.clone());

        match tile_handles {
            Ok(tile_handles) => {
                trace!(target: TILE_INIT, "Adding TileHandles for tile '{}'", str_id);
                comps.push((enti, tile_handles));
            }
            Err(err) => {
                error!(target: TILE_INIT, "Failed to create TileHandles for tile '{}': {}", str_id, err);
            }
        }
    }
    cmd.try_insert_batch(comps);
}

#[allow(unused_parens)]
pub fn map_min_dist_tiles(
    mut cmd: Commands,
    mut seris_handles: ResMut<TileSerisHandles>,
    mut assets: ResMut<Assets<TileSeri>>,
    tiles_map: Res<TileEntityMap>,
    tile_cats: Res<TileEntsWithinTag>,
) {
    let mut keep_away: EntityHashMap<HashSet<Entity>> = EntityHashMap::default();
    let mut comps = Vec::with_capacity(seris_handles.handles.len() / 10);
    let mut comps2 = Vec::with_capacity(seris_handles.handles.len() / 10);

    for handle in seris_handles.handles.drain(..) {
        let Some(seri) = assets.remove(&handle) else {
            continue;
        };

        let Some(min_distances) = seri.min_distances else {
            continue;
        };

        if min_distances.is_empty() {
            continue;
        }

        let Ok(tile_ent) = tiles_map.0.get_cloned(&seri.id) else {
            continue;
        };

        let mut min_dists = MinDistancesMap::default();

        for (tile_id, min_dist) in min_distances {
            if let Some(cat) = tile_id.strip_prefix("c.")
                && let Some(cat_entities) = tile_cats.0.get(&Tag::trunc(cat))
            {
                for cat_tile_ent in cat_entities {
                    min_dists.0.insert(*cat_tile_ent, min_dist);
                    if cat_tile_ent != &tile_ent {
                        keep_away.entry(*cat_tile_ent).or_default().insert(tile_ent);
                    }
                }
            } else if let Ok(other_tile_ent) = tiles_map.0.get_cloned(&tile_id) {
                min_dists.0.insert(other_tile_ent, min_dist);
                if other_tile_ent != tile_ent {
                    keep_away
                        .entry(other_tile_ent)
                        .or_default()
                        .insert(tile_ent);
                }
            } else {
                warn!(
                    "Tile '{}' min_distances references unknown tile id '{}'",
                    seri.id, tile_id
                );
                continue;
            };
        }

        if min_dists.0.is_empty() {
            continue;
        }

        comps.push((tile_ent, min_dists));
    }

    for (tile_ent, ents) in keep_away {
        comps2.push((tile_ent, KeepDistanceFrom(ents.into_iter().collect())));
    }
    cmd.try_insert_batch(comps);
    cmd.try_insert_batch(comps2);
}

#[allow(unused_parens)]
pub fn map_portal_tiles(
    mut cmd: Commands,
    mut portals_ezero_query: Query<
        (Entity, &TileStrId, &mut PortalSeri),
        (With<EntityZero>, common::AnyDisabling, Changed<PortalSeri>),
    >,
    tiles_map: Res<TileEntityMap>,
) {
    info!("Mapping portal tiles");
    portals_ezero_query.iter_mut().for_each(|(ent, str_id, mut portal_seri)| {
            let Ok(tile_ent) = tiles_map.0.get_cloned(&portal_seri.oe_tile) else {
                error!(target: PORTAL_INIT, "Portal tile {} to '{}' references unknown oe_tile '{}'", str_id, portal_seri.dest_dimension, portal_seri.oe_tile);
                return;
            };
            info!(target: PORTAL_INIT, "Mapping portal tile '{}' to destination dimension '{}'", str_id, portal_seri.dest_dimension);
            cmd.entity(ent).insert(PortalRecipe{
                dest_dimension: Entity::PLACEHOLDER,
                oe_portal_tile: tile_ent,
                tags: TagSet::new(take(&mut portal_seri.oe_op_tags)), //SETEARLO DESPUÉS
                op_i: portal_seri.op_i.unwrap_or(-1),
                min_val: portal_seri.min.unwrap_or(0.),
                max_val: portal_seri.max.unwrap_or(1.),
                one_way: portal_seri.one_way.unwrap_or(false),
            });
        });
}

#[allow(unused_parens)]
pub fn validate_portal_recipes(
    mut cmd: Commands,
    portal_recipes: Query<(Entity, &PortalRecipe), (Changed<PortalRecipe>)>,
    dimension_query: Query<Option<&DimensionRootOplist>>,
    oplist_query: Query<(&OperationList, Option<&TagSet>)>,
) {
    for (ezero_portal, recipe) in portal_recipes.iter() {
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
        let mut found = false;
        let searched_tags = &recipe.tags;

        //start recursive search here through every oplist's bifurcations
        let mut oplist_queue = vec![root_oplist.0];
        let mut visited = EntityHashSet::new();

        'search_loop: while let Some(current_oplist_ent) = oplist_queue.pop() {
            if visited.contains(&current_oplist_ent) {
                continue;
            }
            visited.insert(current_oplist_ent);

            let Ok((oplist, tag_set)) = oplist_query.get(current_oplist_ent) else {
                continue;
            };
            if let Some(tags) = tag_set {
                if tags.intersects(searched_tags) {
                    found = true;
                    break 'search_loop;
                }
            }
            for bifurcation in &oplist.bifurcations {
                if let Some(bifur_oplist_ent) = bifurcation.oplist {
                    if !visited.contains(&bifur_oplist_ent) {
                        oplist_queue.push(bifur_oplist_ent);
                    }
                }
            }
        }
        if found {
            debug!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} successfully found oplist with intersecting tagset for {:?}.", recipe.dest_dimension, searched_tags);
            cmd.entity(ezero_portal).try_insert(AwaitingStartSearch);
        } else {
            error!(target: PORTAL_INIT, "PortalRecipe with dest_dimension entity {:?} could not find any oplist's tagset which intersects {:?}.", recipe.dest_dimension, searched_tags);
            cmd.entity(ezero_portal)
                .try_remove::<AwaitingStartSearch>();
        }
    }
}

macro_rules! run_suitable_pos_search_logic {
    (
        target: $target:expr,
        searched_entity_label: $searched_entity_label:expr,
        cmd: $cmd:ident,
        searching_entities: $searching_entities:ident,
        search_params: $search_params:ident,
        make_search_request: $make_search_request:ident,
        handle_success_event: $handle_success_event:ident,
        handle_pending_failure: $handle_pending_failure:ident,
    ) => {{
        $search_params.pending_by_filter.clear();
        for (ent, &dim_ref, &my_pos, &ezero_ref, _, searching_for) in $searching_entities.iter() {
            if let Some(SearchingForSuitablePos { filtered_op_ent }) = searching_for {
                $search_params.pending_by_filter.insert(*filtered_op_ent, (ent, my_pos, dim_ref, ezero_ref));
            }
        }

        $searching_entities
            .iter()
            .for_each(|(search_ent, _dim_ref, &global_pos, ezero_ref, is_awaiting_start, ..)| {
                if !is_awaiting_start {
                    return;
                }
                $cmd.entity(search_ent).try_remove::<AwaitingStartSearch>();

                let Some((op_filter_ent, probe)) =
                    $make_search_request(&mut $cmd, search_ent, global_pos, *ezero_ref)
                else {
                    return;
                };

                info!(
                    target: $target,
                    "Starting suitable-pos search for {} entity {:?} at position {:?}",
                    $searched_entity_label,
                    search_ent,
                    global_pos
                );

                $cmd.entity(search_ent)
                    .try_insert(SearchingForSuitablePos { filtered_op_ent: op_filter_ent });
                $search_params.pos_searches_msgs_to_write.push(probe);
                $search_params.searches_started_this_call.insert(op_filter_ent, search_ent);
            });

        'successful_searches: for suitable_pos in $search_params.reader_search_successful.read() {
            let ss_filtered_op_ent = suitable_pos.op_filter_ent;
            if $search_params.successful_searches.contains(&ss_filtered_op_ent) {
                trace!(
                    target: $target,
                    "Ignoring duplicate SuitablePosFound for studied_op_ent {:?}",
                    ss_filtered_op_ent
                );
                continue 'successful_searches;
            }
            let (search_ent, my_pos, dim_ref, ezero_ref) =
                if let Some(search_ent) = $search_params.searches_started_this_call.remove(&ss_filtered_op_ent) {
                    let Ok((_, &orig_dim_ref, &orig_pos, &orig_tile_ref, ..)) =
                        $searching_entities.get(search_ent)
                    else {
                        continue 'successful_searches;
                    };
                    (search_ent, orig_pos, orig_dim_ref, orig_tile_ref)
                } else if let Some((search_ent, my_pos, dim_ref, ezero_ref)) =
                    $search_params.pending_by_filter.remove(&ss_filtered_op_ent)
                {
                    (search_ent, my_pos, dim_ref, ezero_ref)
                } else {
                    continue 'successful_searches;
                };

            if $handle_success_event(
                &mut $cmd,
                search_ent,
                my_pos,
                dim_ref,
                ezero_ref,
                suitable_pos.found_pos,
                ss_filtered_op_ent,
            ) {
                $search_params.successful_searches.insert(ss_filtered_op_ent);
            }
        }

        for failed_search in $search_params.mreader_search_failed.read() {
            if $search_params.successful_searches.contains(&failed_search.0) {
                continue; //not actually a failed search!
            }
            if $search_params.searches_started_this_call.remove(&failed_search.0).is_some() {
                error!(
                    target: $target,
                    "Failed to find suitable pos for a {} entity, {:?}",
                    $searched_entity_label,
                    failed_search.0
                );
                $cmd.entity(failed_search.0).try_despawn();
                continue;
            }
            if let Some((search_ent, global_pos, dim_ref, ezero_ref)) =
                $search_params.pending_by_filter.remove(&failed_search.0)
            {
                $handle_pending_failure(search_ent, global_pos, dim_ref, ezero_ref, failed_search.0);
            }
        }
    }};
}

#[allow(unused_parens)]
pub fn instantiate_portal(
    mut cmd: Commands,
    portals: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef, Has<AwaitingStartSearch>, Option<&SearchingForSuitablePos>),(Without<EntityZero>),
    >,
    ezero_query: Query<(&TileStrId, Option<&PortalRecipe>), (With<EntityZero>,)>,
    dimension_query: Query<(&DimensionRootOplist), ()>,
    mut mass_collected: ResMut<MassCollectedTiles>,
    mut register_pos: ResMut<ImportantRegisteredPositions>,
    clone_spawn_param_set: CloneSpawnParamSet,
    mut search_params: SearchParams,
) {
    let make_search_request =
        |cmd: &mut Commands,
         portal_ent: Entity,
         global_pos: GlobalTilePos,
         ezero_ref: EntityZeroRef| -> Option<(Entity, TerrainProbe)> {
            let Ok((str_id, portal_recipe_opt)) = ezero_query.get(ezero_ref.0) else {
                error!(target: PORTAL_INIT, "Portal tile entity {:?} references an EntityZero {:?} which no longer exists.", portal_ent, ezero_ref.0);
                return None;
            };
            let Some(portal_recipe) = portal_recipe_opt else {
                error!(target: PORTAL_INIT, "Portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", portal_ent, ezero_ref.0);
                return None;
            };
            let Ok((&dimension_root_oplist)) = dimension_query.get(portal_recipe.dest_dimension) else {
                error!(target: PORTAL_INIT,
                "PortalRecipe {} (entity: {:?}) references a DestDimension that doesn't exist ({:?}).", str_id, portal_ent, portal_recipe.dest_dimension,
                );
                return None;
            };

            let op_filter_ent = cmd.spawn((portal_recipe.to_op_filter(global_pos, dimension_root_oplist.0))).id();
            let probe = TerrainProbe::standard_spiral_probe(
                DimensionRef(portal_recipe.dest_dimension),
                op_filter_ent,
                global_pos,
            );
            Some((op_filter_ent, probe))
        };

    let mut handle_success_event = |cmd: &mut Commands,
                                    portal_ent: Entity,
                                    my_pos: GlobalTilePos,
                                    dim_ref: DimensionRef,
                                    ezero_ref: EntityZeroRef,
                                    found_pos: GlobalTilePos,
                                    filtered_op_ent: Entity| -> bool {
        let Ok((str_id, portal_recipe_opt)) = ezero_query.get(ezero_ref.0) else {
            error!(target: PORTAL_INIT, "SuitablePosFound for studied_op_ent {:?} but portal tile entity {:?} references an EntityZero {:?} which no longer exists.", filtered_op_ent, portal_ent, ezero_ref.0);
            return false;
        };
        let Some(portal_recipe) = portal_recipe_opt else {
            error!(target: PORTAL_INIT, "SuitablePosFound for studied_op_ent {:?} but portal tile entity {:?} references an EntityZero {:?} which doesn't have a PortalRecipe.", filtered_op_ent, portal_ent, ezero_ref.0);
            return false;
        };
        let portal_recipe = portal_recipe.clone();

        info!(target: PORTAL_INIT,
            "Found suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}, found pos: {:?}", str_id, portal_ent, dim_ref.0, my_pos, portal_recipe.dest_dimension, found_pos
        );

        let oe_dim_ref = DimensionRef(portal_recipe.dest_dimension);
        cmd.entity(filtered_op_ent).try_despawn();

        let oe_portal_tileref = EntityZeroRef(portal_recipe.oe_portal_tile);
        debug!(target: PORTAL_INIT, "OE Portal TileRef: {:?}", oe_portal_tileref);

        let oe_portal = mass_collected.clonespawn_and_push_tile(
            cmd,
            oe_portal_tileref,
            found_pos,
            oe_dim_ref,
            &clone_spawn_param_set,
        );
        register_pos.exempt_entity_from_mindist_checks(oe_portal);

        cmd.entity(portal_ent)
            .try_insert(PortalTo::new(oe_portal))
            .try_remove::<(SearchingForSuitablePos, AwaitingStartSearch)>();

        cmd.entity(oe_portal)
            .try_remove::<(AwaitingStartSearch)>()
            .try_insert(DeleteOtherTiles {
                spared_z: HashSet::from_iter(vec![AcZ::new(-900.0)]),
                extra_radius: 2,
                ..Default::default()
            });

        if portal_recipe.one_way == false {
            cmd.entity(oe_portal).try_insert(PortalTo::new(portal_ent));
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

            error!(target: PORTAL_INIT,
                "Failed to find suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}", str_id, portal_ent, dim_ref.0, global_pos, portal_template.dest_dimension
            );
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

    search_params
        .ew_pos_search
        .write_batch(search_params.pos_searches_msgs_to_write.drain(..));
}
