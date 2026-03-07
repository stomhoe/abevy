use bevy::{
    asset::RenderAssetUsages,
    ecs::entity::EntityHashMap,
    ecs::system::SystemParam,
    math::U16Vec2,
    platform::collections::HashSet,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_ecs_tilemap::prelude::{*, TilemapTexture::Vector};
use bevy_replicon::prelude::{ClientState, Replicated};
use common::{TILEMAP_SYSTEM, common_components::HashId, common_resources::ImageSizeMap };
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::{EntityZero, EntityZeroRef, Persisted, };
use sprite_shared::{AcZ, YSortOrigin};
use ::tilemap_shared::*;
use crate::{chunking::chunking_resources::*, tile::{tile_bundles::*, tile_components::*, tile_shader::{tile_material::prelude::*, tile_shader_components::*} }, tilemap_bundles::*, tilemap_resources::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub struct MapKey {
    dim_ref: DimensionRef,
    chunk_pos: ChunkPos,
    ac_z: AcZ,
    tile_size: U16Vec2,
    shader_ref: Option<TileShaderRef>,
}
impl MapKey {
    pub fn new(
        dim_ref: DimensionRef,
        chunk_pos: ChunkPos,
        ac_z: AcZ,
        tile_size: U16Vec2,
        shader_ref: Option<TileShaderRef>,
    ) -> Self {
        Self { dim_ref, chunk_pos, ac_z, tile_size, shader_ref }
    }
    pub fn shader_ref(&self) -> Option<TileShaderRef> {self.shader_ref}
}

#[derive(Debug, Clone, )]
pub struct MapStruct{
    pub tmap_ent: Entity,
    pub texture: TilemapTexture,
    pub storage: TileStorage,
    pub tmap_hash_id_map: HashIdToTexIndex,
}
use std::{collections::HashMap, mem::take, };
impl MapStruct {
    pub fn take_texture(&mut self) -> TilemapTexture {take(&mut self.texture)}
    pub fn take_storage(&mut self) -> TileStorage {take(&mut self.storage)}
    pub fn take_hash_id_map(&mut self) -> HashIdToTexIndex {take(&mut self.tmap_hash_id_map)}
}


//ESTRATEGIA PERSISTENCIA: DEJAR TODAS LAS TILES MODIFICADAS EN WORLD (COMO ENTITIES), MARCARLAS CON ALGO.
//NO SE PUEDEN GUARDAR EN ESTRUCTURAS DE DATOS COMO HASHMAPS POR LA INFINIDAD DE COMBINACIONES POSIBLES DE COMPONENTES


#[derive(SystemParam)]
pub struct ProcessTilesPreParams<'w, 's> {
    pub collected_tiles: ResMut<'w, MassCollectedTiles>,


    pub tilemaps: Query<'w, 's, (
        &'static mut TilemapTexture,
        &'static mut TileStorage,
        &'static mut HashIdToTexIndex,
    ), ()>,
    pub image_size_map: Res<'w, ImageSizeMap>,

    pub texture_overlay_mat: ResMut<'w, Assets<TerrBlendMat>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub chunkrange: Res<'w, AaChunkRangeSettings>,

    pub min_dists_query: Query<'w, 's, &'static MinDistancesMap, common::AnyDisabling>,
    pub regpos_map: ResMut<'w, ImportantRegisteredPositions>,
    pub shader_query: Query<'w, 's, &'static TileShader, ()>,
    pub tile_query: Query<'w, 's, (&'static EntityZeroRef, &'static TileTextureIndex), (With<Tile>, Without<EntityZero>)>,
    pub ezero_terrbl_query: Query<'w, 's, Option<&'static TerrBlendParams>, With<EntityZero>>,

    pub loaded_chunks: Res<'w, LoadedChunks>,
    pub state: Res<'w, State<ClientState>>,
}

#[derive(Resource, Debug, Default, )]
pub struct TmapMap (
    pub HashMap<MapKey, MapStruct>
);

#[allow(unused_parens)]
pub fn on_tilemap_despawn(trig: On<Despawn, (TilemapTileSize, )>,
    query: Query<(&DimensionRef, &ChunkPos, &AcZ, &TilemapTileSize, &TileShaderRef)>,
    mut tmap_map: ResMut<TmapMap>,
) {
    let Ok((dimension_ref, chunk_pos, ac_z, tile_size, shader_ref)) = query.get(trig.entity)
    else {
        error_once!("Failed to get tilemap despawn query for entity {:?}", trig.entity);
        return;
    };
    let opt_shader = if shader_ref.is_placeholder() {
        None
    } else {
        Some(*shader_ref)
    };
    let tile_size_u16vec2 = U16Vec2::new(tile_size.x as u16, tile_size.y as u16);
    let map_key = MapKey::new(
        *dimension_ref,
        *chunk_pos,
        *ac_z,
        tile_size_u16vec2,
        opt_shader,
    );
    tmap_map.0.remove(&map_key);

}

#[allow(unused_parens, )]
pub fn process_tiles_pre(
    mut cmd: Commands,
    mut params: ProcessTilesPreParams,
    ezero_query: Query<(
        &TileStrId,
        &HashId,
        &SizeInTiles,
        Option<&MinDistancesMap>,
        Option<&KeepDistanceFrom>,
        Has<Persisted>,
        Option<&AcZ>,
        Option<&TileHashIdsHandles>,
        Option<&TileShaderRef>,
        Option<&TerrBlendParams>,
        Has<SpriteTile>,
        Option<&TileColor>,
        Has<YSortOrigin>,
    ), common::AnyDisabling>,
    mut spritetiles_map: ResMut<SpriteTilesAtGpos>,
    mut tmap_map: ResMut<TmapMap>,
    mut changed_structs: Local<HashSet<MapKey>>,
    mut tile_runtime_info: Local<EntityHashMap<(EntityZeroRef, TileTextureIndex)>>,
    mut terrbl_debug_budget: Local<u32>,
) {
    if *terrbl_debug_budget == 0 {
        *terrbl_debug_budget = 40;
    }
    tile_runtime_info.clear();
    let is_host = *params.state.get() == ClientState::Disconnected;

    if params.collected_tiles.0.is_empty() { return; }

    let reserved = params.chunkrange.approximate_number_of_chunks(0.06);
    let tiles_len = params.collected_tiles.0.len();
    changed_structs.reserve(reserved);

    let mut tilemap_bundles = Vec::with_capacity(200);//TODO HACER ALGO CON EL CHILDOF (CAMBIAR POR OTRO STRUCT?)

    let mut to_insert_replicated = Vec::with_capacity(tiles_len/100);
    //let mut spritetiles_to_insert_pos_and_dim_ref = Vec::with_capacity(tiles_len/20);
    let mut spritetiles_to_remove_bundle = Vec::with_capacity(tiles_len/20);

    let mut child_ofs_to_insert: Vec<(Entity, ChildOf)> = Vec::with_capacity(tiles_len);

    let mut i = 0;
    while i < params.collected_tiles.0.len() {
        // To avoid borrow checker issues, destructure the entry first, then operate on it
        let (tile_ent, bundle_ptr) = {
            unsafe{
                let (tile_ent, bundle) = &mut params.collected_tiles.0.get_mut(i).debug_unwrap_unchecked();
                (*tile_ent, bundle as *mut TileMassSpawnBundle)
            }
        };
        // SAFETY: We only have one mutable reference to this bundle at a time
        let bundle = unsafe { &mut *bundle_ptr };

        let query_result = ezero_query.get(bundle.ezero_ref.0);
        if query_result.is_err() {
            error_once!(target: TILEMAP_SYSTEM, "Original tile entity {} is despawned", bundle.ezero_ref.0);
            cmd.entity(tile_ent).try_despawn();
            params.collected_tiles.0.swap_remove(i);
            continue;
        }
        let (_tile_strid, hash_id, size_in_tiles, min_dists, keep_distance_from, to_persist, tile_z_index, tile_handles, shader_ref, terrbl_params, is_spritetile, color, y_sort) = query_result.unwrap();

        if !params.regpos_map.check_min_distances(
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
            params.min_dists_query,
        ) {
            cmd.entity(tile_ent).try_despawn();
            params.collected_tiles.0.swap_remove(i);
            info!(target: TILEMAP_SYSTEM, "Tile entity {:?} at gpos {:?} in dim {:?} despawned due to min distance check failure", tile_ent, bundle.gpos, bundle.dim_ref);
            continue;
        }

        if to_persist {
            if is_host {
                child_ofs_to_insert.push((tile_ent, ChildOf(bundle.dim_ref.0)));
                to_insert_replicated.push((tile_ent, Replicated));
                if is_spritetile{
                    spritetiles_map.insert(tile_ent, bundle.dim_ref, bundle.gpos, *size_in_tiles);
                    spritetiles_to_remove_bundle.push(tile_ent);
                    i += 1;
                    continue;
                }
            } else {
                cmd.entity(tile_ent).try_despawn();
                params.collected_tiles.0.swap_remove(i);
                continue;
            }
        }

        let Some(chunk_ent) = params.loaded_chunks.0.get(&(bundle.dim_ref, ChunkPos::from(bundle.gpos))).copied()
        else{
            cmd.entity(tile_ent).try_despawn();
            params.collected_tiles.0.swap_remove(i);
            continue;
        };

        if is_spritetile {
            spritetiles_to_remove_bundle.push(tile_ent);
            spritetiles_map.insert(tile_ent, bundle.dim_ref, bundle.gpos, *size_in_tiles);
            child_ofs_to_insert.push((tile_ent, ChildOf(chunk_ent)));
            i += 1;
            continue;
        }

        bundle.tile_bundle.color = color.cloned().unwrap_or_default();

        let chunk_pos = ChunkPos::from(bundle.gpos);

        let tile_img_size = if let Some(ref handles) = tile_handles {
            params.image_size_map.0.get(&handles.first_handle().id()).copied().unwrap_or(U16Vec2::ONE)
        } else {
            U16Vec2::ONE
        };
        let terrbl_img_size = if let Some(terrbl_params) = terrbl_params {
            if terrbl_params.texture_handle != Handle::default() {
                params
                    .image_size_map
                    .0
                    .get(&terrbl_params.texture_handle.id())
                    .copied()
                    .unwrap_or(U16Vec2::ZERO)
            } else {
                U16Vec2::ZERO
            }
        } else {
            U16Vec2::ZERO
        };

        process_tile_into_corresponding_tilemap(
            &mut cmd,
            tile_ent,
            *hash_id,
            *size_in_tiles,
            &mut bundle.tile_bundle.visible,
            &mut bundle.tile_bundle.texture_index,
            &mut bundle.tile_bundle.tilemap_id,
            bundle.tile_bundle.position,
            tile_z_index.cloned().unwrap_or_default(),
            tile_handles,
            shader_ref,
            tile_img_size,
            &mut tmap_map.0,
            chunk_ent,
            chunk_pos,
            bundle.dim_ref,
            &mut params.tilemaps,
            &mut changed_structs,
            &mut tilemap_bundles,
            y_sort,
            &mut child_ofs_to_insert,
            to_persist,
        );
        tile_runtime_info.insert(tile_ent, (bundle.ezero_ref, bundle.tile_bundle.texture_index));
        i += 1;
    }
    //DEJAR CON IF NEW ASÍ TILES DE TILEMAP PUEDEN SER REPLICADAS
    cmd.try_insert_batch_if_new(take(&mut params.collected_tiles.0));

    for tile_ent in spritetiles_to_remove_bundle.drain(..) {
        cmd.entity(tile_ent).try_remove::<TileBundleNoTileFlip>();
    }
    cmd.try_insert_batch(child_ofs_to_insert);

    cmd.try_insert_batch(to_insert_replicated);

    cmd.try_insert_batch(tilemap_bundles);

    let mut insert2tmaps = Vec::with_capacity(changed_structs.len());
    let mut default_mats = Vec::with_capacity(changed_structs.len());
    let mut terrbl_mats = Vec::with_capacity(changed_structs.len());

    for mapkey in changed_structs.drain() {
        let Some(mapstruct) = tmap_map.0.get_mut(&mapkey) else {
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
                        &mut params.images,
                        &params.tile_query,
                        &tile_runtime_info,
                        &params.ezero_terrbl_query,
                        &storage,
                        tmap_ent,
                        mapkey.tile_size,
                        &mut terrbl_debug_budget,
                    ) {
                        let material = MaterialTilemapHandle::from(params.texture_overlay_mat.add(material));
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
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut HashIdToTexIndex)>,
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

        let (tmap_handles, storage, tmap_hash_id_map) =
        if let Ok((tmap_handles, storage, tmap_hash_id_map)) = tilemaps.get_mut(tmap_ent)
        {
            (tmap_handles.into_inner(), storage.into_inner(), tmap_hash_id_map.into_inner())
        } else {
            changed_structs.insert(map_key.clone());
            let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
            (tmap_handles, storage, tmap_hash_id_map)
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

fn build_terrbl_material_for_map(
    images: &mut Assets<Image>,
    tile_query: &Query<(&EntityZeroRef, &TileTextureIndex), (With<Tile>, Without<EntityZero>)>,
    tile_runtime_info: &EntityHashMap<(EntityZeroRef, TileTextureIndex)>,
    ezero_terrbl_query: &Query<Option<&TerrBlendParams>, With<EntityZero>>,
    storage: &TileStorage,
    tmap_ent: Entity,
    tile_size_px: U16Vec2,
    terrbl_debug_budget: &mut u32,
) -> Option<TerrBlendMat> {
    const MAX_TERRBL_OVERLAYS: usize = 8;
    let width = storage.size.x;
    let height = storage.size.y;
    if width == 0 || height == 0 {
        error!(
            target: TILEMAP_SYSTEM,
            "terrbl debug: build skipped due to zero storage size for tmap {:?} (storage: {}x{}, tile_size: {:?}, )",
            tmap_ent,
            width,
            height,
            tile_size_px,
        );
        return None;
    }
    let px_count = (width as usize) * (height as usize);
    let mut tile_indices_data = vec![0_u8; px_count * 4];
    let mut tile_flags_data = vec![0_u8; px_count * 4];
    let mut tile_params_data = vec![0_u8; px_count * 16];
    let mut overlay_textures: Vec<Handle<Image>> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let tile_pos = TilePos { x, y };
            let Some(tile_ent) = storage.get(&tile_pos) else {
                continue;
            };
            let (ezero_ref, base_texture_index) = if let Some((ezero_ref, base_texture_index)) =
                tile_runtime_info.get(&tile_ent)
            {
                (*ezero_ref, *base_texture_index)
            } else if let Ok((ezero_ref, base_texture_index)) = tile_query.get(tile_ent) {
                (*ezero_ref, *base_texture_index)
            } else {
                if *terrbl_debug_budget > 0 {
                    *terrbl_debug_budget -= 1;
                    error!(
                        target: TILEMAP_SYSTEM,
                        "terrbl debug: missing tile runtime/query data at tile {:?} ent {:?}",
                        tile_pos,
                        tile_ent
                    );
                }
                continue;
            };
            let px_i = ((y as usize) * (width as usize) + (x as usize)) * 4;
            encode_u16(&mut tile_indices_data, px_i, base_texture_index.0 as u16);
            let Some(params) = ezero_terrbl_query.get(ezero_ref.0).ok().flatten() else {
                if *terrbl_debug_budget > 0 {
                    *terrbl_debug_budget -= 1;
                    error!(
                        target: TILEMAP_SYSTEM,
                        "terrbl debug: no TerrBlendParams on ezero {:?} tile {:?}",
                        ezero_ref.0,
                        tile_pos
                    );
                }
                continue;
            };

            let mut flags = 0_u8;
            flags |= 1 << 0; // has params
            if params.blend_enabled {
                flags |= 1 << 1;
            }

            let mut overlay_idx = 0_u16;
            if let Some(path_holder) = params.texture_path.as_ref() {
                let overlay_handle = params.texture_handle.clone();
                if overlay_handle == Handle::default() {
                    if *terrbl_debug_budget > 0 {
                        *terrbl_debug_budget -= 1;
                        error!(
                            target: TILEMAP_SYSTEM,
                            "terrbl debug: missing texture handle for '{}' at tile {:?}",
                            path_holder.path(),
                            tile_pos
                        );
                    }
                    continue;
                }

                flags |= 1 << 2;
                overlay_idx = match overlay_textures.iter().position(|h| *h == overlay_handle) {
                    Some(i) => i as u16,
                    None => {
                        if overlay_textures.len() >= MAX_TERRBL_OVERLAYS {
                            if *terrbl_debug_budget > 0 {
                                *terrbl_debug_budget -= 1;
                                error!(
                                    target: TILEMAP_SYSTEM,
                                    "terrbl debug: too many overlay textures in one terrbl map (max {}), skipping '{}' at tile {:?}",
                                    MAX_TERRBL_OVERLAYS,
                                    path_holder.path(),
                                    tile_pos
                                );
                            }
                            flags &= !(1 << 2);
                            0
                        } else {
                            overlay_textures.push(overlay_handle);
                            (overlay_textures.len() - 1) as u16
                        }
                    }
                };
            }

            encode_u16(&mut tile_indices_data, px_i + 2, overlay_idx);
            tile_flags_data[px_i] = flags;
            tile_flags_data[px_i + 3] = 255;
            if *terrbl_debug_budget > 0 {
                *terrbl_debug_budget -= 1;
                trace!(
                    target: TILEMAP_SYSTEM,
                    "terrbl debug: tile {:?} base_idx {} overlay_idx {} flags {:08b} has_params {} blend_enabled {} tex '{}'",
                    tile_pos,
                    base_texture_index.0,
                    overlay_idx,
                    flags,
                    true,
                    params.blend_enabled,
                    params.texture_path.as_ref().map(ToString::to_string).unwrap_or_default()
                );
            }

            encode_f32x4(
                &mut tile_params_data,
                px_i * 4,
                [params.scale, params.speed, params.wavy_strength, params.time_offset],
            );
        }
    }

    let tile_indices_map = images.add(create_image_u8(width, height, tile_indices_data));
    let tile_flags_map = images.add(create_image_u8(width, height, tile_flags_data));
    let tile_params_map = images.add(create_image_f32(width, height, tile_params_data));

    let mut mat = TerrBlendMat {
        tile_indices_map,
        tile_flags_map,
        tile_params_map,
        map_size_tiles: Vec2::new(width as f32, height as f32),
        time: 0.0,
        ..Default::default()
    };
    if let Some(h) = overlay_textures.first() {
        mat.overlay_tex_0 = h.clone();
    }
    if let Some(h) = overlay_textures.get(1) {
        mat.overlay_tex_1 = h.clone();
    }
    if let Some(h) = overlay_textures.get(2) {
        mat.overlay_tex_2 = h.clone();
    }
    if let Some(h) = overlay_textures.get(3) {
        mat.overlay_tex_3 = h.clone();
    }
    if let Some(h) = overlay_textures.get(4) {
        mat.overlay_tex_4 = h.clone();
    }
    if let Some(h) = overlay_textures.get(5) {
        mat.overlay_tex_5 = h.clone();
    }
    if let Some(h) = overlay_textures.get(6) {
        mat.overlay_tex_6 = h.clone();
    }
    if let Some(h) = overlay_textures.get(7) {
        mat.overlay_tex_7 = h.clone();
    }
    Some(mat)
}

fn encode_u16(out: &mut [u8], index: usize, value: u16) {
    out[index] = (value & 0x00FF) as u8;
    out[index + 1] = ((value >> 8) & 0x00FF) as u8;
}

fn encode_f32x4(out: &mut [u8], index: usize, values: [f32; 4]) {
    let mut byte_i = index;
    for value in values {
        let bytes = value.to_ne_bytes();
        out[byte_i] = bytes[0];
        out[byte_i + 1] = bytes[1];
        out[byte_i + 2] = bytes[2];
        out[byte_i + 3] = bytes[3];
        byte_i += 4;
    }
}

fn create_image_u8(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    )
}

fn create_image_f32(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    )
}
