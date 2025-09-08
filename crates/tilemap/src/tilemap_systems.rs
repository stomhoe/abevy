use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled, world::OnDespawn}, math::U16Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::{common_components::StrId, common_resources::ImageSizeMap, common_states::GameSetupType};
use dimension_shared::DimensionRef;
use game_common::game_common_components::{EntityZeroRef, MyZ};
use ::tilemap_shared::*;

use crate::{chunking_components::*, chunking_resources::AaChunkRangeSettings, terrain_gen::{terrgen_events::{MassCollectedTiles, TileHelperStruct,  }, terrgen_resources::RegisteredPositions}, tile::{tile_components::*, tile_materials::*}, tilemap_components::*};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct MapKey {z_index: MyZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>,}
impl MapKey {
    pub fn new(z_index: MyZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>) -> Self {
        Self { z_index, oplist_size, tile_size, shader_ref }
    }
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



use bevy::ecs::entity::EntityHashMap;


use bevy_ecs_tilemap::prelude::TilemapTexture::Vector;

#[allow(unused_parens, )]//TODO: USAR try_insert_bundle
pub fn process_tiles_pre(
    mut cmd: Commands, 

    mut ereader_mass_collected_tiles: ResMut<MassCollectedTiles>,

    oritile_query: Query<(&TileStrId, Option<&MinDistancesMap>, Option<&KeepDistanceFrom>, Has<ChunkOrTilemapChild>, 
        Option<&MyZ>, Option<&TileHidsHandles>, Option<&TileShaderRef>, Option<&mut Transform>, Option<&TileColor>), (With<Disabled>)>,

    mut chunk_query: Query<(&mut LayersMap), ()>,
    mut tilemaps: Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex, ), ( )>,
    image_size_map: Res<ImageSizeMap>,

    mut texture_overlay_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
    mut voronoi_mat: ResMut<Assets<VoronoiTextureOverlayMat>>,
    mut event_writer: EventWriter<DrawTilemap>,
    chunkrange: Res<AaChunkRangeSettings>,

