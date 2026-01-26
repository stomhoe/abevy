use bevy::{asset::AssetId, math::U16Vec2, platform::collections::HashSet, prelude::*, render::sync_world::SyncToRenderWorld, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::{ClientState, Replicated};
use common::{common_components::{AnyDisabling}, common_resources::ImageSizeMap, };
use game_common::game_common_components::{Persisted};
use sprite_shared::AcZ;
use ::tilemap_shared::*;
use dimension_shared::DimensionRef;

use crate::{chunking_components::*, chunking_resources::*, terrain_gen::terrgen_resources::*, tile::{tile_components::*, tile_shader::{tile_material::prelude::*, tile_shader_components::*}, }, tilemap_components::*, tilemap_resources::*};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct MapKey {ac_z: AcZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: TileShaderRef, }
impl MapKey {
    pub fn new(ac_z: AcZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: TileShaderRef) -> Self {
        Self { ac_z, oplist_size, tile_size, shader_ref }
    }
    pub fn ac_z(&self) -> AcZ {self.ac_z}
    pub fn oplist_size(&self) -> OplistSize {self.oplist_size}
    pub fn tile_size(&self) -> U16Vec2 {self.tile_size}
    pub fn shader_ref(&self) -> TileShaderRef {self.shader_ref}
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

#[allow(unused_parens, )]//TODO: USAR try_insert_bundle
pub fn process_tiles_pre(
    mut cmd: Commands, 

    mut collected_tiles: ResMut<MassCollectedTiles>,
    mut tilemap_tasks: ResMut<TilemapAsyncTasks>,

    ezero_query: Query<(&TileStrId, Option<&MinDistancesMap>, Option<&KeepDistanceFrom>, Has<Persisted>, 
        Option<&AcZ>, Option<&TileHidsHandles>, Option<&TileShaderRef>, Option<&Transform>, Option<&TileColor>, ), (AnyDisabling)>,

    mut chunk_query: Query<(&mut ChunkTmapsMap), ()>,
    mut tilemaps: Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex, ), ( )>,
    image_size_map: Res<ImageSizeMap>,

    mut texture_overlay_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
    mut voronoi_mat: ResMut<Assets<VoronoiTextureOverlayMat>>,
    mut wavy_mat: ResMut<Assets<WavyMat>>,
    chunkrange: Res<AaChunkRangeSettings>,

    min_dists_query: Query<(&MinDistancesMap), (AnyDisabling)>,
    mut regpos_map: ResMut<RegisteredPositions>,
    shader_query: Query<(&TileShader, ), ( )>,

    loaded_chunks: Res<LoadedChunks>,
    state: Res<State<ClientState>>,
) -> Result {

    let is_host = *state.get() == ClientState::Disconnected;

    let mut plan_results: Vec<TilemapPlanResult> = Vec::new();
    tilemap_tasks.plan_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            plan_results.push(result);
            false
        } else {
            true
        }
    });

    if let Some(plan_result) = plan_results.pop() {
            apply_tilemap_plan(
                &mut cmd,
                plan_result,
                &mut chunk_query,
                &mut tilemaps,
                &shader_query,
                &mut texture_overlay_mat,
                &mut voronoi_mat,
                &mut wavy_mat,
                chunkrange,
            );
        return Ok(());
    }
    if !tilemap_tasks.plan_tasks.is_empty() {
        return Ok(());
    }

    let mut prepared_tiles: Vec<TilemapPreparedTile> = Vec::new();
    tilemap_tasks.prep_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            prepared_tiles.extend(result.prepared_tiles);
            false
        } else {
            true
        }
    });

    if prepared_tiles.is_empty() {
        if !tilemap_tasks.prep_tasks.is_empty() { return Ok(()); }
        if collected_tiles.0.is_empty() { return Ok(()); }

        let inputs = collect_tilemap_prep_inputs(&mut cmd, &mut collected_tiles, &ezero_query);
        if inputs.is_empty() { return Ok(()); }

        let image_sizes: HashMap<AssetId<Image>, U16Vec2> = image_size_map
            .0
            .iter()
            .map(|(id, size)| (*id, *size))
            .collect();
        let loaded_chunks_map: HashMap<(DimensionRef, ChunkPos), Entity> = loaded_chunks
            .0
            .iter()
            .map(|(key, &ent)| (*key, ent))
            .collect();

        let task_pool = AsyncComputeTaskPool::get();
        tilemap_tasks.prep_tasks.push(task_pool.spawn(async move {
            process_tilemap_prep_batch(inputs, image_sizes, loaded_chunks_map)
        }));
        return Ok(());
    }

    let mut plan_inputs: Vec<TilemapPlanInput> = Vec::with_capacity(prepared_tiles.len());
    for prepared in prepared_tiles {
        let tile_ent = prepared.tile_ent;
        let mut bundle = prepared.bundle;

        if false == regpos_map.check_min_distances(
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
            min_dists_query,
        ) {
            cmd.entity(tile_ent).try_despawn();
            info!(target: "tilemap_systems", "Tile entity {:?} at gpos {:?} in dim {:?} despawned due to min distance check failure", tile_ent, bundle.gpos, bundle.dim_ref);
            continue;
        }

        let should_replicate = prepared.to_persist && is_host;
        if prepared.to_persist && !is_host {
            cmd.entity(tile_ent).try_despawn();
            continue;//client shouldn't spawn this
        }

        let is_sprite = prepared.transform.is_some();
        if !is_sprite && prepared.chunk_ent.is_none() {
            let chunk_pos = ChunkPos::from(bundle.gpos);
            cmd.entity(tile_ent).try_despawn();
            trace!(target: "tilemap_systems", "Chunk for tile entity {:?} at gpos {:?}, {} in dim {:?} not loaded, despawning tile", tile_ent, bundle.gpos, chunk_pos, bundle.dim_ref);
            continue;//chunk not loaded
        }

        bundle.tile_bundle.color = prepared.color.unwrap_or_default();
        plan_inputs.push(TilemapPlanInput {
            tile_ent,
            bundle,
            tile_z_index: prepared.tile_z_index,
            tile_handles: prepared.tile_handles,
            shader_ref: prepared.shader_ref,
            tile_size: prepared.tile_size,
            chunk_ent: prepared.chunk_ent,
            is_sprite,
            should_replicate,
        });
    }

    if plan_inputs.is_empty() {
        return Ok(());
    }

    let task_pool = AsyncComputeTaskPool::get();
    tilemap_tasks.plan_tasks.push(task_pool.spawn(async move {
        build_tilemap_plan(plan_inputs)
    }));
    Ok(())
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

