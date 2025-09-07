use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled, world::OnDespawn}, math::U16Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use common::{common_components::StrId, common_resources::ImageSizeMap};
use game_common::game_common_components::{EntityZero, MyZ};
use ::tilemap_shared::*;

use crate::{chunking_components::*, chunking_resources::AaChunkRangeSettings, terrain_gen::terrgen_events::Tiles2TmapProcess, tile::{tile_components::*, tile_materials::*}, tilemap_components::*};



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
pub fn produce_tilemaps(
    mut cmd: Commands, 
    mut ereader_processed_tiles: EventReader<Tiles2TmapProcess>,
    oritile_query: Query<(&TileStrId, Has<ChunkOrTilemapChild>, Option<&MyZ>, Option<&TileHidsHandles>, Option<&TileShaderRef>, ), 
    (With<Disabled>)>,
    mut tile_comps: Query<(Entity, &TilePos, &OplistSize, &EntityZero, &mut TilemapId, &mut TileTextureIndex, &mut TileVisible), (
    (Or<(With<Disabled>, Without<Disabled>)>, Without<Transform>))>,


    mut chunk_query: Query<(&mut LayersMap), ()>,
    mut tilemaps: Query<(&mut TilemapTexture, &mut TileStorage, &mut TmapHashIdtoTextureIndex, ), ( )>,
    image_size_map: Res<ImageSizeMap>,

    mut texture_overlay_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
    mut voronoi_mat: ResMut<Assets<VoronoiTextureOverlayMat>>,
    mut event_writer: EventWriter<DrawTilemap>,
    chunkrange: Res<AaChunkRangeSettings>,

    shader_query: Query<(&TileShader, ), ( )>,
) -> Result {

    if ereader_processed_tiles.is_empty() { return Ok(()); }

    let reserved = chunkrange.approximate_number_of_chunks(0.06);

    let mut changed_structs: HashSet<(Entity, MapKey)> = HashSet::with_capacity(reserved);

    trace!("Producing tilemaps for {} tile events, reserving space for {} chunks", ereader_processed_tiles.len(), reserved);


    let mut to_draw = Vec::new();

    let mut tilemap_bundles = Vec::new();//TODO HACER ALGO CON EL CHILDOF (CAMBIAR POR OTRO STRUCT?)


    #[allow(unused_mut)]
    'eventsfor: for ev in ereader_processed_tiles.read() {
        trace!("Processing event for chunk {:?}", ev.chunk);

        let Ok(mut layers) = chunk_query.get_mut(ev.chunk) else {
            continue 'eventsfor;
        };

        'tilefor: for tile_ent in ev.tiles.iter() {
            

            let Ok((tile_ent, &tile_pos, &oplist_size, orig_ref, mut tilemap_id, mut tile_texture_index, mut tile_visible)) = 
            tile_comps.get_mut(tile_ent)
            else {
                continue 'tilefor;
            };

            let Ok((tile_strid, is_child, tile_z_index, tile_handles, shader_ref, )) = oritile_query.get(orig_ref.0) else{
                error!("Original tile entity {} is despawned", orig_ref.0);
                continue 'tilefor;
            };
            if ! is_child{
                trace!("Original tile entity {} is not a ChunkOrTilemapChild", orig_ref.0);
                continue 'tilefor;
            }



            let tile_z_index = tile_z_index.cloned().unwrap_or_default();

            let tile_size = match tile_handles {
                Some(handles) => (image_size_map.0.get(&handles.first_handle()).copied().unwrap_or(U16Vec2::ONE)),
                None => {
                    tile_visible.0 = false; U16Vec2::ONE
                }
            };

            let map_key = MapKey::new(tile_z_index, oplist_size, tile_size, shader_ref.copied());
            trace!("Changed tilemap {:?} in chunk {:?}", map_key, ev.chunk);

            
            if let Some(mapstruct) = layers.0.get_mut(&map_key) {
                let tmap_ent = mapstruct.tmap_ent;
                if to_draw.iter().all(|e: &DrawTilemap| e.0 != tmap_ent) {
                    to_draw.push(DrawTilemap(tmap_ent));
                }

                let (tmap_handles, storage, tmap_hash_id_map) =
                    if let Ok((mut tmap_handles, mut storage, mut tmap_hash_id_map)) = 
                    tilemaps.get_mut(tmap_ent) {
                        (tmap_handles.into_inner(), storage.into_inner(), tmap_hash_id_map.into_inner())
                    } 
                    else {
                        changed_structs.insert((ev.chunk, map_key.clone()));
                        let MapStruct { texture: tmap_handles, storage, tmap_hash_id_map, .. } = mapstruct;
                        (tmap_handles, storage, tmap_hash_id_map)
                    };
                let (Vector(tmap_handles)) = tmap_handles else 
                { error!("Failed to get tilemap handles for {:?}", tmap_ent); continue; };

                tilemap_id.0 = tmap_ent;

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
                tile_texture_index.0 = first_texture_index.unwrap_or_default().0;
                
            
            } else {
                let mut tmap_hash_id_map = TmapHashIdtoTextureIndex::default();
                changed_structs.insert((ev.chunk, map_key.clone()));

                let handles = if let Some(tile_handles) = tile_handles {
                    for (i, (id, _)) in tile_handles.iter().enumerate() {
                        tmap_hash_id_map.0.insert_with_id(id, TileTextureIndex(i as u32));
                    }
                    tile_handles.handles().clone()
                } else { Vec::new() };

                let tmap_ent = cmd.spawn(ChildOf(ev.chunk)).id();

                tilemap_bundles.push(
                    (tmap_ent, 
                    (
                        TilemapConfig::new(oplist_size, tile_size),
                        map_key.z_index,
                        ChildOf(ev.chunk),
                    )
                ));

                tilemap_id.0 = tmap_ent;

                if to_draw.iter().all(|e: &DrawTilemap| e.0 != tmap_ent) {
                    to_draw.push(DrawTilemap(tmap_ent));
                }

                let mut storage = TilemapConfig::new_storage(oplist_size);
                storage.set(&tile_pos, tile_ent);
                layers.0.insert(map_key, MapStruct {
                    tmap_ent,
                    texture: TilemapTexture::Vector(handles),
                    storage,
                    tmap_hash_id_map,
                });
            }
            cmd.entity(tile_ent).try_remove::<(Disabled, )>();//MOVER ABAJO DESP
        }
    }

    cmd.insert_batch(tilemap_bundles);

    //info!("Producing {} tilemaps", changed_structs.len());
    //info!("Requesting draw for {} tilemaps", to_draw.len());
    //info!("Producing tilemaps for {} changed structs", changed_structs.len());

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
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn assign_child_of(mut cmd: Commands, 
    tile_instances_holder_query: Single<Entity, With<TileInstancesHolder>>,
    mut query: Query<(Entity, ),(Without<ChildOf>, With<TilePos>, With<TileTextureIndex>)>,
) {
    let query_len = query.iter().count();
    let mut child_ofs_for_tiles: Vec<(Entity, ChildOf)> = Vec::with_capacity(query_len);
    let parent = tile_instances_holder_query.into_inner();

    for (tile_ent,) in query.iter_mut() {

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