    min_dists_query: Query<(&MinDistancesMap), (With<Disabled>)>,
    mut regpos_map: ResMut<RegisteredPositions>,
    shader_query: Query<(&TileShader, ), ( )>,
    state: Res<State<GameSetupType>>,
) -> Result {unsafe{

    let is_host = state.get() != &GameSetupType::AsJoiner;

    if ereader_mass_collected_tiles.0.is_empty() { return Ok(()); }

    let reserved = chunkrange.approximate_number_of_chunks(0.06);

    let mut changed_structs: HashSet<(Entity, MapKey)> = HashSet::with_capacity(reserved);


    let mut to_draw = Vec::with_capacity(ereader_mass_collected_tiles.0.len());

    let mut tilemap_bundles = Vec::new();//TODO HACER ALGO CON EL CHILDOF (CAMBIAR POR OTRO STRUCT?)

    let mut to_insert_replicated = Vec::new();
    let mut to_remove_tile_bundle_and_oplist: Vec<Entity> = Vec::new();


    // Closure capturing the environment
    let mut asd = |tile_ent: Entity, ezero: EntityZeroRef, global_pos: GlobalTilePos, chunk: Entity, dim_ref: DimensionRef, oplist_size: OplistSize| {
        // You can access variables from the outer scope here
        // Example usage:
        // cmd, regpos_map, is_host, min_dists_query, etc. are accessible

        // Add your logic here
    };

    #[allow(unused_mut)]
    'eventsfor: for mut ev in ereader_mass_collected_tiles.0.iter_mut() {

        let &mut (tile_ent, TileHelperStruct {
            ezero, global_pos, chunk, dim_ref, oplist_size, tile_bundle: ref mut bundle, initial_pos: _,
        })  = ev; 

            //TODO EXTRAER ESTA PARTE A UNA FUNCIÓN PARA Q SE PUEDAN ADMITIR EVENTOS EN OTRO FORMATO
    
            

            let Ok(mut layers) = chunk_query.get_mut(chunk.0) else {
                continue 'eventsfor;
            };

            let Ok((tile_strid, min_dists, keep_distance_from, is_child, tile_z_index, tile_handles, shader_ref, transform, color))
            = oritile_query.get(ezero.0) else{
                error!("Original tile entity {} is despawned", ezero.0);
                continue;
            };

            
            
            if false == regpos_map.check_min_distances(&mut cmd, is_host, 
                (tile_ent, ezero, dim_ref, global_pos, min_dists, keep_distance_from), min_dists_query) {
                    cmd.entity(tile_ent).try_despawn(); 
                continue; 
            }
            
            
            if !is_child {
                if is_host {
                    cmd.entity(tile_ent).try_insert((ChildOf(dim_ref.0), ));
                    to_insert_replicated.push((tile_ent, Replicated));
                }
                else{
                    cmd.entity(tile_ent).try_despawn();
                    continue;
                }
            }
            
            cmd.entity(tile_ent).try_remove::<(Disabled, )>();

            if transform.is_some() {
                to_remove_tile_bundle_and_oplist.push(tile_ent);
                
                if is_child  {
                    cmd.entity(tile_ent).try_insert((ChildOf(chunk.0), ));
                }
                continue;
            }
            bundle.color = color.cloned().unwrap_or_default();
            
            process_tilemaps(
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
                chunk.0,
                &mut tilemaps,
                &mut changed_structs,
                &mut tilemap_bundles,
                &mut to_draw,
            );

            
        

            
        }
    cmd.try_insert_batch(take(&mut ereader_mass_collected_tiles.0));

    for tile_ent in to_remove_tile_bundle_and_oplist.drain(..) {
        cmd.entity(tile_ent).try_remove::<(TileBundle, OplistSize)>();
    }

    cmd.try_insert_batch(to_insert_replicated);


    cmd.try_insert_batch(tilemap_bundles);

    for (chunk_ent, mapkey) in changed_structs.iter() {
        trace!("Changed tilemap {:?} in chunk {:?}", mapkey, chunk_ent);

        let Ok(mut layers) = chunk_query.get_mut(*chunk_ent) else {
            continue ;
        };

        //DEJAR EN GET_MUT, CON REMOVE SE PIERDE LA TMAP ENTITY USADA ARRIBA
        let Some(mapstruct) = layers.0.get_mut(mapkey) else {
            continue ;
        };
        let tmap_ent = mapstruct.tmap_ent;

        let (texture, storage, tmap_hash_id_map) = (
            mapstruct.take_texture(),
            mapstruct.take_storage(),
            mapstruct.take_hash_id_map(),
        );

        let shader = if let Some(shader_ref) = mapkey.shader_ref {
            shader_query.get(shader_ref.0).ok().map(|(shader,)| shader.clone())
        } else {
            None
        };
        cmd.entity(tmap_ent).insert((//TODO: usar try_insert_bundle
            tmap_hash_id_map,
            storage,
            texture,
        ));
        
        if let Some(shader) = shader {
            trace!("Inserting tmapshader {:?} for tilemap entity {:?}", shader, tmap_ent);
            match shader {
                TileShader::TexRepeat(handle) => {
                    let material = MaterialTilemapHandle::from(texture_overlay_mat.add(handle.clone()));
                    cmd.entity(tmap_ent).try_insert(material);
                }
                TileShader::Voronoi(handle) => {
                    let material = MaterialTilemapHandle::from(voronoi_mat.add(handle.clone()));
                    cmd.entity(tmap_ent).try_insert(material);
                }
                TileShader::TwoTexRepeat(handle) => todo!(),
            };

        } else {
            cmd.entity(tmap_ent)
            .try_insert(MaterialTilemapHandle::<StandardTilemapMaterial>::default());
        }
    }


        //CLONES PROVISORIOS
    event_writer.write_batch(to_draw);


    Ok(())
}}

