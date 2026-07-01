use bevy::{
    ecs::entity::EntityHashMap,
    ecs::system::SystemParam,
    math::U16Vec2,
    platform::collections::HashSet,
    prelude::*,
};
use bevy_ecs_tilemap::prelude::{*, TilemapTexture::Vector};
use bevy_replicon::prelude::*;
use common::{TILEMAP_SYSTEM, common_components::HashId, common_resources::ImageSizeMap };
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::*;
use sprite_shared::{AcZ, YSortOrigin};
use ::tilemap_shared::*;
use crate::{
    chunking::MacroChunkU16IndexMatrix,
    tile::{
        tile_bundles::*,
        tile_delete_others_systems::TileDeleteOthersParamSet,
        tile_despawn_systems::*,
        tile_shader::tile_shader_components::*,
        tile_shader::tile_shader_resources::TileShaderEntityMap,
        tile_resources::*,
    },
    tile::U16TileIndex,
    tilemap_resources::*,
    tilemap_structs::*,
    tilemap_bundles::TilemapConfig,
};
use std::{collections::HashMap, mem::take};

#[derive(SystemParam)]
pub struct SystemLocals<'s> {
    pub changed_structs: Local<'s, HashSet<MapKey>>,
    pub tile_runtime_info: Local<'s, EntityHashMap<(TileRef, TileTextureIndex)>>,
    pub terrbl_debug_budget: Local<'s, u32>,
}



#[derive(SystemParam)]
pub struct SystemResources<'w> {
    pub collected_tiles: ResMut<'w, MassCollectedTiles>,
    pub image_size_map: Res<'w, ImageSizeMap>,
    pub texture_overlay_mat: ResMut<'w, Assets<TerrBlendMat>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub regpos_map: ResMut<'w, ImportantRegisteredPositions>,
    pub loaded_chunks: Res<'w, LoadedChunks>,
    pub loaded_macro_chunks: Res<'w, LoadedMacroChunks>,
    pub state: Res<'w, State<ClientState>>,
    pub tmap_map: ResMut<'w, TmapMap>,
    pub dimension_map: Res<'w, DimensionEntityMap>,
    pub tile_shader_map: Res<'w, TileShaderEntityMap>,
    pub tile_map: Res<'w, TileEntityMap>,
}

#[derive(SystemParam)]
pub struct ComponentsQueries<'w, 's> {
    pub min_dists_query: Query<'w, 's, &'static MinDistancesMap, common::AnyDisabling>,
    pub size_query: Query<'w, 's, &'static SizeInTiles, common::AnyDisabling>,
    pub keep_distance_query: Query<'w, 's, &'static KeepDistanceFrom, common::AnyDisabling>,
    pub persisted_query: Query<'w, 's, (), (With<Persisted>, common::AnyDisabling)>,
    pub z_query: Query<'w, 's, &'static AcZ, common::AnyDisabling>,
    pub handles_query: Query<'w, 's, &'static TileHashIdsHandles, common::AnyDisabling>,
    pub shader_ref_query: Query<'w, 's, &'static TileShaderRef, common::AnyDisabling>,
    pub sprite_tile_query: Query<'w, 's, (), (With<SpriteTile>, common::AnyDisabling)>,
    pub color_query: Query<'w, 's, &'static TileColor, common::AnyDisabling>,
    pub y_sort_query: Query<'w, 's, (), (With<YSortOrigin>, common::AnyDisabling)>,
    pub tile_ref_query: Query<'w, 's, &'static TileRef>,
    pub tile_index_query: Query<'w, 's, &'static U16TileIndex, common::AnyDisabling>,
    pub tile_texture_index_query: Query<'w, 's, &'static TileTextureIndex>,
    pub interaction_zones_query: Query<'w, 's, &'static InteractionZones, common::AnyDisabling>,
    pub macro_chunk_tile_indices_query: Query<'w, 's, &'static mut MacroChunkU16IndexMatrix>,
    pub delete_others_paramset: TileDeleteOthersParamSet<'w, 's>,
}

#[derive(SystemParam)]
pub struct ProcessTilesPreParams<'w, 's> {
    pub resources: SystemResources<'w>,
    pub tile_gathering_paramset: TileGatheringParamSet<'w, 's>,
    pub tile_components_queries: ComponentsQueries<'w, 's>,
    pub shader_query: Query<'w, 's, &'static TileShader, ()>,
    pub terrbl_query: Query<'w, 's, &'static TerrBlendParams>,
    pub terrbl_handle_query: Query<'w, 's, &'static mut MaterialTilemapHandle<TerrBlendMat>>,
    pub locals: SystemLocals<'s>,
}

