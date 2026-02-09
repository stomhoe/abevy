use bevy::{ecs::system::SystemParam, math::U16Vec2, platform::collections::HashSet, prelude::*, };
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::{ClientState, Replicated};
use common::{common_components::{HashId}, common_resources::ImageSizeMap, };
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::{Persisted, };
use sprite_shared::{AcZ, YSortOrigin};
use ::tilemap_shared::*;


use crate::{chunking::{chunking_resources::*}, tile::{tile_components::*, tile_shader::{tile_material::prelude::*, tile_shader_components::*} }, tilemap_components::*, tilemap_resources::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct MapKey {
    dim_ref: DimensionRef,
    chunk_pos: ChunkPos,
    ac_z: AcZ,
    oplist_size: OplistSize,
    tile_size: U16Vec2,
    shader_ref: Option<TileShaderRef>,
}
impl MapKey {
    pub fn new(
        dim_ref: DimensionRef,
        chunk_pos: ChunkPos,
        ac_z: AcZ,
        oplist_size: OplistSize,
        tile_size: U16Vec2,
        shader_ref: Option<TileShaderRef>,
    ) -> Self {
        Self { dim_ref, chunk_pos, ac_z, oplist_size, tile_size, shader_ref }
    }
    pub fn shader_ref(&self) -> Option<TileShaderRef> {self.shader_ref}
}

#[derive(Debug, Clone, Reflect)]
/// NO BORRAR ESTE STRUCT, DENTRO DE UNA INSTANCIA DE EJECUCIÓN DE FUNCIÓN LAS QUERIES NO SE ACTUALIZAN HASTA Q SE SALE DE LA FUNCIÓN. HACE FALTA ESTO
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


use bevy_ecs_tilemap::prelude::TilemapTexture::Vector;

#[derive(SystemParam)]
pub struct ProcessTilesPreParams<'w, 's> {
    pub collected_tiles: ResMut<'w, MassCollectedTiles>,


    pub tilemaps: Query<'w, 's, (
        &'static mut TilemapTexture,
        &'static mut TileStorage,
        &'static mut HashIdToTexIndex,
    ), ()>,
    pub image_size_map: Res<'w, ImageSizeMap>,

    pub texture_overlay_mat: ResMut<'w, Assets<MonoRepeatTextureOverlayMat>>,
    pub voronoi_mat: ResMut<'w, Assets<VoronoiTextureOverlayMat>>,
    pub wavy_mat: ResMut<'w, Assets<WavyMat>>,
    pub rocky_mat: ResMut<'w, Assets<RockyTerrainMat>>,
    pub chunkrange: Res<'w, AaChunkRangeSettings>,

    pub min_dists_query: Query<'w, 's, (&'static MinDistancesMap), common::AnyDisabling>,
    pub regpos_map: ResMut<'w, ImportantRegisteredPositions>,
    pub shader_query: Query<'w, 's, (&'static TileShader), ()>,

    pub loaded_chunks: Res<'w, LoadedChunks>,
    pub state: Res<'w, State<ClientState>>,
}

#[derive(Resource, Debug, Reflect, Default, )]
#[reflect(Resource, Default)]
pub struct TmapMap (
    pub HashMap<MapKey, MapStruct>
);

#[allow(unused_parens)]
pub fn on_tilemap_despawn(trig: On<Despawn, (DimensionRef, ChunkPos, AcZ, OplistSize, TilemapTileSize, TileShaderRef)>,
    query: Query<(&DimensionRef, &ChunkPos, &AcZ, &OplistSize, &TilemapTileSize, &TileShaderRef)>,
    mut tmap_map: ResMut<TmapMap>,
) {
    let Ok((dimension_ref, chunk_pos, ac_z, oplist_size, tile_size, shader_ref)) = query.get(trig.entity) else {
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
        *oplist_size,
        tile_size_u16vec2,
        opt_shader,
    );
    tmap_map.0.remove(&map_key);
}


#[allow(unused_parens, )]//TODO: USAR try_insert_bundle
pub fn process_tiles_pre(
    mut cmd: Commands,
    mut params: ProcessTilesPreParams,
    mut tmap_map: ResMut<TmapMap>,
    ezero_query: Query<(
        &TileStrId,
        &HashId,
        Option<&MinDistancesMap>,
        Option<&KeepDistanceFrom>,
        Has<Persisted>,
        Option<&AcZ>,
        Option<&TileHashIdsHandles>,
        Option<&TileShaderRef>,
        Has<SpriteTile>,
        Option<&TileColor>,
        Has<YSortOrigin>,
    ), common::AnyDisabling>,
    mut spritetiles_map: ResMut<SpriteTilesAtGpos>,

)
{
    let is_host = *params.state.get() == ClientState::Disconnected;

    if params.collected_tiles.0.is_empty() { return; }

    let reserved = params.chunkrange.approximate_number_of_chunks(0.06);
    let tiles_len = params.collected_tiles.0.len();

    let mut changed_structs: HashSet<MapKey> = HashSet::with_capacity(reserved);


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
            error!(target: "tilemap_systems", "Original tile entity {} is despawned", bundle.ezero_ref.0);
            cmd.entity(tile_ent).try_despawn();
            params.collected_tiles.0.swap_remove(i);
            continue;
        }
        let (tile_strid, hash_id, min_dists, keep_distance_from, to_persist, tile_z_index, tile_handles, shader_ref, is_spritetile, color, y_sort) = query_result.unwrap();

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
            info!(target: "tilemap_systems", "Tile entity {:?} at gpos {:?} in dim {:?} despawned due to min distance check failure", tile_ent, bundle.gpos, bundle.dim_ref);
            continue;
        }

        if to_persist {
            if is_host {
                to_insert_replicated.push((tile_ent, Replicated));
            } else {
                cmd.entity(tile_ent).try_despawn();
                params.collected_tiles.0.swap_remove(i);
                continue;
            }
        }

        if is_spritetile {
            spritetiles_to_remove_bundle.push(tile_ent);
            spritetiles_map.insert(tile_ent, bundle.dim_ref, bundle.gpos);
            i += 1;
            continue;
        }

        bundle.tile_bundle.color = color.cloned().unwrap_or_default();

        let chunk_opt = params.loaded_chunks.0.get(&(bundle.dim_ref, ChunkPos::from(bundle.gpos))).copied();
        if chunk_opt.is_none() {
            cmd.entity(tile_ent).try_despawn();
            params.collected_tiles.0.swap_remove(i);
            continue;
        }
        let chunk = chunk_opt.unwrap();
        let chunk_pos = ChunkPos::from(bundle.gpos);

        let tile_size = if let Some(ref handles) = tile_handles {
            params.image_size_map
                .0
                .get(&handles.first_handle().id())
                .copied()
                .unwrap_or(U16Vec2::ONE)
        } else {
            U16Vec2::ONE
        };

        func_process_tile_into_tilemaps(
            &mut cmd,
            tile_ent,
            *hash_id,
            &mut bundle.tile_bundle.visible,
            &mut bundle.tile_bundle.texture_index,
            &mut bundle.tile_bundle.tilemap_id,
            bundle.oplist_size,
            bundle.tile_bundle.position,
            tile_z_index.cloned().unwrap_or_default(),
            tile_handles,
            shader_ref,
            tile_size,
            &mut tmap_map.0,
            chunk,
            chunk_pos,
            bundle.dim_ref,
            &mut params.tilemaps,
            &mut changed_structs,
            &mut tilemap_bundles,
            y_sort,
            &mut child_ofs_to_insert,
        );
        i += 1;
    }
    //DEJAR CON IF NEW ASÍ TILES DE TILEMAP PUEDEN SER REPLICADAS
    cmd.try_insert_batch_if_new(take(&mut params.collected_tiles.0));

    for tile_ent in spritetiles_to_remove_bundle.drain(..) {
        cmd.entity(tile_ent).try_remove::<TileBundle>();
    }
    cmd.try_insert_batch(child_ofs_to_insert);

    cmd.try_insert_batch(to_insert_replicated);

    cmd.try_insert_batch(tilemap_bundles);

    let mut insert2tmaps = Vec::with_capacity(changed_structs.len());
    let mut default_mats = Vec::with_capacity(changed_structs.len());
    let mut wavy_mats = Vec::with_capacity(changed_structs.len());
    let mut texture_overlay_mats = Vec::with_capacity(changed_structs.len());
    let mut rocky_mats = Vec::with_capacity(changed_structs.len());

    for mapkey in changed_structs.iter() {
        //trace!(target: "tilemap_systems", "Changed tilemap {:?} in chunk {:?}", mapkey, mapkey.chunk_pos());
        let Some(mapstruct) = tmap_map.0.get_mut(mapkey) else {
            continue;
        };
        let tmap_ent = mapstruct.tmap_ent;

        let (texture_vec, storage, tmap_hash_id_map) = (
            mapstruct.take_texture(),
            mapstruct.take_storage(),
            mapstruct.take_hash_id_map(),
        );
        insert2tmaps.push((tmap_ent, (tmap_hash_id_map, storage, texture_vec, )));

        let shader = if let Some(shader_ref) = mapkey.shader_ref() {
            params.shader_query.get(shader_ref.0).ok().map(|(shader)| shader.clone())
        } else {
            None
        };
        if let Some(shader) = shader {
            //trace!(target: "tilemap_systems", "Inserting tmapshader {:?} for tilemap entity {:?}", shader, tmap_ent);
            match shader {
                TileShader::TexRepeat(handle) => {
                    let material = MaterialTilemapHandle::from(params.texture_overlay_mat.add(handle));
                    texture_overlay_mats.push((tmap_ent, material));
                }
                TileShader::Voronoi(handle) => {
                    let material = MaterialTilemapHandle::from(params.voronoi_mat.add(handle));
                    cmd.entity(tmap_ent).try_insert(material);
                }
                TileShader::Wavy(handle) => {
                    let material = MaterialTilemapHandle::from(params.wavy_mat.add(handle));
                    wavy_mats.push((tmap_ent, material.clone()));
                }
                TileShader::TwoTexRepeat(_handle) => todo!(),
                TileShader::RockyTerrain(rocky_terrain_mat) => {
                    let material = MaterialTilemapHandle::from(params.rocky_mat.add(rocky_terrain_mat));
                    rocky_mats.push((tmap_ent, material.clone()));
                }
            };

        } else {
            default_mats.push((tmap_ent, MaterialTilemapHandle::<StandardTilemapMaterial>::default()));
        }
    }
    cmd.try_insert_batch(default_mats);
    cmd.try_insert_batch(texture_overlay_mats);
    cmd.try_insert_batch(wavy_mats);
    cmd.try_insert_batch(insert2tmaps);
    cmd.try_insert_batch(rocky_mats);

}

