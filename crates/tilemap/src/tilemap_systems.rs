use bevy::{asset::AssetId, ecs::{query, system::SystemParam}, math::U16Vec2, platform::collections::HashSet, prelude::*, render::sync_world::SyncToRenderWorld, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::{ClientState, Replicated};
use common::{common_components::{AnyDisabling}, common_resources::ImageSizeMap, };
use game_common::game_common_components::{Persisted, ReparentingRetries};
use sprite_shared::AcZ;
use ::tilemap_shared::*;
use dimension_shared::DimensionRef;

use crate::{chunking_components::Chunk, chunking_resources::*, terrain_gen::terrgen_resources::*, tile::{tile_components::*, tile_shader::{tile_material::prelude::*, tile_shader_components::*}, }, tilemap_components::*, tilemap_resources::*};

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
    pub fn dim_ref(&self) -> DimensionRef { self.dim_ref }
    pub fn chunk_pos(&self) -> ChunkPos { self.chunk_pos }
    pub fn ac_z(&self) -> AcZ {self.ac_z}
    pub fn oplist_size(&self) -> OplistSize {self.oplist_size}
    pub fn tile_size(&self) -> U16Vec2 {self.tile_size}
    pub fn shader_ref(&self) -> Option<TileShaderRef> {self.shader_ref}
}

#[derive(Debug, Clone, Reflect)]
/// NO BORRAR ESTE STRUCT, DENTRO DE UNA INSTANCIA DE EJECUCIÓN DE FUNCIÓN LAS QUERIES NO SE ACTUALIZAN HASTA Q SE SALE DE LA FUNCIÓN. HACE FALTA ESTO
pub struct MapStruct{
    pub tmap_ent: Entity,
    pub texture: TilemapTexture,
    pub storage: TileStorage,
    pub tmap_hash_id_map: TmapHashIdtoTextureIndex,
}
use std::{collections::HashMap, mem::take};
impl MapStruct {
    pub fn take_texture(&mut self) -> TilemapTexture {take(&mut self.texture)}
    pub fn take_storage(&mut self) -> TileStorage {take(&mut self.storage)}
    pub fn take_hash_id_map(&mut self) -> TmapHashIdtoTextureIndex {take(&mut self.tmap_hash_id_map)}
}


//ESTRATEGIA PERSISTENCIA: DEJAR TODAS LAS TILES MODIFICADAS EN WORLD (COMO ENTITIES), MARCARLAS CON ALGO. 
//NO SE PUEDEN GUARDAR EN ESTRUCTURAS DE DATOS COMO HASHMAPS POR LA INFINIDAD DE COMBINACIONES POSIBLES DE COMPONENTES


use bevy_ecs_tilemap::prelude::TilemapTexture::Vector;

#[derive(SystemParam)]
pub struct ProcessTilesPreParams<'w, 's> {
    pub collected_tiles: ResMut<'w, MassCollectedTiles>,
    pub limbo_tiles: ResMut<'w, TilemapLimboTiles>,
    pub tilemap_tasks: ResMut<'w, TilemapAsyncTasks>,

    pub ezero_query: Query<'w, 's, (
        &'static TileStrId,
        Option<&'static MinDistancesMap>,
        Option<&'static KeepDistanceFrom>,
        Has<Persisted>,
        Option<&'static AcZ>,
        Option<&'static TileHidsHandles>,
        Option<&'static TileShaderRef>,
        Option<&'static Transform>,
        Option<&'static TileColor>,
    ), AnyDisabling>,

    pub tilemaps: Query<'w, 's, (
        &'static mut TilemapTexture,
        &'static mut TileStorage,
        &'static mut TmapHashIdtoTextureIndex,
    ), ()>,
    pub image_size_map: Res<'w, ImageSizeMap>,