#[allow(unused_parens, )]
pub fn process_tiles_pre(
    mut cmd: Commands,
    mut params: ProcessTilesPreParams,
) {
    let resources = &mut params.resources;
    let tile_components = &mut params.tile_components_queries;
    let locals = &mut params.locals;

    if *locals.terrbl_debug_budget == 0 {
        *locals.terrbl_debug_budget = 40;
    }
    locals.tile_runtime_info.clear();
    let is_host = *resources.state.get() == ClientState::Disconnected;

    if resources.collected_tiles.0.is_empty() { return; }

    let tiles_len = resources.collected_tiles.0.len();

    let mut tilemap_bundles = Vec::new();

    let mut to_insert_replicated = Vec::with_capacity(tiles_len/100);
    let mut spritetiles_to_remove_tmapbundle = Vec::with_capacity(tiles_len/20);

    let mut child_ofs_to_insert: Vec<(Entity, ChildOf)> = Vec::with_capacity(tiles_len);

    let mut i = 0;
    while i < resources.collected_tiles.0.len() {
        // To avoid borrow checker issues
        let (tile_ent, bundle_ptr) = {
            unsafe{
                let (tile_ent, bundle) = &mut resources.collected_tiles.0.get_mut(i).debug_unwrap_unchecked();
                (*tile_ent, bundle as *mut TileMassSpawnBundle)
            }
        };
        let bundle = unsafe { &mut *bundle_ptr };
        let Ok(templ_ent) = resources.tile_map.0.get_cloned(bundle.templ_ref.0) else {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} is despawned", bundle.templ_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };

        let Ok(&size_in_tiles) = tile_components.size_query.get(templ_ent) else {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} missing SizeInTiles", bundle.templ_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };
        let Ok(&tile_z_index) = tile_components.z_query.get(templ_ent) else {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} missing AcZ query access", bundle.templ_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };
        let tile_handles = tile_components.handles_query.get(templ_ent).ok().cloned();
        let shader_ref = tile_components.shader_ref_query.get(templ_ent).ok().copied();
        let color = tile_components.color_query.get(templ_ent).cloned();
        let is_spritetile = tile_components.sprite_tile_query.get(templ_ent).is_ok();
        let y_sort = tile_components.y_sort_query.get(templ_ent).is_ok();
        let to_persist = tile_components.persisted_query.get(templ_ent).is_ok();
        let min_dists = tile_components.min_dists_query.get(templ_ent).ok();
        let keep_distance_from = tile_components.keep_distance_query.get(templ_ent).ok();
        let interaction_zones = tile_components.interaction_zones_query.get(templ_ent).ok();
        let _dim_hash = bundle.dim_ref.0;
        if resources.dimension_map.0.get_cloned(_dim_hash).is_err() {
            error_once!(target: TILEMAP_SYSTEM, "Dimension hash {} is missing from DimensionEntityMap", _dim_hash);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        }

        if !resources.regpos_map.check_min_distances(
            &mut cmd,
            is_host,
            (
                tile_ent,
                templ_ent,
                bundle.dim_ref,
                bundle.gpos,
                min_dists,
                keep_distance_from,
            ),
            tile_components.min_dists_query,
        ) {
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            trace!(target: TILEMAP_SYSTEM, "Tile entity {:?} at gpos {:?} in dim {:?} despawned due to min distance check failure", tile_ent, bundle.gpos, bundle.dim_ref);
            continue;
        }

        if process_tile_despawns_from_templ(
            &mut tile_components.delete_others_paramset,
            &resources.regpos_map,
            &params.tile_gathering_paramset,
            tile_ent,
            bundle.templ_ref,
            bundle.dim_ref,
            bundle.gpos,
        ) {
            resources.collected_tiles.0.swap_remove(i);
            continue;
        }

        //cmd.entity(tile_ent).try_insert_if_new(Signature::from((ez_hash_id, _dim_hash, bundle.gpos)));
        let dimension_ent = resources.dimension_map.0.get_cloned(bundle.dim_ref.0).ok();

        let chunk_ent = resources.loaded_chunks.0.get(&(bundle.dim_ref, ChunkPos::from(bundle.gpos))).copied();
        let macro_chunk_pos = bundle.gpos.to_chunkpos().to_macrochunk_pos();
        let macro_chunk_ent = resources.loaded_macro_chunks.0.get(&(bundle.dim_ref, macro_chunk_pos)).copied();
        let is_chunk_loaded = chunk_ent.is_some() && macro_chunk_ent.is_some();

        if is_chunk_loaded {
            let Some(macro_chunk_ent) = macro_chunk_ent else {
                unreachable!();
            };
            let Ok(&tile_index) = tile_components.tile_index_query.get(templ_ent) else {
                error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} missing TileIndex", bundle.templ_ref.0);
                cmd.entity(tile_ent).try_despawn();
                resources.collected_tiles.0.swap_remove(i);
                continue;
            };
            let Ok(mut macro_chunk_tile_indices) = tile_components.macro_chunk_tile_indices_query.get_mut(macro_chunk_ent) else {
                error_once!(target: TILEMAP_SYSTEM, "Macrochunk entity {} missing MacroChunkTileIndices", macro_chunk_ent);
                continue;
            };
            if !macro_chunk_tile_indices.push_tile_index(macro_chunk_pos.to_chunkpos().to_tilepos(), bundle.gpos, tile_index) {
                error_once!(target: TILEMAP_SYSTEM, "Tile entity {} at {:?} did not fit in macrochunk {:?}", tile_ent, bundle.gpos, macro_chunk_ent);
            }
        }

        match (to_persist, is_host, is_spritetile, chunk_ent, dimension_ent) {
            (_, _, _, _, None) => {
                cmd.entity(tile_ent).try_despawn();
                resources.collected_tiles.0.swap_remove(i);
                continue;
            }
            (false, _, _, None, _) => {
                cmd.entity(tile_ent).try_despawn();
                resources.collected_tiles.0.swap_remove(i);
                continue;
            }
            (true, false, _, _, _) => {
                cmd.entity(tile_ent).try_despawn();
                resources.collected_tiles.0.swap_remove(i);
                continue;
            }
            (true, true, true, None, Some(dimension_ent)) => {
                child_ofs_to_insert.push((tile_ent, ChildOf(dimension_ent)));
                to_insert_replicated.push((tile_ent, Replicated));
                params.tile_gathering_paramset.insert_spritetile(tile_ent, bundle.dim_ref, bundle.gpos, interaction_zones);
                spritetiles_to_remove_tmapbundle.push(tile_ent);
                i += 1;
                continue;
            }
            (true, true, true, Some(_chunk_ent), Some(dimension_ent)) => {
                child_ofs_to_insert.push((tile_ent, ChildOf(dimension_ent)));
                to_insert_replicated.push((tile_ent, Replicated));
                params.tile_gathering_paramset.insert_spritetile(tile_ent, bundle.dim_ref, bundle.gpos, interaction_zones);
                spritetiles_to_remove_tmapbundle.push(tile_ent);
                i += 1;
                continue;
            }
            (true, true, false, Some(_chunk_ent), Some(dimension_ent)) => {
                child_ofs_to_insert.push((tile_ent, ChildOf(dimension_ent)));
                to_insert_replicated.push((tile_ent, Replicated));
            }
            //chunk descargado, es un persisted tmaptile
            (true, _, false, None, Some(_dimension_ent)) => {
                error!(target: TILEMAP_SYSTEM, "PERSISTED tmap tile entity {:?} at gpos {:?} in dim {:?} cannot be processed because its chunk is not loaded", tile_ent, bundle.gpos, bundle.dim_ref);
                resources.collected_tiles.0.swap_remove(i);
                continue;
            }
            (false, _, true, Some(chunk_ent), Some(_dimension_ent)) => {
                spritetiles_to_remove_tmapbundle.push(tile_ent);
                params.tile_gathering_paramset.insert_spritetile(tile_ent, bundle.dim_ref, bundle.gpos, interaction_zones);
                child_ofs_to_insert.push((tile_ent, ChildOf(chunk_ent)));
                i += 1;
                continue;
            }
            (false, _, false, Some(_chunk_ent), Some(_dimension_ent)) => {}
        }

        let Some(chunk_ent) = chunk_ent else {
            unreachable!();
        };

        bundle.tile_bundle.color = color.unwrap_or_default();

        let chunk_pos = ChunkPos::from(bundle.gpos);

        let tile_img_size = tile_handles
            .as_ref()
            .and_then(|handles| resources.image_size_map.0.get(&handles.first_handle().id()).copied())
            .unwrap_or(U16Vec2::ONE);

        process_tile_into_corresponding_tilemap(
            &mut cmd,
            tile_ent,
            bundle.templ_ref.0,
            size_in_tiles,
            &mut bundle.tile_bundle.visible,
            &mut bundle.tile_bundle.texture_index,
            &mut bundle.tile_bundle.tilemap_id,
            bundle.tile_bundle.position,
            tile_z_index,
            tile_handles.as_ref(),
            shader_ref.as_ref(),
            tile_img_size,
            &mut resources.tmap_map.0,
            chunk_ent,
            chunk_pos,
            bundle.dim_ref,
            &mut params.tile_gathering_paramset.tilemap_query,
            &mut locals.changed_structs,
            &mut tilemap_bundles,
            y_sort,
            &mut child_ofs_to_insert,
            to_persist,
        );
        locals.tile_runtime_info.insert(tile_ent, (bundle.templ_ref, bundle.tile_bundle.texture_index));
        i += 1;
    }
    //DEJAR CON IF NEW ASÍ TILES DE TILEMAP PUEDEN SER REPLICADAS
    cmd.try_insert_batch_if_new(take(&mut resources.collected_tiles.0));

    for tile_ent in spritetiles_to_remove_tmapbundle.drain(..) {
        cmd.entity(tile_ent).try_remove::<TileBundleNoTileFlip>();
    }
    cmd.try_insert_batch(child_ofs_to_insert);

    cmd.try_insert_batch(to_insert_replicated);

    cmd.try_insert_batch(tilemap_bundles);

    for mapkey in locals.changed_structs.iter() {
        let Some(shader_ref) = mapkey.shader_ref() else {
            continue;
        };
        let Ok(shader_ent) = resources.tile_shader_map.0.get_cloned(shader_ref.0) else {
            continue;
        };
        let Ok(shader) = params.shader_query.get(shader_ent) else {
            continue;
        };
        if !matches!(shader, TileShader::TerrBlend(_)) {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let neighbor_key = MapKey::new(
                    mapkey.dim_ref,
                    mapkey.chunk_pos + IVec2::new(dx, dy),
                    mapkey.ac_z,
                    mapkey.tile_size,
                    mapkey.shader_ref(),
                );
                let Some(neighbor_mapstruct) = resources.tmap_map.0.get(&neighbor_key) else {
                    continue;
                };
                cmd.entity(neighbor_mapstruct.tmap_ent).try_insert(NeedsTerrblRefresh);
            }
        }
    }

    let mut insert2tmaps = Vec::with_capacity(locals.changed_structs.len());
    let mut default_mats = Vec::with_capacity(locals.changed_structs.len());
    let mut terrbl_mats = Vec::with_capacity(locals.changed_structs.len());

    for mapkey in locals.changed_structs.drain() {
        let shader = if let Some(shader_ref) = mapkey.shader_ref() {
            resources
                .tile_shader_map
                .0
                .get_cloned(shader_ref.0)
                .ok()
                .and_then(|shader_ent| params.shader_query.get(shader_ent).ok())
                .map(|shader| shader.clone())
        } else {
            None
        };
        let Some(mapstruct) = resources.tmap_map.0.get_mut(&mapkey) else {
            continue;
        };
        let tmap_ent = mapstruct.tmap_ent;
        let texture_vec = mapstruct.take_texture();
        let storage = mapstruct.take_storage();
        let tmap_hash_id_map = mapstruct.take_hash_id_map();

        if let Some(shader) = shader {
            match shader {
                TileShader::TerrBlend(_) => {
                    if params.terrbl_handle_query.get_mut(tmap_ent).is_err() {
                        terrbl_mats.push((
                            tmap_ent,
                            MaterialTilemapHandle::from(
                                resources.texture_overlay_mat.add(TerrBlendMat::default())
                            ),
                        ));
                    }
                }
            };
        } else {
            default_mats.push((tmap_ent, MaterialTilemapHandle::<StandardTilemapMaterial>::default()));
        }
        insert2tmaps.push((tmap_ent, (tmap_hash_id_map, storage, texture_vec, )));
    }
    cmd.try_insert_batch(insert2tmaps);
    cmd.try_insert_batch(default_mats);
    cmd.try_insert_batch(terrbl_mats);
}