#[allow(clippy::too_many_arguments)]
fn func_process_tile_into_tilemaps(
    cmd: &mut Commands,
    tile_ent: Entity,
    tile_hid: HashId,
    tile_visible: &mut TileVisible,
    texture_index: &mut TileTextureIndex,
    tilemap_id: &mut TilemapId,
    oplist_size: OplistSize,
    position: TilePos,
    tile_z_index: AcZ,
    tile_handles: Option<&TileHashIdsHandles>,
    shader_ref: Option<&TileShaderRef>,
    tile_size: U16Vec2,
    tmap_map: &mut HashMap<MapKey, MapStruct>,
    chunk: Entity,
    chunk_pos: ChunkPos,
    dim_ref: DimensionRef,
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut HashIdToTexIndex)>,
    changed_structs: &mut HashSet<MapKey>,
    tilemap_bundles: &mut Vec<(Entity, (TilemapConfig, ChildOf, TilemapOf, DimensionRef, TileShaderRef))>,
    y_sort: bool,
    childofs: &mut Vec<(Entity, ChildOf)>,
) {
    let tile_size = match tile_handles {
        Some(_) => tile_size,
        None => {
            tile_visible.0 = false;
            error!(target: "tilemap_systems", "Tile entity {:?} has no TileHashIdsHandles", tile_ent);
            return;
        }
    };
    let map_key = MapKey::new(dim_ref, chunk_pos, tile_z_index, oplist_size, tile_size, shader_ref.copied());

    if let Some(mapstruct) = tmap_map.get_mut(&map_key) {
        let tmap_ent = mapstruct.tmap_ent;

        let (tmap_handles, storage, tmap_hash_id_map) =
        if let Ok((tmap_handles, storage, tmap_hash_id_map)) = tilemaps.get_mut(tmap_ent)
        {
            //no insertion into changed structs needed since tilemap's components are getting edited directly
            (tmap_handles.into_inner(), storage.into_inner(), tmap_hash_id_map.into_inner())
        } else {
            changed_structs.insert(map_key.clone());
            let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
            (tmap_handles, storage, tmap_hash_id_map)
        };
        let Vector(tmap_handles) = tmap_handles else {
            return;
        };

        if storage.get(&position).is_some() {
            //no overwriting, tile must be despawned first
            return;
        }

        tilemap_id.0 = tmap_ent;//esto activa un draw
        storage.set(&position, tile_ent);

        let Some(tile_handles) = tile_handles else { return; };

        let mut first_texture_index = None;

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
            if first_texture_index.is_none() {
                first_texture_index = Some(texture_index);
            }
        }
        texture_index.0 = first_texture_index.unwrap_or_default().0;

        childofs.push((tile_ent, ChildOf(tmap_ent)));

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
                TilemapConfig::new(oplist_size, tile_size, chunk_pos, tile_z_index, y_sort),
                ChildOf(chunk),
                TilemapOf::new(chunk),
                dim_ref,
                shader_ref.copied().unwrap_or_default(),
            ))
        );

        tilemap_id.0 = tmap_ent;


        let mut storage = TilemapConfig::new_storage(oplist_size);
        storage.set(&position, tile_ent);
        tmap_map.insert(map_key, MapStruct {
            tmap_ent,
            texture: TilemapTexture::Vector(handles),
            storage,
            tmap_hash_id_map,
            });
        childofs.push((tile_ent, ChildOf(tmap_ent)));
    }
}