    pub texture_overlay_mat: ResMut<'w, Assets<MonoRepeatTextureOverlayMat>>,
    pub voronoi_mat: ResMut<'w, Assets<VoronoiTextureOverlayMat>>,
    pub wavy_mat: ResMut<'w, Assets<WavyMat>>,
    pub chunkrange: Res<'w, AaChunkRangeSettings>,

    pub min_dists_query: Query<'w, 's, (&'static MinDistancesMap), AnyDisabling>,
    pub regpos_map: ResMut<'w, RegisteredPositions>,
    pub shader_query: Query<'w, 's, (&'static TileShader), ()>,

    pub loaded_chunks: Res<'w, LoadedChunks>,
    pub state: Res<'w, State<ClientState>>,
}

#[allow(unused_parens, )]//TODO: USAR try_insert_bundle
pub fn process_tiles_pre(
    mut cmd: Commands,
    mut params: ProcessTilesPreParams,
    mut tmap_map: Local<HashMap<MapKey, MapStruct>>,
) {
    let is_host = *params.state.get() == ClientState::Disconnected;

    let mut prepared_tiles: Vec<TilemapPreparedTile> = Vec::new();
    params.tilemap_tasks.prep_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            prepared_tiles.extend(result.prepared_tiles);
            false
        } else {
            true
        }
    });

    if prepared_tiles.is_empty() {
        if !params.tilemap_tasks.prep_tasks.is_empty() { return; }
        if params.collected_tiles.0.is_empty() { return; }

        let inputs = collect_tilemap_prep_inputs(&mut cmd, &mut params.collected_tiles, &params.ezero_query);
        if inputs.is_empty() { return; }
        let image_sizes: HashMap<AssetId<Image>, U16Vec2> = params.image_size_map
            .0
            .iter()
            .map(|(id, size)| (*id, *size))
            .collect();
        let loaded_chunks_map: HashMap<(DimensionRef, ChunkPos), Entity> = params.loaded_chunks
            .0
            .iter()
            .map(|(key, &ent)| (*key, ent))
            .collect();

        let task_pool = AsyncComputeTaskPool::get();
        params.tilemap_tasks.prep_tasks.push(task_pool.spawn(async move {
            process_tilemap_prep_batch(inputs, image_sizes, loaded_chunks_map)
        }));
    }

    if !tmap_map.is_empty() {
        tmap_map.retain(|key, _| params.loaded_chunks.0.contains_key(&(key.dim_ref(), key.chunk_pos())));
    }

    let reserved = params.chunkrange.approximate_number_of_chunks(0.06);
    let tiles_len = prepared_tiles.len();

    let mut changed_structs: HashSet<MapKey> = HashSet::with_capacity(reserved);


    let mut tilemap_bundles = Vec::with_capacity(200);//TODO HACER ALGO CON EL CHILDOF (CAMBIAR POR OTRO STRUCT?)

    let mut to_insert_replicated = Vec::with_capacity(tiles_len/100);
    let mut spritetiles_to_insert_pos_and_dim_ref = Vec::with_capacity(tiles_len/20);

    let mut tiles_to_insert: Vec<(Entity, TileMassSpawnBundle)> = Vec::with_capacity(tiles_len);
    for prepared in prepared_tiles {
        let tile_ent = prepared.tile_ent;
        let mut bundle = prepared.bundle;

        if false == params.regpos_map.check_min_distances(
            &mut cmd,
            is_host,
            (
                tile_ent,
                bundle.ezero_ref,
                bundle.dim_ref,
                bundle.gpos,
                prepared.min_dists.as_ref(),
                prepared.keep_distance_from.as_ref(),
            ),
            params.min_dists_query,
        ) {
            cmd.entity(tile_ent).try_despawn();
            info!(target: "tilemap_systems", "Tile entity {:?} at gpos {:?} in dim {:?} despawned due to min distance check failure", tile_ent, bundle.gpos, bundle.dim_ref);
            continue;
        }

        if prepared.to_persist {
            if is_host {
                to_insert_replicated.push((tile_ent, Replicated));
            } else {
                cmd.entity(tile_ent).try_despawn();
                continue;//client shouldn't spawn this
            }
        }

        if prepared.transform.is_some() {
            spritetiles_to_insert_pos_and_dim_ref.push((
                tile_ent,
                (
                    bundle.ezero_ref,
                    bundle.gpos,
                    bundle.tile_bundle.position,
                    bundle.dim_ref,
                    bundle.initial_pos,
                    SyncToRenderWorld::default(),
                ),
            ));
            continue;//is sprite tile
        }

        bundle.tile_bundle.color = prepared.color.unwrap_or_default();

        let Some(chunk) = prepared.chunk_ent else {
            let chunk_pos = ChunkPos::from(bundle.gpos);
            let gpos = bundle.gpos;
            let dim_ref = bundle.dim_ref;
            params.limbo_tiles.0.push(LimboTileEntry::new(tile_ent, bundle));
            trace!(target: "tilemap_systems", "Chunk for tile entity {:?} at gpos {:?}, {} in dim {:?} not loaded, sending to limbo", tile_ent, gpos, chunk_pos, dim_ref);
            continue;//chunk not loaded
        };
        let chunk_pos = ChunkPos::from(bundle.gpos);

        func_process_tile_into_tilemaps(
            &mut cmd,
            tile_ent,
            &mut bundle.tile_bundle.visible,
            &mut bundle.tile_bundle.texture_index,
            &mut bundle.tile_bundle.tilemap_id,
            bundle.oplist_size,
            bundle.tile_bundle.position,
            prepared.tile_z_index,
            prepared.tile_handles.as_ref(),
            prepared.shader_ref.as_ref(),
            prepared.tile_size,
            &mut *tmap_map,
            chunk,
            chunk_pos,
            bundle.dim_ref,
            &mut params.tilemaps,
            &mut changed_structs,
            &mut tilemap_bundles,
        );

        tiles_to_insert.push((tile_ent, bundle));
    }
    //DEJAR CON IF NEW ASÍ TILES DE TILEMAP PUEDEN SER REPLICADAS 
    cmd.try_insert_batch_if_new(tiles_to_insert);

    cmd.try_insert_batch(spritetiles_to_insert_pos_and_dim_ref);

    cmd.try_insert_batch(to_insert_replicated);

    cmd.try_insert_batch(tilemap_bundles);

    let mut insert2tmaps = Vec::with_capacity(changed_structs.len());
    let mut default_mats = Vec::with_capacity(changed_structs.len());
    let mut wavy_mats = Vec::with_capacity(changed_structs.len());
    let mut texture_overlay_mats = Vec::with_capacity(changed_structs.len());

    for mapkey in changed_structs.iter() {
        //trace!(target: "tilemap_systems", "Changed tilemap {:?} in chunk {:?}", mapkey, mapkey.chunk_pos());
        let Some(mapstruct) = tmap_map.get_mut(mapkey) else {
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
            };

        } else {
            default_mats.push((tmap_ent, MaterialTilemapHandle::<StandardTilemapMaterial>::default()));
        }
    }
    cmd.try_insert_batch(default_mats);
    cmd.try_insert_batch(texture_overlay_mats);
    cmd.try_insert_batch(wavy_mats);
    cmd.try_insert_batch(insert2tmaps);

}