#[allow(clippy::too_many_arguments)]
fn process_tile_into_corresponding_tilemap(
    cmd: &mut Commands,
    tile_ent: Entity,
    tile_hid: HashId,
    size_in_tiles: SizeInTiles,
    tile_visible: &mut TileVisible,
    texture_index: &mut TileTextureIndex,
    tilemap_id: &mut TilemapId,
    position: TilePos,
    tile_z_index: AcZ,
    tile_handles: Option<&TileHashIdsHandles>,
    shader_ref: Option<&TileShaderRef>,
    img_size: U16Vec2,
    tmap_map: &mut HashMap<MapKey, MapStruct>,
    chunk: Entity,
    chunk_pos: ChunkPos,
    dim_ref: DimensionRef,
    tilemaps: &mut Query<(&mut TileStorage, &mut HashIdToTexIndex, &mut TilemapTexture)>,
    changed_structs: &mut HashSet<MapKey>,
    tilemap_bundles: &mut Vec<(Entity, (TilemapConfig, ChildOf, TilemapOf, DimensionRef, TileShaderRef))>,
    y_sort: bool,
    childofs: &mut Vec<(Entity, ChildOf)>,
    to_persist: bool,

) {
    if let Err(()) = size_in_tiles.tiles_per_chunk() {
        error!(
            target: TILEMAP_SYSTEM,
            "Tile {:?} has invalid size_in_tiles {:?} for chunk {:?}; tiles_per_chunk returned error",
            tile_ent,
            size_in_tiles.inner(),
            ChunkPos::CHUNK_SIZE,
        );
    }
    let tile_size = match tile_handles {
        Some(_) => img_size,
        None => {
            tile_visible.0 = false;
            error!(target: TILEMAP_SYSTEM, "Tile entity {:?} has no TileHashIdsHandles", tile_ent);
            return;
        }
    };
    let map_key = MapKey::new(dim_ref, chunk_pos, tile_z_index, tile_size, shader_ref.copied());

    if let Some(mapstruct) = tmap_map.get_mut(&map_key) {
        let tmap_ent = mapstruct.tmap_ent;
        if map_key.shader_ref().is_some() {
            cmd.entity(tmap_ent).try_insert(NeedsTerrblRefresh);
        }

        let (storage, tmap_hash_id_map, tmap_handles) =
        if let Ok((storage, tmap_hash_id_map, tmap_handles)) = tilemaps.get_mut(tmap_ent)
        {
            (storage.into_inner(), tmap_hash_id_map.into_inner(), tmap_handles.into_inner())
        } else {
            changed_structs.insert(map_key.clone());
            let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
            (storage, tmap_hash_id_map, tmap_handles)
        };
        let Vector(tmap_handles) = tmap_handles else {
            return;
        };

        if let Some(prev_tile_ent) = storage.get(&position) {
            cmd.entity(prev_tile_ent).try_despawn();
        }

        tilemap_id.0 = tmap_ent;//esto activa un draw
        storage.set(&position, tile_ent);

        let Some(tile_handles) = tile_handles else { return; };

        let mut first_matching_texture_index = None;

        for (handle_hid, handle) in tile_handles.iter() {
            let texture_index = tmap_handles
                .iter()
                .position(|x| *x == *handle)
                .map(|i| TileTextureIndex(i as u32))
                .unwrap_or_else(|| {
                    tmap_handles.push(handle.clone());
                    TileTextureIndex((tmap_handles.len() - 1) as u32)
                });
            tmap_hash_id_map.insert(tile_hid, handle_hid, texture_index);
            if first_matching_texture_index.is_none() {
                first_matching_texture_index = Some(texture_index);
                //don't do break
            }
        }
        texture_index.0 = first_matching_texture_index.unwrap_or_default().0;

        if ! to_persist {
            childofs.push((tile_ent, ChildOf(tmap_ent)));
        }

    } else {
        let mut tmap_hash_id_map = HashIdToTexIndex::with_capacity(0);
        changed_structs.insert(map_key.clone());

        let handles = if let Some(tile_handles) = tile_handles {
            tmap_hash_id_map.reserve(tile_handles.len());
            for (i, (handle_hid, _)) in tile_handles.iter().enumerate() {
                tmap_hash_id_map.insert(tile_hid, handle_hid, TileTextureIndex(i as u32));
            }
            tile_handles.handles().clone()
        } else {
            Vec::new()
        };
        let tmap_ent = cmd.spawn_empty().id();

        tilemap_bundles.push(
            (tmap_ent,
            (
                TilemapConfig::new(size_in_tiles, tile_size, chunk_pos, tile_z_index, y_sort),
                ChildOf(chunk),
                TilemapOf{chunk},
                dim_ref,
                shader_ref.copied().unwrap_or_default(),
            ))
        );
        if map_key.shader_ref().is_some() {
            cmd.entity(tmap_ent).try_insert(NeedsTerrblRefresh);
        }
        tilemap_id.0 = tmap_ent;

        let mut storage = TilemapConfig::new_storage(size_in_tiles);
        storage.set(&position, tile_ent);
        tmap_map.insert(map_key, MapStruct {
            tmap_ent,
            texture: TilemapTexture::Vector(handles),
            storage,
            tmap_hash_id_map,
            });
        if ! to_persist {
            childofs.push((tile_ent, ChildOf(tmap_ent)));
        }
    }
}