fn build_tilemap_plan(inputs: Vec<TilemapPlanInput>) -> TilemapPlanResult {
    let mut result = TilemapPlanResult::default();
    result.tilemap_tiles = Vec::with_capacity(inputs.len());
    result.sprite_tiles = Vec::with_capacity(inputs.len() / 4);
    result.replicated_tiles = Vec::with_capacity(inputs.len() / 8);

    for input in inputs {
        let oplist_size = input.bundle.oplist_size;
        if input.should_replicate {
            result.replicated_tiles.push(input.tile_ent);
        }

        if input.is_sprite {
            result.sprite_tiles.push(SpriteTilePlan {
                tile_ent: input.tile_ent,
                ezero_ref: input.bundle.ezero_ref,
                gpos: input.bundle.gpos,
                position: input.bundle.tile_bundle.position,
                dim_ref: input.bundle.dim_ref,
                initial_pos: input.bundle.initial_pos,
            });
            continue;
        }

        let Some(chunk_ent) = input.chunk_ent else {
            continue;
        };

        result.tilemap_tiles.push(TilemapTilePlan {
            tile_ent: input.tile_ent,
            bundle: input.bundle,
            key: TilemapKeyData {
                ac_z: input.tile_z_index,
                oplist_size,
                tile_size: input.tile_size,
                shader_ref: input.shader_ref.unwrap_or_default(),
            },
            tile_handles: input.tile_handles,
            shader_ref: input.shader_ref.unwrap_or_default(),
            tile_size: input.tile_size,
            tile_z_index: input.tile_z_index,
            chunk_ent,
        });
    }

    result
}

