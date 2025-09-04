use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled, world::OnDespawn}, math::U16Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use common::{common_components::StrId, common_resources::ImageSizeMap};
use game_common::game_common_components::MyZ;
use ::tilemap_shared::*;

use crate::{chunking_components::*, terrain_gen::{terrgen_events::ProcessedTiles}, tile::{tile_components::*, tile_materials::*}, tilemap_components::*};



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

#[allow(unused_parens, )]
pub fn produce_tilemaps(
    mut cmd: Commands, 
    mut ereader_processed_tiles: EventReader<ProcessedTiles>,
    tile_comps: Query<(Entity, &TilePos, &OplistSize, Option<&TileHidsHandles>, Option<&MyZ>, Option<&TileShaderRef>, ), 
    (Or<(With<Disabled>, Without<Disabled>)>, With<ChunkOrTilemapChild>, Without<Transform>)>,
    mut chunk_query: Query<(&mut LayersMap), ()>,
    mut tilemaps: Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex, ), ( )>,
    image_size_map: Res<ImageSizeMap>,

    mut texture_overlay_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
    mut voronoi_mat: ResMut<Assets<VoronoiTextureOverlayMat>>,
    mut event_writer: EventWriter<DrawTilemap>,
    shader_query: Query<(&TileShader, ), ( )>,
) -> Result {

    //let mut changed_tilemaps = HashSet::new();

    let mut changed_structs: HashSet<(Entity, MapKey)> = HashSet::new();
    let mut to_draw = HashSet::new();

    #[allow(unused_mut)]
    'eventsfor: for ev in ereader_processed_tiles.read() {
        trace!("Processing event for chunk {:?}", ev.chunk);

        let Ok(mut layers) = chunk_query.get_mut(ev.chunk) else {
            error!("Failed to get layers for chunk {:?}", ev.chunk);
            continue 'eventsfor;
        };

        'tilefor: for tile_ent in ev.tiles.iter() {
            

            let Ok((tile_ent, &tile_pos, &oplist_size, tile_handles, tile_z_index, shader_ref, )) = tile_comps.get(tile_ent)
            else { 
                continue 'tilefor;
            };


            cmd.entity(tile_ent)
            .try_insert_if_new(TileBundle::default())
            .try_remove::<(ChunkOrTilemapChild, Disabled)>();

            let tile_z_index = tile_z_index.cloned().unwrap_or_default();

            let tile_size = match tile_handles {
                Some(handles) => (image_size_map.0.get(&handles.first_handle()).copied().unwrap_or(U16Vec2::ONE)),
                None => {
                    cmd.entity(tile_ent).try_insert(TileVisible(false)); U16Vec2::ONE
                }
            };

            let map_key = MapKey::new(tile_z_index, oplist_size, tile_size, shader_ref.copied());
            trace!("Changed tilemap {:?} in chunk {:?}", map_key, ev.chunk);

            
            if let Some(mapstruct) = layers.0.get_mut(&map_key) {
                let tmap_ent = mapstruct.tmap_ent;
                to_draw.insert(DrawTilemap(tmap_ent));
                
                let (tmap_handles, storage, tmap_hash_id_map) =
                    if let Ok((mut tmap_handles, mut storage, mut tmap_hash_id_map)) = tilemaps.get_mut(tmap_ent) {
                        (tmap_handles.into_inner(), storage.into_inner(), tmap_hash_id_map.into_inner())
                    } else 
                    {
                        changed_structs.insert((ev.chunk, map_key.clone()));
                        let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
                        (tmap_handles, storage, tmap_hash_id_map)
                    };
                let (Vector(tmap_handles)) = tmap_handles else { error!("Failed to get tilemap handles for {:?}", tmap_ent); continue; };



                cmd.entity(tile_ent).try_insert((ChildOf(tmap_ent), TilemapId(tmap_ent)));

                storage.set(&tile_pos, tile_ent);

                let Some(tile_handles) = tile_handles 
                else { continue; };
                
                let mut first_texture_index = None;


                for (id, handle) in tile_handles.iter() {
                    let texture_index = tmap_handles
                        .into_iter()
                        .position(|x| *x == *handle)
                        .map(|i| TileTextureIndex(i as u32))
                        .unwrap_or_else(|| {
                            tmap_handles.push(handle.clone());
                            TileTextureIndex((tmap_handles.len() - 1) as u32)
                        });
                    tmap_hash_id_map.0.insert_with_id(id, texture_index);
                    if first_texture_index.is_none() {
                        first_texture_index = Some(texture_index);
                        //NO HACER BREAK
                    }
                }
                cmd.entity(tile_ent).try_insert(first_texture_index.unwrap_or_default());
                
            
            } else {
                let mut tmap_hash_id_map = TmapHashIdtoTextureIndex::default();
                changed_structs.insert((ev.chunk, map_key.clone()));


                let handles = if let Some(tile_handles) = tile_handles {
                    for (i, (id, _)) in tile_handles.iter().enumerate() {
                        tmap_hash_id_map.0.insert_with_id(id, TileTextureIndex(i as u32));
                    }
                    tile_handles.handles().clone()
                } else {
                    Vec::new()
                };

                let tmap_ent = cmd.spawn((
                    TilemapConfig::new(oplist_size, tile_size),
                    map_key.z_index,
                    ChildOf(ev.chunk),
                ))
                .id();

                // if let Ok(mut chunk) = cmd.get_entity(ev.chunk) {
                //     chunk.add_child(tmap_ent);
                // } else {
                //     cmd.entity(tmap_ent).try_despawn();
                //     cmd.entity(tile_ent).try_despawn();
                //     continue 'eventsfor;
                // }

                to_draw.insert(DrawTilemap(tmap_ent));


                //TODO HACER UN SYSTEM Q BORRE TILEMAPS HUÉRFANOS?
                

                cmd.entity(tile_ent).try_insert((ChildOf(tmap_ent), TilemapId(tmap_ent)));

                let mut storage = TilemapConfig::new_storage(oplist_size);
                storage.set(&tile_pos, tile_ent);
                layers.0.insert(map_key, MapStruct {
                    tmap_ent,
                    texture: TilemapTexture::Vector(handles),
                    storage,
                    tmap_hash_id_map,
                });
            }

        }
   

    }
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
        cmd.entity(tmap_ent).insert((
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
            trace!("Inserting default TilemapBundle for tilemap entity {:?}", tmap_ent);
            cmd.entity(tmap_ent)
            .try_insert(MaterialTilemapHandle::<StandardTilemapMaterial>::default());
        }
    }
        //CLONES PROVISORIOS
    event_writer.write_batch(to_draw);


    Ok(())
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