#[allow(clippy::too_many_arguments)]
fn process_tilemaps(
    cmd: &mut Commands,
    tile_ent: Entity,
    tile_visible: &mut TileVisible,
    texture_index: &mut TileTextureIndex,
    tilemap_id: &mut TilemapId,
    oplist_size: OplistSize,
    position: TilePos,
    tile_z_index: MyZ,
    tile_handles: Option<&TileHidsHandles>,
    shader_ref: Option<&TileShaderRef>,
    image_size_map: &ImageSizeMap,
    layers: &mut LayersMap,
    chunk: Entity,
    tilemaps: &mut Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex)>,
    changed_structs: &mut HashSet<(Entity, MapKey)>,
    tilemap_bundles: &mut Vec<(Entity, (TilemapConfig, MyZ, ChildOf))>,
    to_draw: &mut Vec<DrawTilemap>,
) {

    let tile_size = match tile_handles {
        Some(handles) => (image_size_map.0.get(&handles.first_handle()).copied().unwrap_or(U16Vec2::ONE)),
        None => {
            tile_visible.0 = false; U16Vec2::ONE
        }
    };

    let map_key = MapKey::new(tile_z_index, oplist_size, tile_size, shader_ref.copied());

    if let Some(mapstruct) = layers.0.get_mut(&map_key) {
        let tmap_ent = mapstruct.tmap_ent;
        if to_draw.iter().all(|e: &DrawTilemap| e.0 != tmap_ent) {
            to_draw.push(DrawTilemap(tmap_ent));
        }

        let (tmap_handles, storage, tmap_hash_id_map) =
            if let Ok((tmap_handles, storage, tmap_hash_id_map)) = tilemaps.get_mut(tmap_ent)
            {
                (tmap_handles.into_inner(), storage.into_inner(), tmap_hash_id_map.into_inner())
            } else {
                changed_structs.insert((chunk, map_key.clone()));
                let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
                (tmap_handles, storage, tmap_hash_id_map)
            };
        let Vector(tmap_handles) = tmap_handles else {
            error!("Failed to get tilemap handles for {:?}", tmap_ent);
            return;
        };

        tilemap_id.0 = tmap_ent;

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
        let mut tmap_hash_id_map = TmapHashIdtoTextureIndex::default();
        changed_structs.insert((chunk, map_key.clone()));

        let handles = if let Some(tile_handles) = tile_handles {
            for (i, (id, _)) in tile_handles.iter().enumerate() {
                tmap_hash_id_map.0.insert_with_id(id, TileTextureIndex(i as u32));
            }
            tile_handles.handles().clone()
        } else {
            Vec::new()
        };

        let tmap_ent = cmd.spawn(ChildOf(chunk)).id();

        tilemap_bundles.push(
            (tmap_ent,
            (
                TilemapConfig::new(oplist_size, tile_size),
                map_key.z_index,
                ChildOf(chunk),
            ))
        );

        tilemap_id.0 = tmap_ent;

        to_draw.push(DrawTilemap(tmap_ent));

        let mut storage = TilemapConfig::new_storage(oplist_size);
        storage.set(&position, tile_ent);
        layers.0.insert(map_key, MapStruct {
            tmap_ent,
            texture: TilemapTexture::Vector(handles),
            storage,
            tmap_hash_id_map,
        });
    }
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn tile_assign_child_of(mut cmd: Commands, 
    tile_instances_holder_query: Single<Entity, With<TileInstancesHolder>>,
    mut query: Query<(Entity, ),(Without<ChildOf>, With<TilePos>, With<TileTextureIndex>)>,
) {
    let query_len = query.iter().count();
    let mut child_ofs_for_tiles: Vec<(Entity, ChildOf)> = Vec::with_capacity(query_len);
    let parent = tile_instances_holder_query.into_inner();

    for (tile_ent,) in query.iter_mut() {
        //info!("Assigning ChildOf to tile entity {:?}", tile_ent);
        child_ofs_for_tiles.push((tile_ent, ChildOf(parent.clone())));//TODO HACER EN EL MOMENTO Q SE LE METEN LAS COSAS A LAS TILES
    }

    cmd.try_insert_batch(child_ofs_for_tiles);
}


#[derive(Event, Eq, PartialEq, Hash)]
pub struct TilemapChanged (pub Entity);

#[allow(unused_parens)]
pub fn despawn_orphan_tilemaps(mut cmd: Commands, 
    mut query: Query<(Entity, ), (Without<ChildOf>, With<TilemapGridSize>)>,
) {
    for (tilemap_ent, ) in query.iter_mut() {
        info!("Despawning orphan tilemap entity {:?}", tilemap_ent);
        cmd.entity(tilemap_ent).despawn();
    }
}