#[allow(clippy::too_many_arguments)]
fn apply_tilemap_plan(
    cmd: &mut Commands,
    plan: TilemapPlanResult,
    chunk_query: &mut Query<&mut ChunkTmapsMap, ()>,
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex)>,
    shader_query: &Query<(&TileShader,), ()>,
    texture_overlay_mat: &mut Assets<MonoRepeatTextureOverlayMat>,
    voronoi_mat: &mut Assets<VoronoiTextureOverlayMat>,
    wavy_mat: &mut Assets<WavyMat>,
    chunkrange: Res<AaChunkRangeSettings>,
) {
    let reserved = chunkrange.approximate_number_of_chunks(0.06);
    let tiles_len = plan.tilemap_tiles.len();

    let mut changed_structs: HashSet<(Entity, MapKey)> = HashSet::with_capacity(reserved);
    let mut tilemap_bundles = Vec::with_capacity(200);
    let mut spritetiles_to_insert_pos_and_dim_ref = Vec::with_capacity(plan.sprite_tiles.len());
    //collect here
    let mut to_insert_replicated = Vec::with_capacity(plan.replicated_tiles.len());
    for &tile_ent in &plan.replicated_tiles {
        to_insert_replicated.push((tile_ent, Replicated));
    }
    let mut tiles_to_insert: Vec<(Entity, TileMassSpawnBundle)> = Vec::with_capacity(tiles_len);

    for sprite in plan.sprite_tiles {
        spritetiles_to_insert_pos_and_dim_ref.push((
            sprite.tile_ent,
            (
                sprite.ezero_ref,
                sprite.gpos,
                sprite.position,
                sprite.dim_ref,
                sprite.initial_pos,
                SyncToRenderWorld::default(),
            ),
        ));
    }


    for mut plan_tile in plan.tilemap_tiles {
        let chunk = plan_tile.chunk_ent;
        let Ok(mut layers) = chunk_query.get_mut(chunk) else {
            cmd.entity(plan_tile.tile_ent).try_despawn();
            trace!(target: "tilemap_systems", "Chunk entity {:?} not found in chunk query when processing tile entity {:?}, despawning tile", chunk, plan_tile.tile_ent);
            continue;
        };

        let map_key = MapKey::new(
            plan_tile.key.ac_z,
            plan_tile.key.oplist_size,
            plan_tile.key.tile_size,
            plan_tile.key.shader_ref,
        );

        func_process_tile_into_tilemaps(
            cmd,
            plan_tile.tile_ent,
            &mut plan_tile.bundle.tile_bundle.visible,
            &mut plan_tile.bundle.tile_bundle.texture_index,
            &mut plan_tile.bundle.tile_bundle.tilemap_id,
            plan_tile.bundle.oplist_size,
            plan_tile.bundle.tile_bundle.position,
            map_key,
            plan_tile.tile_handles.as_ref(),
            plan_tile.tile_size,
            &mut layers,
            chunk,
            tilemaps,
            &mut changed_structs,
            &mut tilemap_bundles,
        );

        tiles_to_insert.push((plan_tile.tile_ent, plan_tile.bundle));
    }

    cmd.try_insert_batch_if_new(tiles_to_insert);
    cmd.try_insert_batch(spritetiles_to_insert_pos_and_dim_ref);
    cmd.try_insert_batch(to_insert_replicated);
    cmd.try_insert_batch(tilemap_bundles);

    let mut insert2tmaps = Vec::with_capacity(changed_structs.len());
    let mut default_mats = Vec::with_capacity(changed_structs.len());
    let mut wavy_mats = Vec::with_capacity(changed_structs.len());
    let mut texture_overlay_mats = Vec::with_capacity(changed_structs.len());

    for (chunk_ent, mapkey) in changed_structs.iter() {
        let Ok(mut layers) = chunk_query.get_mut(*chunk_ent) else {
            continue ;
        };

        let Some(mapstruct) = layers.0.get_mut(mapkey) else {
            continue;
        };
        let tmap_ent = mapstruct.tmap_ent;

        let (texture_vec, storage, tmap_hash_id_map) = (
            mapstruct.take_texture(),
            mapstruct.take_storage(),
            mapstruct.take_hash_id_map(),
        );
        insert2tmaps.push((tmap_ent, (tmap_hash_id_map, storage, texture_vec, )));

        if let Ok((shader,)) = shader_query.get(mapkey.shader_ref.0) {
            match shader {
                TileShader::TexRepeat(handle) => {
                    let material = MaterialTilemapHandle::from(texture_overlay_mat.add(handle.clone()));
                    texture_overlay_mats.push((tmap_ent, material));
                }
                TileShader::Voronoi(handle) => {
                    let material = MaterialTilemapHandle::from(voronoi_mat.add(handle.clone()));
                    cmd.entity(tmap_ent).try_insert(material);
                }
                TileShader::Wavy(handle) => {
                    let material = MaterialTilemapHandle::from(wavy_mat.add(handle.clone()));
                    wavy_mats.push((tmap_ent, material.clone()));
                }
                TileShader::TwoTexRepeat(_handle) => todo!(),
            };
        }
        else {
            default_mats.push((tmap_ent, MaterialTilemapHandle::<StandardTilemapMaterial>::default()));
        }
    }
    cmd.try_insert_batch(default_mats);
    cmd.try_insert_batch(texture_overlay_mats);
    cmd.try_insert_batch(wavy_mats);
    cmd.try_insert_batch(insert2tmaps);
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
    map_key: MapKey,
    tile_handles: Option<&TileHidsHandles>,
    tile_size: U16Vec2,
    layers: &mut ChunkTmapsMap,
    chunk: Entity,
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex)>,
    changed_structs: &mut HashSet<(Entity, MapKey)>,
    tilemap_bundles: &mut Vec<(Entity, (TilemapConfig, AcZ, ChildOf))>,
) {
    let tile_size = match tile_handles {
        Some(_) => tile_size,
        None => {
            tile_visible.0 = false; 
            error!(target: "tilemap_systems", "Tile entity {:?} has no TileHidsHandles", tile_ent);
            return;
        }
    };
    if let Some(mapstruct) = layers.0.get_mut(&map_key) {
        let tmap_ent = mapstruct.tmap_ent;
        
        
        let (tmap_handles, storage, tmap_hash_id_map) =
        if let Ok((tmap_handles, storage, tmap_hash_id_map)) = tilemaps.get_mut(tmap_ent)
        {
            //no insertion into changed structs needed since tilemap's components are getting edited directly
            (tmap_handles.into_inner(), storage.into_inner(), tmap_hash_id_map.into_inner())
        } else {
            changed_structs.insert((chunk, map_key.clone()));
            let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
            (tmap_handles, storage, tmap_hash_id_map)
        };
        let Vector(tmap_handles) = tmap_handles else {
            error!(target: "tilemap_systems", "Failed to get tilemap handles for {:?}", tmap_ent);
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
        changed_structs.insert((chunk, map_key.clone()));

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
                TilemapConfig::new(oplist_size, tile_size),
                map_key.ac_z(),
                ChildOf(chunk),
            ))
        );

        tilemap_id.0 = tmap_ent;


        let mut storage = TilemapConfig::new_storage(oplist_size);
        storage.set(&position, tile_ent);
        layers.0.entry(map_key)
            .insert(MapStruct {
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

