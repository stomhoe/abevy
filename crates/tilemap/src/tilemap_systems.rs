use bevy::{ecs::entity_disabling::Disabled, math::U16Vec2, platform::collections::HashSet, prelude::*, render::sync_world::SyncToRenderWorld};
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::{ClientState, Replicated};
use common::{common_components::{AnyDisabling}, common_resources::ImageSizeMap, };
use game_common::game_common_components::{Persisted};
use sprite_shared::AcZ;
use ::tilemap_shared::*;

use crate::{chunking_components::*, chunking_resources::*, terrain_gen::terrgen_resources::*, tile::{tile_components::*, tile_shader::{tile_material::prelude::*, tile_shader_components::*}, }, tilemap_components::*, tilemap_resources::*};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct MapKey {ac_z: AcZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>, }
impl MapKey {
    pub fn new(ac_z: AcZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>) -> Self {
        Self { ac_z, oplist_size, tile_size, shader_ref }
    }
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
use std::mem::take;
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
) -> Result {unsafe{

    let is_host = *state.get() == ClientState::Disconnected;

    if collected_tiles.0.is_empty() { return Ok(()); }

    let reserved = chunkrange.approximate_number_of_chunks(0.06);
    let tiles_len = collected_tiles.0.len();

    let mut changed_structs: HashSet<(Entity, MapKey)> = HashSet::with_capacity(reserved);


    let mut tilemap_bundles = Vec::with_capacity(200);//TODO HACER ALGO CON EL CHILDOF (CAMBIAR POR OTRO STRUCT?)

    let mut to_insert_replicated = Vec::with_capacity(tiles_len/100);
    let mut spritetiles_to_insert_pos_and_dim_ref = Vec::with_capacity(tiles_len/20);

    let mut i = 0;
    while i < collected_tiles.0.len() {
        let ev = collected_tiles.0.get_unchecked_mut(i);

        let &mut (tile_ent, TileMassSpawnBundle {
            ezero_ref, gpos, dim_ref, oplist_size, tile_bundle: ref mut bundle, initial_pos, prev_gpos: _, prev_dim_ref: _,
        }) = ev;

        let Ok((_tile_strid, min_dists, keep_distance_from, to_persist, tile_z_index, tile_handles, shader_ref, transform, color, ))
        = ezero_query.get(ezero_ref.0) else{
            error!(target: "tilemap_systems", "Original tile entity {} is despawned", ezero_ref.0);
            continue;
        };
        
        if false == regpos_map.check_min_distances(&mut cmd, is_host, (tile_ent, ezero_ref, dim_ref, gpos, min_dists, keep_distance_from), min_dists_query) {
            
            collected_tiles.0.swap_remove(i); cmd.entity(tile_ent).try_despawn(); 
            info!(target: "tilemap_systems", "Tile entity {:?} at gpos {:?} in dim {:?} despawned due to min distance check failure", tile_ent, gpos, dim_ref);
            continue; 
        }
        if to_persist {
            if is_host {
                to_insert_replicated.push((tile_ent, Replicated));
            }
            else{
                collected_tiles.0.swap_remove(i); cmd.entity(tile_ent).try_despawn(); 
                continue;//client shouldn't spawn this
            }
        }
        if transform.is_some() {
            //trace!(target: "tilemap_systems", "Processing tile entity {:?} with strid {:?}", tile_ent, tile_strid);
            spritetiles_to_insert_pos_and_dim_ref.push((tile_ent, (ezero_ref, gpos, bundle.position, dim_ref, initial_pos, SyncToRenderWorld::default())));
            collected_tiles.0.swap_remove(i);
            // Disabled is removed in tile_readjust_transform !

            continue;//is sprite tile
        }
        bundle.color = color.cloned().unwrap_or_default();
        
        let Some(&chunk) = loaded_chunks.0.get(&(dim_ref, gpos.into())) else {
            let chunk_pos = ChunkPos::from(gpos);
            collected_tiles.0.swap_remove(i); cmd.entity(tile_ent).try_despawn(); 
            trace!(target: "tilemap_systems", "Chunk for tile entity {:?} at gpos {:?}, {} in dim {:?} not loaded, despawning tile", tile_ent, gpos, chunk_pos, dim_ref);
            continue;//chunk not loaded
        };
        let Ok(mut layers) = chunk_query.get_mut(chunk) else {
            collected_tiles.0.swap_remove(i); cmd.entity(tile_ent).try_despawn(); 
            trace!(target: "tilemap_systems", "Chunk entity {:?} not found in chunk query when processing tile entity {:?}, despawning tile", chunk, tile_ent);
            continue;//chunk entity not found
        };

        func_process_tile_into_tilemaps(
            &mut cmd,
            tile_ent,
            &mut bundle.visible,
            &mut bundle.texture_index,
            &mut bundle.tilemap_id,
            oplist_size,
            bundle.position,
            tile_z_index.cloned().unwrap_or_default(),
            tile_handles,
            shader_ref,
            &image_size_map,
            &mut layers,
            chunk,
            &mut tilemaps,
            &mut changed_structs,
            &mut tilemap_bundles,
        );
        i += 1;
    }
    //DEJAR CON IF NEW ASÍ TILES DE TILEMAP PUEDEN SER REPLICADAS 
    cmd.try_insert_batch_if_new(take(&mut collected_tiles.0));

    cmd.try_insert_batch(spritetiles_to_insert_pos_and_dim_ref);

    cmd.try_insert_batch(to_insert_replicated);

    cmd.try_insert_batch(tilemap_bundles);

    let mut insert2tmaps = Vec::with_capacity(changed_structs.len());
    let mut default_mats = Vec::with_capacity(changed_structs.len());
    let mut wavy_mats = Vec::with_capacity(changed_structs.len());
    let mut texture_overlay_mats = Vec::with_capacity(changed_structs.len());

    for (chunk_ent, mapkey) in changed_structs.iter() {
        //trace!(target: "tilemap_systems", "Changed tilemap {:?} in chunk {:?}", mapkey, chunk_ent);

        let Ok(mut layers) = chunk_query.get_mut(*chunk_ent) else {
            continue ;
        };

        //DEJAR EN GET_MUT, CON REMOVE SE PIERDE LA TMAP ENTITY USADA ARRIBA
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

        let shader = if let Some(shader_ref) = mapkey.shader_ref {
            shader_query.get(shader_ref.0).ok().map(|(shader,)| shader.clone())
        } else {
            None
        };
        if let Some(shader) = shader {
            //trace!(target: "tilemap_systems", "Inserting tmapshader {:?} for tilemap entity {:?}", shader, tmap_ent);
            match shader {
                TileShader::TexRepeat(handle) => {
                    let material = MaterialTilemapHandle::from(texture_overlay_mat.add(handle));
                    texture_overlay_mats.push((tmap_ent, material));
                }
                TileShader::Voronoi(handle) => {
                    let material = MaterialTilemapHandle::from(voronoi_mat.add(handle));
                    cmd.entity(tmap_ent).try_insert(material);
                }
                TileShader::Wavy(handle) => {
                    let material = MaterialTilemapHandle::from(wavy_mat.add(handle));
                    wavy_mats.push((tmap_ent, material.clone()));
                }
                TileShader::TwoTexRepeat(_handle) => todo!(),
            };

        } else {
            default_mats.push((tmap_ent, MaterialTilemapHandle::<StandardTilemapMaterial>::default()));
        }
    }
    cmd.try_insert_batch(texture_overlay_mats);
    cmd.try_insert_batch(wavy_mats);
    cmd.try_insert_batch(insert2tmaps);

    Ok(())
}}




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
    image_size_map: &ImageSizeMap,
    layers: &mut ChunkTmapsMap,
    chunk: Entity,
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex)>,
    changed_structs: &mut HashSet<(Entity, MapKey)>,
    tilemap_bundles: &mut Vec<(Entity, (TilemapConfig, AcZ, ChildOf))>,
) {

    let tile_size = match tile_handles {
        Some(handles) => image_size_map.0.get(&handles.first_handle().id()).copied()
        .unwrap_or(U16Vec2::ONE) ,
        None => {
            tile_visible.0 = false; 
            error!(target: "tilemap_systems", "Tile entity {:?} has no TileHidsHandles", tile_ent);
            return;
        }
    };
    let map_key = MapKey::new(tile_z_index, oplist_size, tile_size, shader_ref.copied());

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
                tile_z_index,
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
    query: Query<Entity, (Without<ChildOf>, With<TilemapId>, With<TileTextureIndex>, AnyDisabling)>,
) {
    let parent = tile_instances_holder_query.into_inner();

    let child_ofs_for_tiles: Vec<(Entity, ChildOf)> = query
        .iter()
        .map(|tile_ent| (tile_ent, ChildOf(parent)))
        .collect();

    cmd.try_insert_batch(child_ofs_for_tiles);
}

