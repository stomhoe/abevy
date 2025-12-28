use bevy::{asset, ecs::{entity::{EntityHashMap, EntityHashSet}, entity_disabling::Disabled, }, platform::collections::{HashMap, HashSet}, render::sync_world::SyncToRenderWorld};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::*;
use ::dimension_shared::*;
use ::game_common::{color_sampler_resources::*, game_common_components::*, game_common_components_samplers::*, *};
use bevy_ecs_tilemap::tiles::TilePos;
use sprite_animation_shared::AcAnimationProgresses;
use ::sprite_shared::{sprite_scale_offset::Offset2D, *};
use ::tilemap_shared::*;

use crate::{chunking_resources::LoadedChunks, terrain_gen::{terrgen_messages::*, terrgen_resources::RegisteredPositions}, tile::{tile_components::*,  tile_materials::*, tile_resources::*, tile_shader_components::{TileShader, TileShaderRef}, tile_shader_resources::*} };
use crate::terrain_gen::terrgen_resources::MassCollectedTiles;

use std::mem::take;



#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands, 
    seris_handles: Res<TileSerisHandles>, mut assets: ResMut<Assets<TileSerialization>>,
    shader_map: Res<TileShaderEntityMap>,
    tiling_map: Option<Res<TileEntitiesMap>>,
    color_map: Res<ColorWeightedSamplersMap>,

) {
    if tiling_map.is_some() { return; }
    let mut tiling_map = TileEntitiesMap::default();

    let holder = cmd.spawn((TilesEguiHolder, )).id();
    cmd.spawn((TileInstancesHolder, ChildOf(holder)));

    let egui_portal_holder = cmd.spawn((PortalsZeroEguiHolder, ChildOf(holder))).id();

    let mut res_tile_cats = TileCategories::default();

    for handle in seris_handles.handles.iter() {
        //info!("Loading TileSeri from handle: {:?}", handle);
        let Some(seri) = assets.get_mut(handle) else { continue; };

        let str_id = match TileStrId::new_with_result(seri.id.clone(), Tile::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create TileStrId for tile '{}': {}", seri.id, err);
                continue;
            }
        };
        let my_z = AcZ(seri.z);
        let tile_enti = cmd.spawn((
            Tile, Replicated, str_id.clone(), Disabled,
            EntityPrefix::new_truncated("Tile"), 
            my_z.clone(),
            EntityZero,
            ChildOf(holder),
        )).id();

        if let Ok(existing) = tiling_map.0.get(&str_id) {
            error!("Tile with '{}' already in TilingEntityMap : {:?}", str_id, existing);
            cmd.entity(tile_enti).try_despawn();
            continue;
        }
        tiling_map.0.overwrite(&str_id, tile_enti);

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
        if let Some(ref color_map_str) = seri.color_map {
            if !color_map_str.is_empty() {
                match color_map.0.get(color_map_str) {
                    Ok(color_sampler_ent) => {
                        cmd.entity(tile_enti).insert(ColorSamplerRef(color_sampler_ent));
                    }
                    Err(err) => {
                        error!("Tile '{}': Weighted color sampler with id '{}' not found: {}", str_id, color_map_str, err);
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
        if seri.sprite != Some(true) {//todo hacer q se puedan persistir tilemap tiles
            
            cmd.entity(tile_enti).insert(TileImagePaths(take(&mut seri.img_paths)));

            if let Some(shader_str) = &seri.shader {
                if shader_str.len() > 2 {
                    match shader_map.0.get(shader_str) {
                        Ok(shader_ent) => {
                            cmd.entity(tile_enti).insert(TileShaderRef(shader_ent));
                        }
                        Err(err) => {
                            warn!("Tile '{}' references missing shader '{}': {}", str_id, shader_str, err);
                        }
                    }
                } else if shader_str.len() > 0 {
                    warn!("Tile {} shader {} is too short for a shader", str_id, shader_str);
                }
            }

            cmd.entity(tile_enti).insert_if_new((TileColor::from(color), ));
        }
        else{// sprite tile
             cmd.entity(tile_enti).insert((
                Transform::default(),
                Visibility::default(),
            ));
            let mut sprite_cfgs = Vec::new();
            for (key, path) in seri.img_paths.iter_mut() {
                let path_holder = ImagePathHolder::new(take(path));
                if  (path.trim().is_empty() || path_holder.is_err()) && !key.trim().is_empty() {
                    sprite_cfgs.push(take(key));
                } else{
                    let path_holder = path_holder.unwrap();

                    let child_sprite = cmd.spawn((
                        Replicated,
                        path_holder,
                        ChildOf(tile_enti),
                        BaseHolderRef{ base: tile_enti },
                        my_z.clone(),
                    )).id();
                    
                    if let Some(offset) = seri.offset {
                        cmd.entity(child_sprite).insert(Offset2D::from(offset));
                    }
                    if let Some(y_sort_origin) = seri.y_sort {
                        cmd.entity(child_sprite).insert(YSortOrigin(seri.offset.unwrap_or_default().1 + y_sort_origin - 10.0));
                    }
                    break;
                }
            }
            if !sprite_cfgs.is_empty() {
                let sprite_cfgs = SpriteConfigStrIds::new(sprite_cfgs);
                cmd.entity(tile_enti).insert(sprite_cfgs);
            }
        }
        if let Some(cats) = &seri.cats {
            for cat in cats.iter() {
                if cat.trim().is_empty() { continue; }
                res_tile_cats.0.entry(Tag::new_truncated(cat)).or_default().insert(tile_enti);
            }
        }
    }
    cmd.insert_resource(res_tile_cats);
    cmd.insert_resource(tiling_map);
}

#[allow(unused_parens)]
pub fn init_tile_sprite(mut cmd: Commands, 
    asset_server: Res<AssetServer>,
    ezero_img_path: Query<&ImagePathHolder, (Without<EntityZeroRef>, Or<(With<Disabled>, Without<Disabled>)>)>,
    query: Query<(Entity, AnyOf<(&ImagePathHolder, &EntityZeroRef)>),(Without<AcAnimationProgresses>, 
        Without<Sprite>, Without<TilePos>, Without<Children>, Without<TileShader>, Or<(Changed<ImagePathHolder>, Changed<EntityZeroRef>)>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    let mut to_insert = Vec::new();
    for (entity, (image_path_holder, ezero_ref)) in query.iter() {
        if let Some(img_path_holder) = image_path_holder {
            trace!(target: "init_tile_sprite","Inserting Sprite for entity {:?} with direct ImagePathHolder: {:?}", entity, img_path_holder.path());
            to_insert.push((entity, Sprite{
                image: asset_server.load(img_path_holder.path()),
                ..Default::default()
            }));
        }
        else if let Some(ezero_ref) = ezero_ref {
            let Ok(img_path_holder) = ezero_img_path.get(ezero_ref.0) else {
                continue;
            };
            trace!(target: "init_tile_sprite","Inserting Sprite for entity {:?} via EntityZeroRef {:?}, path: {:?}", entity, ezero_ref.0, img_path_holder.path());
            to_insert.push((entity, Sprite{
                image: asset_server.load(img_path_holder.path()),
                ..Default::default()
            }));
        } else {
            error!(target: "init_tile_sprite","Entity {:?} has neither ImagePathHolder nor EntityZeroRef", entity);
        }
    }
    cmd.insert_batch(to_insert);
}


#[allow(unused_parens)]
pub fn add_handles(  
    mut cmd: Commands,  asset_server: Res<AssetServer>,
    query: Query<(Entity, &TileStrId, &TileImagePaths),(With<EntityZero>, Without<TilePos>, Without<TileHidsHandles>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    for (enti, str_id, tile_image_paths) in query.iter() {
        let tile_handles = TileHidsHandles::from_paths(&asset_server, tile_image_paths.clone(), );

        match tile_handles {
            Ok(tile_handles) => {
                debug!(target: "tile_init", "Adding TileHandles for tile '{}'", str_id);
                cmd.entity(enti).insert(tile_handles);
            }
            Err(err) => {
                error!(target: "tile_init", "Failed to create TileHandles for tile '{}': {:?}", str_id, err);
            }
        }
    }
}

#[allow(unused_parens)]
pub fn map_min_dist_tiles(mut cmd: Commands, 
    mut seris_handles: ResMut<TileSerisHandles>, mut assets: ResMut<Assets<TileSerialization>>,
    tiles_map: Res<TileEntitiesMap>,
    tile_cats: Res<TileCategories>,
) {
    let mut keep_away: EntityHashMap<HashSet<Entity>> = EntityHashMap::default();

    for handle in seris_handles.handles.drain(..) {
        let Some(seri) = assets.remove(&handle) else { continue; };

        let Some(min_distances) = seri.min_distances else { continue; };

        if min_distances.is_empty() { continue; }

        let Ok(tile_ent) = tiles_map.0.get(&seri.id) else { continue; };

        let mut min_dists = MinDistancesMap::default();

        for (tile_id, min_dist) in min_distances {

            if let Some(cat) = tile_id.strip_prefix("c.") && let Some(cat_entities) = tile_cats.0.get(&Tag::new_truncated(cat)) {

                for cat_tile_ent in cat_entities {
                    min_dists.0.insert(*cat_tile_ent, min_dist);
                    if cat_tile_ent != &tile_ent {
                        keep_away.entry(*cat_tile_ent).or_default().insert(tile_ent);
                    }
                }
            }
            else if let Ok(other_tile_ent) = tiles_map.0.get(&tile_id) {
                min_dists.0.insert(other_tile_ent, min_dist);
                if other_tile_ent != tile_ent {
                    keep_away.entry(other_tile_ent).or_default().insert(tile_ent);
                }
            } else {
                warn!("Tile '{}' min_distances references unknown tile id '{}'", seri.id, tile_id);
                continue;
            };
        }

        if min_dists.0.is_empty() { continue; }
        
        cmd.entity(tile_ent).insert(min_dists);
    }

    for (tile_ent, ents) in keep_away {
        cmd.entity(tile_ent).insert(KeepDistanceFrom(ents.into_iter().collect()));
    }
}

#[allow(unused_parens)]
pub fn map_portal_tiles(mut cmd: Commands, 
    mut query: Query<(Entity, &TileStrId, &mut PortalSeri, ),(With<Disabled>)>,
    tiles_map: Res<TileEntitiesMap>,
) {
    info!("Mapping portal tiles");
    for (ent, str_id, mut portal_seri) in query.iter_mut() {
        let Ok(tile_ent) = tiles_map.0.get(&portal_seri.oe_tile) else { 
            error!(target:"portal_init", "Portal tile {} to '{}' references unknown oe_tile '{}'", str_id, portal_seri.dest_dimension, portal_seri.oe_tile);
            continue; 
        };
        info!(target:"portal_init", "Mapping portal tile '{}' to destination dimension '{}'", str_id, portal_seri.dest_dimension);
        cmd.entity(ent).insert(PortalRecipe{
            dest_dimension: Entity::PLACEHOLDER,
            oe_portal_tile: tile_ent,
            tags: Tags::new(take(&mut portal_seri.oe_tags)), //SETEARLO DESPUÉS
            op_i: portal_seri.op_i,
            min_val: portal_seri.lim_below,
            max_val: portal_seri.lim_above,
            one_way: portal_seri.one_way,
        });
    }
}

#[allow(unused_parens)]
pub fn instantiate_portal(mut cmd: Commands,
    ori_tile_str_id_query: Query<&TileStrId, (With<Disabled>)>,
    new_portals: Query<(Entity, &PortalRecipe, &GlobalTilePos, &DimensionRef, &EntityZeroRef),(Without<SearchingForSuitablePos>, )>,
    pending_search: Query<(Entity, &SearchingForSuitablePos, &PortalRecipe, &GlobalTilePos, &DimensionRef, &EntityZeroRef),()>,
    dimension_query: Query<(&HashId, &DimensionRootOplist), ()>,
    mut ew_pos_search: MessageWriter<TerrainProbe>, 
    mut mass_collected: ResMut<MassCollectedTiles>,
    mut mreader_search_successful: MessageReader<SuitablePosFound>,
    mut mreader_search_failed: MessageReader<SearchFailed>, 
    mut register_pos: ResMut<RegisteredPositions>

) {
    let mut started_searches: EntityHashMap<Entity> = EntityHashMap::new();
    let mut pos_searches = Vec::new();

    for (portal_ent, portal_recipe, &global_pos, dim_ref, tile_ref) in new_portals.iter() {

        
        let str_id = ori_tile_str_id_query.get(tile_ref.0).map(|id| id.as_str()).unwrap_or_default();
        
        let Ok((&dimension_hash_id, &dimension_root_oplist)) = dimension_query.get(portal_recipe.dest_dimension) 
        else {
                error!(target:"portal_init",
                "PortalRecipe {} (entity: {:?}) references a DestDimension that doesn't exist ({:?}). Entity's own dimension: {:?}, pos: {:?}, ", str_id, portal_ent, portal_recipe.dest_dimension, dim_ref.0, global_pos,
            );
            cmd.entity(portal_ent).remove::<PortalRecipe>();
            continue;
        };
        let op_filter = portal_recipe.to_op_filter(global_pos, dimension_root_oplist.0);


        let op_filter_ent = cmd.spawn((op_filter)).id();

        cmd.entity(portal_ent).try_insert(SearchingForSuitablePos{ filtered_op_ent: op_filter_ent });

        pos_searches.push(TerrainProbe::standard_spiral_probe(dimension_hash_id, op_filter_ent, global_pos));
        started_searches.insert(op_filter_ent, portal_ent);
    }

    let mut successful_searches: EntityHashSet = EntityHashSet::new();

    let mut handle_success = |this_end_portal: Entity, portal_template: &PortalRecipe, 
        found_pos: GlobalTilePos, my_orig_tile_ref: EntityZeroRef, filtered_op_ent: Entity| 
    {
        cmd.entity(this_end_portal).remove::<(SearchingForSuitablePos, PortalRecipe)>();
        let oe_dim_ref = DimensionRef(portal_template.dest_dimension);
        cmd.entity(filtered_op_ent).try_despawn();
        
        register_pos.0.entry(portal_template.oe_portal_tile)
            .or_default()
            .push((oe_dim_ref, found_pos));

        let oe_portal_tileref = if portal_template.oe_portal_tile == this_end_portal {
            my_orig_tile_ref
        } else {
            EntityZeroRef(portal_template.oe_portal_tile)
        };

        debug!(target:"portal_init", "OE Portal TileRef: {:?}", oe_portal_tileref);

        let oe_portal = 
        mass_collected
        .clonespawn_and_push_tile(&mut cmd, oe_portal_tileref, found_pos, oe_dim_ref, OplistSize::default());

        cmd.entity(this_end_portal).insert(PortalConnection::new(oe_portal));

        cmd.entity(oe_portal).remove::<PortalRecipe>();

        if portal_template.one_way {return;}

        cmd.entity(oe_portal).insert(PortalConnection::new(this_end_portal));

        debug!(target:"portal_init", "Instantiated oe-portal '{}' at position {:?} in dimension {:?}", oe_portal, found_pos, portal_template.dest_dimension);

    };

    'successful_searches: for search_successful_msg in mreader_search_successful.read() {
        let ss_filtered_op_ent = search_successful_msg.op_filter_ent;
        if successful_searches.contains(&ss_filtered_op_ent) {
            trace!(target:"portal_init", "Ignoring duplicate SuitablePosFound for studied_op_ent {:?}", ss_filtered_op_ent);
            continue 'successful_searches;
        }

        if let Some(portal_ent) = started_searches.remove(&ss_filtered_op_ent) {
            let Ok((_, portal_recipe, &_, &_, &orig_tile_ref)) = new_portals.get(portal_ent) else {
                error!(target:"portal_init", "SuitablePosFound for studied_op_ent {:?} but portal entity which started the search {:?} is no longer spawned", ss_filtered_op_ent, portal_ent);
                continue 'successful_searches;
            };
            successful_searches.insert(ss_filtered_op_ent);
            debug!(target:"portal_init", "added ss_filtered_op_ent {:?} to successful searches", ss_filtered_op_ent);
            handle_success(portal_ent, portal_recipe, search_successful_msg.found_pos.clone(), orig_tile_ref, ss_filtered_op_ent);
            continue 'successful_searches;
        }

        for (ent, searching_for, portal_template, &my_pos, &dim_ref, &orig_tile_ref) in pending_search.iter() {
            if !successful_searches.contains(&ss_filtered_op_ent){
                if ss_filtered_op_ent == searching_for.filtered_op_ent  {
                    successful_searches.insert(ss_filtered_op_ent);
                    let str_id = ori_tile_str_id_query.get(orig_tile_ref.0).map(|id| id.as_str()).unwrap_or_default();
                    info!(target:"portal_init",
                        "Found suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}, found pos: {:?}", str_id, ent, dim_ref.0, my_pos, portal_template.dest_dimension, search_successful_msg.found_pos
                    );
                    handle_success(ent, portal_template, search_successful_msg.found_pos.clone(), orig_tile_ref, ss_filtered_op_ent);
                    continue 'successful_searches;
                }
            } else {
                trace!(target:"portal_init", "Ignoring duplicate SuitablePosFound for studied_op_ent {:?} in pending searches", ss_filtered_op_ent);
                continue 'successful_searches;
            }
        }
    }

    for failed_search in mreader_search_failed.read() {

        if successful_searches.contains(&failed_search.0) { 
            continue;//not actually a failed search!
        }

        if started_searches.remove(&failed_search.0).is_some() {
            error!(target:"portal_init", "Failed to find suitable pos for a portal tile, {:?}", failed_search.0);
            cmd.entity(failed_search.0).try_despawn();

            continue;
        }

        for (portal_ent, searching_for, portal_template, &global_pos, dim_ref, tile_ref) in pending_search.iter() {
            let str_id = ori_tile_str_id_query.get(tile_ref.0).map(|id| id.as_str()).unwrap_or_default();
            if failed_search.0 == searching_for.filtered_op_ent {
                error!(target:"portal_init",
                    "Failed to find suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}", str_id, portal_ent, dim_ref.0, global_pos, portal_template.dest_dimension
                );
            }
        }
    }
    ew_pos_search.write_batch(pos_searches);
}

#[allow(unused_parens)]
pub fn client_sync_tile(
    mut cmd: Commands, 
    query: Query<(Entity, &EntityZeroRef, &GlobalTilePos, &DimensionRef, ), (Added<Replicated>, With<Tile>, Or<(Without<Disabled>, With<Disabled>)>)>,
    loaded_chunks: Res<LoadedChunks>,
    mut collected: Res<MassCollectedTiles>//TODO synquear tiles de tilemaps si es posible

) {
    //let mut tiles_to_tmap_process = Vec::new();
    // for (tile_ent, &orig_ref, &global_pos, &dim_ref, ) in query.iter() {
    //     let Ok((tile_strid, is_child, sprite)) = ori_query.get(orig_ref.0) else{
    //         error!("Original tile entity {} is despawned", orig_ref.0);
    //         continue;
    //     };
    //     info!("Client instantiated replicated tile '{}' at position {:?} in dimension {:?}", tile_strid, global_pos, dim_ref.0);

    //     let chunk_pos: ChunkPos = global_pos.into();

        
    //     if is_child && let Some(&chunk) = loaded_chunks.0.get(&(dim_ref, chunk_pos)) {
                //despawn prev tile if at same pos
            
    //     } 
    // }
    //ewriter_tmap_process.write_batch(tiles_to_tmap_process);

}

#[allow(unused_parens)]
pub fn make_child_of_chunk(mut cmd: Commands, 

    query: Query<(Entity, &EntityZeroRef, &GlobalTilePos, &DimensionRef, Has<Persisted>), (With<Tile>, Without<TilePos>, Or<(Changed<GlobalTilePos>, Changed<DimensionRef>)>, Or<(Without<Disabled>, With<Disabled>)>)>,
    loaded_chunks: Res<LoadedChunks>,
) {
    let mut child_ofs = Vec::new();
    for (ent, &ezero, &global_pos, &dim_ref, to_persist) in query.iter() {

        let chunk_pos: ChunkPos = global_pos.into();

        
        if to_persist {
            child_ofs.push((ent, ChildOf(dim_ref.0)));
            continue;
        }
        else{
            let Some(&chunk) = loaded_chunks.0.get(&(dim_ref, chunk_pos)) 
            else {
                cmd.entity(ent).try_despawn();
                continue;
            };
    
            child_ofs.push((ent, ChildOf(chunk)));
        }

    }
    cmd.try_insert_batch(child_ofs);
}