#[allow(unused_parens)]
pub fn requeue_limbo_tiles(
    mut cmd: Commands,
    mut limbo_tiles: ResMut<TilemapLimboTiles>,
    mut collected_tiles: ResMut<MassCollectedTiles>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunk_query: Query<(Entity, &ChunkPos, &DimensionRef), With<Chunk>>,
    alive_query: Query<Entity>,
) {
    if limbo_tiles.0.is_empty() {
        return;
    }

    let mut i = 0;
    while i < limbo_tiles.0.len() {
        let mut entry = limbo_tiles.0.swap_remove(i);

        if alive_query.get(entry.tile_ent).is_err() {
            continue;
        }

        let chunk_pos = ChunkPos::from(entry.bundle.gpos);
        if loaded_chunks.0.contains_key(&(entry.bundle.dim_ref, chunk_pos)) {
            collected_tiles.0.push((entry.tile_ent, entry.bundle));
            continue;
        }

        if let Some((chunk_ent, _, _)) = chunk_query
            .iter()
            .find(|(_, pos, dim)| **pos == chunk_pos && **dim == entry.bundle.dim_ref)
        {
            // in case LoadedChunks missed this chunk
            loaded_chunks
                .0
                .insert((entry.bundle.dim_ref, chunk_pos), chunk_ent);
            collected_tiles.0.push((entry.tile_ent, entry.bundle));
            continue;
        }

        if entry.retries_left == 0 {
            cmd.entity(entry.tile_ent).try_despawn();
            continue;
        }

        entry.retries_left -= 1;
        limbo_tiles.0.push(entry);
        i += 1;
    }
}

