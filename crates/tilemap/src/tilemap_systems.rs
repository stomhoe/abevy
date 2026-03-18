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
use common::common_tag_components::TagSet;
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::{EntityZero, EntityZeroRef, Persisted};
use sprite_shared::prelude::{AcZ, YSortOrigin};
use ::tilemap_shared::*;
use crate::{
    chunking::chunking_resources::*,
    tile::{
        tile_bundles::*,
        tile_components::*,
        tile_delete_others_helpers::TileDeleteOthersParamSet,
        tile_despawn_systems::*,
        tile_shader::tile_material::prelude::*,
        tile_shader::tile_shader_components::*,
    },
    tilemap_bundles::*,
    tilemap_resources::*,
    tilemap_structs::*,
    tilemap_terrbl_systems::build_terrbl_material_for_map,
};
use std::{collections::HashMap, mem::take};

#[derive(SystemParam)]
pub struct SystemLocals<'s> {
    pub changed_structs: Local<'s, HashSet<MapKey>>,
    pub tile_runtime_info: Local<'s, EntityHashMap<(EntityZeroRef, TileTextureIndex)>>,
    pub terrbl_debug_budget: Local<'s, u32>,
    pub safe_despawns: Local<'s, Vec<SafeDespawn>>,
    pub checked_ents: Local<'s, HashSet<Entity>>,
}

#[derive(SystemParam)]
pub struct SystemResources<'w> {
    pub collected_tiles: ResMut<'w, MassCollectedTiles>,
    pub image_size_map: Res<'w, ImageSizeMap>,
    pub texture_overlay_mat: ResMut<'w, Assets<TerrBlendMat>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub chunkrange: Res<'w, AaChunkRangeSettings>,
    pub regpos_map: ResMut<'w, ImportantRegisteredPositions>,
    pub loaded_chunks: Res<'w, LoadedChunks>,
    pub state: Res<'w, State<ClientState>>,
    pub tmap_map: ResMut<'w, TmapMap>,
}

#[derive(SystemParam)]
pub struct ComponentsQueries<'w, 's> {
    pub min_dists_query: Query<'w, 's, &'static MinDistancesMap, common::AnyDisabling>,
    pub hash_id_query: Query<'w, 's, &'static HashId, common::AnyDisabling>,
    pub size_query: Query<'w, 's, &'static SizeInTiles, common::AnyDisabling>,
    pub keep_distance_query: Query<'w, 's, &'static KeepDistanceFrom, common::AnyDisabling>,
    pub persisted_query: Query<'w, 's, (), (With<Persisted>, common::AnyDisabling)>,
    pub z_query: Query<'w, 's, &'static AcZ, common::AnyDisabling>,
    pub handles_query: Query<'w, 's, &'static TileHashIdsHandles, common::AnyDisabling>,
    pub shader_ref_query: Query<'w, 's, &'static TileShaderRef, common::AnyDisabling>,
    pub sprite_tile_query: Query<'w, 's, (), (With<SpriteTile>, common::AnyDisabling)>,
    pub color_query: Query<'w, 's, &'static TileColor, common::AnyDisabling>,
    pub y_sort_query: Query<'w, 's, (), (With<YSortOrigin>, common::AnyDisabling)>,
    pub tile_ezero_ref_query: Query<'w, 's, &'static EntityZeroRef>,
    pub tile_texture_index_query: Query<'w, 's, &'static TileTextureIndex>,
    pub tag_set_query: Query<'w, 's, &'static TagSet, common::AnyDisabling>,
    pub delete_others_query: Query<'w, 's, &'static DeleteOtherTilesInSamePos>,
    pub gpos_query: Query<'w, 's, &'static GlobalTilePos>,
    pub delete_others_paramset: TileDeleteOthersParamSet<'w, 's>,
}