#[derive(Clone)]
struct TilemapPrepInput {
    tile_ent: Entity,
    bundle: TileMassSpawnBundle,
    tile_strid: TileStrId,
    min_dists: Option<MinDistancesMap>,
    keep_distance_from: Option<KeepDistanceFrom>,
    to_persist: bool,
    tile_z_index: AcZ,
    tile_handles: Option<TileHidsHandles>,
    shader_ref: Option<TileShaderRef>,
    transform: Option<Transform>,
    color: Option<TileColor>,
}

fn collect_tilemap_prep_inputs(
    cmd: &mut Commands,
    collected_tiles: &mut MassCollectedTiles,
    ezero_query: &Query<(
        &TileStrId,
        Option<&MinDistancesMap>,
        Option<&KeepDistanceFrom>,
        Has<Persisted>,
        Option<&AcZ>,
        Option<&TileHidsHandles>,
        Option<&TileShaderRef>,
        Option<&Transform>,
        Option<&TileColor>,
    ), AnyDisabling>,
) -> Vec<TilemapPrepInput> {
    let mut inputs = Vec::with_capacity(collected_tiles.0.len());
    for (tile_ent, bundle) in collected_tiles.0.drain(..) {
        let Ok((tile_strid, min_dists, keep_distance_from, to_persist, tile_z_index, tile_handles, shader_ref, transform, color))
            = ezero_query.get(bundle.ezero_ref.0)
        else {
            error!(target: "tilemap_systems", "Original tile entity {} is despawned", bundle.ezero_ref.0);
            cmd.entity(tile_ent).try_despawn();
            continue;
        };

        inputs.push(TilemapPrepInput {
            tile_ent,
            bundle,
            tile_strid: tile_strid.clone(),
            min_dists: min_dists.cloned(),
            keep_distance_from: keep_distance_from.cloned(),
            to_persist,
            tile_z_index: tile_z_index.cloned().unwrap_or_default(),
            tile_handles: tile_handles.cloned(),
            shader_ref: shader_ref.copied(),
            transform: transform.cloned(),
            color: color.copied(),
        });
    }
    inputs
}

fn process_tilemap_prep_batch(
    inputs: Vec<TilemapPrepInput>,
    image_sizes: HashMap<AssetId<Image>, U16Vec2>,
    loaded_chunks: HashMap<(DimensionRef, ChunkPos), Entity>,
) -> TilemapPrepResult {
    let mut prepared_tiles = Vec::with_capacity(inputs.len());

    for input in inputs {
        let tile_size = if let Some(ref handles) = input.tile_handles {
            image_sizes
                .get(&handles.first_handle().id())
                .copied()
                .unwrap_or(U16Vec2::ONE)
        } else {
            U16Vec2::ONE
        };

        let chunk_pos = ChunkPos::from(input.bundle.gpos);
        let chunk_ent = loaded_chunks.get(&(input.bundle.dim_ref, chunk_pos)).copied();

        prepared_tiles.push(TilemapPreparedTile {
            tile_ent: input.tile_ent,
            bundle: input.bundle,
            tile_strid: input.tile_strid,
            min_dists: input.min_dists,
            keep_distance_from: input.keep_distance_from,
            to_persist: input.to_persist,
            tile_z_index: input.tile_z_index,
            tile_handles: input.tile_handles,
            shader_ref: input.shader_ref,
            transform: input.transform,
            color: input.color,
            tile_size,
            chunk_ent,
        });
    }

    TilemapPrepResult { prepared_tiles }
}