#[derive(SystemParam)]
pub struct ProcessTilesPreParams<'w, 's> {
    pub resources: ParamSet<'w, 's, (SystemResources<'w>,)>,
    pub tile_gathering_paramset: TileGatheringParamSet<'w, 's>,
    pub tile_components_queries: ParamSet<'w, 's, (ComponentsQueries<'w, 's>,)>,
    pub shader_query: Query<'w, 's, &'static TileShader, ()>,
    pub terrbl_query: Query<'w, 's, &'static TerrBlendParams>,
    pub locals: SystemLocals<'s>,
}

#[allow(unused_parens, )]
pub fn process_tiles_pre(
    mut cmd: Commands,
    mut params: ProcessTilesPreParams,
    mut safe_despawn_writer: MessageWriter<SafeDespawn>,
) {
    let resources = &mut params.resources.p0();
    let tile_components = &mut params.tile_components_queries.p0();
    let locals = &mut params.locals;

    if *locals.terrbl_debug_budget == 0 {
        *locals.terrbl_debug_budget = 40;
    }
    locals.tile_runtime_info.clear();
    let is_host = *resources.state.get() == ClientState::Disconnected;

    if resources.collected_tiles.0.is_empty() { return; }

    let reserved = resources.chunkrange.approximate_number_of_chunks(0.06);
    let tiles_len = resources.collected_tiles.0.len();
    locals.changed_structs.reserve(reserved);

    let mut tilemap_bundles = Vec::with_capacity(200);

    let mut to_insert_replicated = Vec::with_capacity(tiles_len/100);
    let mut spritetiles_to_remove_bundle = Vec::with_capacity(tiles_len/20);

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

        let Ok(&ez_hash_id) = tile_components.hash_id_query.get(bundle.ezero_ref.0) else {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} is despawned", bundle.ezero_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };
        let Ok(&size_in_tiles) = tile_components.size_query.get(bundle.ezero_ref.0) else {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} missing SizeInTiles", bundle.ezero_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };
        let Ok(&tile_z_index) = tile_components.z_query.get(bundle.ezero_ref.0) else {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} missing AcZ query access", bundle.ezero_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };
        let tile_handles = tile_components.handles_query.get(bundle.ezero_ref.0).ok().cloned();
        let shader_ref = tile_components.shader_ref_query.get(bundle.ezero_ref.0).ok().copied();
        let color = tile_components.color_query.get(bundle.ezero_ref.0).cloned();
        let is_spritetile = tile_components.sprite_tile_query.get(bundle.ezero_ref.0).is_ok();
        let y_sort = tile_components.y_sort_query.get(bundle.ezero_ref.0).is_ok();
        let to_persist = tile_components.persisted_query.get(bundle.ezero_ref.0).is_ok();
        let min_dists = tile_components.min_dists_query.get(bundle.ezero_ref.0).ok();
        let keep_distance_from = tile_components.keep_distance_query.get(bundle.ezero_ref.0).ok();
        let Ok(&_dim_hash) = tile_components.hash_id_query.get(bundle.dim_ref.0) else {
            error_once!(target: TILEMAP_SYSTEM, "Dimension entity {} is despawned", bundle.dim_ref.0);
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };

        if !resources.regpos_map.check_min_distances(
            &mut cmd,
            is_host,
            (
                tile_ent,
                bundle.ezero_ref,
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

        {
            process_tile_despawns_from_ezero(
                &mut tile_components.delete_others_paramset,
                &resources.regpos_map,
                &params.tile_gathering_paramset,
                tile_ent,
                bundle,
            );
            if tile_is_pending_despawn(&locals.safe_despawns, tile_ent) {
                cmd.entity(tile_ent).try_despawn();
                resources.collected_tiles.0.swap_remove(i);
                continue;
            }
        }

        //cmd.entity(tile_ent).try_insert_if_new(Signature::from((ez_hash_id, dim_hash, bundle.gpos)));

        if to_persist {
            if is_host {
                child_ofs_to_insert.push((tile_ent, ChildOf(bundle.dim_ref.0)));
                to_insert_replicated.push((tile_ent, Replicated));
                if is_spritetile{
                    params.tile_gathering_paramset.insert_spritetile(tile_ent, bundle.dim_ref, bundle.gpos, size_in_tiles);
                    spritetiles_to_remove_bundle.push(tile_ent);
                    i += 1;
                    continue;
                }
            } else {
                cmd.entity(tile_ent).try_despawn();
                resources.collected_tiles.0.swap_remove(i);
                continue;
            }
        }

        let Some(chunk_ent) = resources.loaded_chunks.0.get(&(bundle.dim_ref, ChunkPos::from(bundle.gpos))).copied()
        else{
            cmd.entity(tile_ent).try_despawn();
            resources.collected_tiles.0.swap_remove(i);
            continue;
        };

        if is_spritetile {
            spritetiles_to_remove_bundle.push(tile_ent);
            params.tile_gathering_paramset.insert_spritetile(tile_ent, bundle.dim_ref, bundle.gpos, size_in_tiles);
            child_ofs_to_insert.push((tile_ent, ChildOf(chunk_ent)));
            i += 1;
            continue;
        }

        bundle.tile_bundle.color = color.unwrap_or_default();

        let chunk_pos = ChunkPos::from(bundle.gpos);

        let tile_img_size = tile_handles
            .as_ref()
            .and_then(|handles| resources.image_size_map.0.get(&handles.first_handle().id()).copied())
            .unwrap_or(U16Vec2::ONE);

        process_tile_into_corresponding_tilemap(
            &mut cmd,
            tile_ent,
            ez_hash_id,
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
        locals.tile_runtime_info.insert(tile_ent, (bundle.ezero_ref, bundle.tile_bundle.texture_index));
        i += 1;
    }
    safe_despawn_writer.write_batch(locals.safe_despawns.drain(..));
    //DEJAR CON IF NEW ASÍ TILES DE TILEMAP PUEDEN SER REPLICADAS
    cmd.try_insert_batch_if_new(take(&mut resources.collected_tiles.0));

    for tile_ent in spritetiles_to_remove_bundle.drain(..) {
        cmd.entity(tile_ent).try_remove::<TileBundleNoTileFlip>();
    }
    cmd.try_insert_batch(child_ofs_to_insert);

    cmd.try_insert_batch(to_insert_replicated);

    cmd.try_insert_batch(tilemap_bundles);

    let mut insert2tmaps = Vec::with_capacity(locals.changed_structs.len());
    let mut default_mats = Vec::with_capacity(locals.changed_structs.len());
    let mut terrbl_mats = Vec::with_capacity(locals.changed_structs.len());

    for mapkey in locals.changed_structs.drain() {
        let Some(mapstruct) = resources.tmap_map.0.get_mut(&mapkey) else {
            continue;
        };
        let tmap_ent = mapstruct.tmap_ent;

        let shader = if let Some(shader_ref) = mapkey.shader_ref() {
            params.shader_query.get(shader_ref.0).ok().map(|(shader)| shader.clone())
        } else {
            None
        };
        let texture_vec = mapstruct.take_texture();
        let storage = mapstruct.take_storage();
        let tmap_hash_id_map = mapstruct.take_hash_id_map();

        if let Some(shader) = shader {
            match shader {
                TileShader::TerrBlend(_) => {
                    if let Some(material) = build_terrbl_material_for_map(
                        &mut resources.images,
                        &tile_components.tile_ezero_ref_query,
                        &tile_components.tile_texture_index_query,
                        &locals.tile_runtime_info,
                        &params.terrbl_query,
                        &storage,
                        tmap_ent,
                        mapkey.tile_size,
                        &mut locals.terrbl_debug_budget,
                    ) {
                        let material = MaterialTilemapHandle::from(resources.texture_overlay_mat.add(material));
                        terrbl_mats.push((tmap_ent, material));
                    } else {
                        error!(
                            target: TILEMAP_SYSTEM,
                            "Failed to build terrbl material for map: tmap {:?}, dim {:?}, chunk {:?}, z {:?}, tile_size {:?}, storage {}x{}",
                            tmap_ent,
                            mapkey.dim_ref,
                            mapkey.chunk_pos,
                            mapkey.ac_z,
                            mapkey.tile_size,
                            storage.size.x,
                            storage.size.y
                        );
                        default_mats.push((tmap_ent, MaterialTilemapHandle::<StandardTilemapMaterial>::default()));
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
            cmd.entity(tmap_ent).insert(NeedsTerrblRefresh);
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
                TilemapOf::new(chunk),
                dim_ref,
                shader_ref.copied().unwrap_or_default(),
            ))
        );
        if map_key.shader_ref().is_some() {
            cmd.entity(tmap_ent).insert(NeedsTerrblRefresh);
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