#[allow(clippy::too_many_arguments)]
fn func_process_tile_into_tilemaps(
    cmd: &mut Commands,
    tile_ent: Entity,
    tile_visible: &mut TileVisible,
    texture_index: &mut TileTextureIndex,
    tilemap_id: &mut TilemapId,
    oplist_size: OplistSize,
    position: TilePos,
    tile_z_index: AcZ,
    tile_handles: Option<&TileHidsHandles>,
    shader_ref: Option<&TileShaderRef>,
    tile_size: U16Vec2,
    tmap_map: &mut HashMap<MapKey, MapStruct>,
    chunk: Entity,
    chunk_pos: ChunkPos,
    dim_ref: DimensionRef,
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex)>,
    changed_structs: &mut HashSet<MapKey>,
    tilemap_bundles: &mut Vec<(Entity, (TilemapConfig, AcZ, ChildOf, ChunkPos, DimensionRef))>,
) {
    let tile_size = match tile_handles {
        Some(_) => tile_size,
        None => {
            tile_visible.0 = false; 
            error!(target: "tilemap_systems", "Tile entity {:?} has no TileHidsHandles", tile_ent);
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

        for (id, handle) in tile_handles.iter() {
            let texture_index = tmap_handles
                .iter()
                .position(|x| *x == *handle)
                .map(|i| TileTextureIndex(i as u32))
                .unwrap_or_else(|| {
                    tmap_handles.push(handle.clone());
                    TileTextureIndex((tmap_handles.len() - 1) as u32)
                });
            tmap_hash_id_map.0.insert_with_id(id, texture_index);
            if first_texture_index.is_none() {
                first_texture_index = Some(texture_index);
            }
        }
        texture_index.0 = first_texture_index.unwrap_or_default().0;

    } else {
        let mut tmap_hash_id_map = TmapHashIdtoTextureIndex::with_capacity(0);
        changed_structs.insert(map_key.clone());

        let handles = if let Some(tile_handles) = tile_handles {
            tmap_hash_id_map.0.reserve(tile_handles.len());
            for (i, (id, _)) in tile_handles.iter().enumerate() {
                tmap_hash_id_map.0.insert_with_id(id, TileTextureIndex(i as u32));
            }
            tile_handles.handles().clone()
        } else {
            Vec::new()
        };
        let tmap_ent = cmd.spawn_empty().id();

        tilemap_bundles.push(
            (tmap_ent,
            (
                TilemapConfig::new(oplist_size, tile_size, ),
                tile_z_index,
                ChildOf(chunk),
                chunk_pos,
                dim_ref,
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
    }
}


#[allow(unused_parens)]
pub fn tile_assign_child_of(mut cmd: Commands, 
    tile_instances_holder_query: Single<Entity, With<TileInstancesHolder>>,
    query: Query<(Entity, &TilemapId), (Without<ChildOf>, AnyDisabling)>,
) {
    let parent = tile_instances_holder_query.into_inner();
    
    let mut child_ofs_for_tiles: Vec<(Entity, ChildOf)> = Vec::with_capacity(query.iter().size_hint().0);
    for (tile_ent, &tile_map_id) in query.iter() {
        if tile_map_id == TilemapId::default() {
            cmd.entity(tile_ent).try_despawn();
            continue;
        }
        child_ofs_for_tiles.push((tile_ent, ChildOf(parent)));
    }

    cmd.try_insert_batch(child_ofs_for_tiles);
}