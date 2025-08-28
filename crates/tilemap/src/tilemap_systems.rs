use bevy::{ecs::{entity_disabling::Disabled, world::OnDespawn}, math::U16Vec2, platform::collections::HashMap, prelude::*};
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::*, prelude::MaterialTilemapHandle, tiles::*, MaterialTilemapBundle, TilemapBundle};
use common::{common_components::StrId, common_resources::ImageSizeMap};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::MyZ;
use tilemap_shared::GlobalTilePos;

use crate::{chunking_components::*, terrain_gen::terrgen_oplist_components::OplistSize, tile::{tile_components::*, tile_materials::*}, tilemap_components::*};




#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MapKey {z_index: MyZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>,}
impl MapKey {
    pub fn new(z_index: MyZ, oplist_size: OplistSize, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>) -> Self {
        Self { z_index, oplist_size, tile_size, shader_ref }
    }
}

struct MapStruct{
    pub tmap_ent: Entity,
    pub handles: TileMapHandles,
    pub storage: TileStorage,
    pub tmap_hash_id_map: TmapHashIdtoTextureIndex,
}

type Map = HashMap<MapKey, MapStruct>;

#[allow(unused_parens)]
pub fn produce_tilemaps(
    mut cmd: Commands, 
    chunk_query: Query<(Entity, &ProducedTiles,), (With<TilesReady>)>,
    tile_comps: Query<(Entity, &TilePos, &OplistSize, Option<&TileIdsHandles>, Option<&MyZ>, Option<&TileShaderRef>, ), (With<Disabled>, With<TilemapChild>, Without<Transform>)>,
    image_size_map: Res<ImageSizeMap>,
) -> Result {
    let mut layers: Map = HashMap::with_capacity(10);

    #[allow(unused_mut)]
    'chunkfor: for (chunk_ent, produced_tiles) in chunk_query.iter() {

        'tilefor: for &tile_ent in produced_tiles.produced_tiles().iter() {

            let Ok((tile_ent, &tile_pos, &oplist_size, tile_handles, tile_z_index, shader_ref, )) = tile_comps.get(tile_ent)
            else { 
                continue 'tilefor;
            };


            cmd.entity(tile_ent)
            .try_insert_if_new(TileBundle::default())
            .try_remove::<(TilemapChild, Disabled)>();

            let tile_z_index = tile_z_index.cloned().unwrap_or_default();

            
            

            let (handles, tile_size) = match tile_handles {
                Some(handles) => (handles.clone_handles(), image_size_map.0.get(&handles.first_handle()).copied().unwrap_or(U16Vec2::ONE)),
                None => {
                    cmd.entity(tile_ent).try_insert(TileVisible(false));
                    (Vec::new(), U16Vec2::ONE)
                }
            };

            let map_key = MapKey::new(tile_z_index, oplist_size, tile_size, shader_ref.copied());

            if let Some(MapStruct { tmap_ent, handles, storage, tmap_hash_id_map }) = layers.get_mut(&map_key) {
                cmd.entity(tile_ent).try_insert((ChildOf(*tmap_ent), TilemapId(*tmap_ent)));

                storage.set(&tile_pos, tile_ent);

                let Some(tile_handles) = tile_handles 
                else { continue; };
                
                let mut first_texture_index = None;
                for (id, handle) in tile_handles.iter() {
                    let texture_index = handles
                        .into_iter()
                        .position(|x| *x == *handle)
                        .map(|i| TileTextureIndex(i as u32))
                        .unwrap_or_else(|| {
                            handles.push_handle(handle.clone());
                            TileTextureIndex((handles.len() - 1) as u32)
                        });
                    tmap_hash_id_map.0.insert_with_id(id, texture_index);
                    if first_texture_index.is_none() {
                        first_texture_index = Some(texture_index);
                        //NO HACER BREAK
                    }
                }
                cmd.entity(tile_ent).try_insert(first_texture_index.unwrap_or_default());
                
            
            } else {
                let tmap_ent = cmd.spawn((
                   
                    TilemapConfig::new(oplist_size, tile_size),
                    ChildOf(chunk_ent),
                ))
                .id();
                
                //TODO HACER UN SYSTEM Q BORRE TILEMAPS HUÉRFANOS?
                
                
                cmd.entity(tile_ent).try_insert((TilemapId(tmap_ent)));
                
                
                let mut storage = TilemapConfig::new_storage(oplist_size);
                storage.set(&tile_pos, tile_ent);
                layers.insert(map_key, MapStruct {
                    tmap_ent,
                    handles: TileMapHandles::new(handles),
                        storage,
                    tmap_hash_id_map: TmapHashIdtoTextureIndex::default(),
                });
            }
            if cmd.get_entity(chunk_ent).is_err() { continue 'chunkfor; }

        }
        if layers.is_empty() {
            warn!(target:"tilemap", "No tiles produced for chunk {:?}", chunk_ent);
            cmd.entity(chunk_ent).try_insert(InitializedChunk);

        } else{
            let layers_size = layers.len();
            cmd.entity(chunk_ent).try_insert(PendingTilemaps(layers_size as i32));
            for (map_key, MapStruct { tmap_ent, handles, storage, tmap_hash_id_map }) in layers.drain() {//TA BIEN DRAIN
                if let Some(shader_ref) = map_key.shader_ref {
                    cmd.entity(tmap_ent).try_insert(shader_ref);
                }
                cmd.entity(tmap_ent).try_insert((map_key.z_index, handles, storage, tmap_hash_id_map, ));

            }
        }
    }
    Ok(())
}


#[allow(unused_parens)]
pub fn fill_tilemaps_data(
    mut cmd: Commands,
    mut tilemaps_query: Query<(Entity, &mut TileMapHandles, Option<&TileShaderRef>, &ChildOf), (Without<TilemapAnchor>, )>,
    mut chunk_query: Query<(Entity, &mut PendingTilemaps), >,
    mut texture_overlay_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
    mut voronoi_mat: ResMut<Assets<VoronoiTextureOverlayMat>>,
    shader_query: Query<(&TileShader, ), ( )>,
) {
        let _now = std::time::Instant::now();

        for (tmap_entity, mut handles, shader_ref, child_of_chunk) in tilemaps_query.iter_mut() {

            cmd.entity(tmap_entity).try_remove::<(TileMapHandles, )>();

            let shader = if let Some(shader_ref) = shader_ref {
                shader_query.get(shader_ref.0).ok().map(|(shader,)| shader.clone())
            } else {
                None
            };
            let texture = TilemapTexture::Vector(handles.take_handles());

            if let Some(shader) = shader {
                info!("Inserting tmapshader {:?} for tilemap entity {:?}", shader, tmap_entity);
                match shader {
                    TileShader::TexRepeat(handle) => {
                        cmd.entity(tmap_entity).try_insert_if_new(MaterialTilemapBundle {
                            texture,
                            material: MaterialTilemapHandle::from(texture_overlay_mat.add(handle.clone())),
                            ..Default::default()
                        });
                    }
                    TileShader::Voronoi(handle) => {
                        cmd.entity(tmap_entity).try_insert_if_new(MaterialTilemapBundle {
                            texture,
                            material: MaterialTilemapHandle::from(voronoi_mat.add(handle.clone())),
                            ..Default::default()
                        });
                    }
                    TileShader::TwoTexRepeat(handle) => todo!(),
                };

            } else {
                info!("Inserting default TilemapBundle for tilemap entity {:?}", tmap_entity);
                cmd.entity(tmap_entity)
                .try_insert_if_new((TilemapBundle { texture, ..Default::default() }));
            }

            let Ok((chunk_ent, mut pending)) = chunk_query.get_mut(child_of_chunk.parent()) else {
                continue;
            };
            pending.0 -= 1;
            if pending.0 <= 0 {
                cmd.entity(chunk_ent).try_remove::<PendingTilemaps>()
                .try_insert(InitializedChunk);
            }
        }

        
        trace!("Filled tilemaps data in {:?}", _now.elapsed());
}





#[allow(unused_parens)]
pub fn despawn_orphan_tilemaps(mut cmd: Commands, 
    mut query: Query<(Entity, ), (Without<ChildOf>, With<TilemapGridSize>)>,
) {
    for (tilemap_ent, ) in query.iter_mut() {
        cmd.entity(tilemap_ent).despawn();
    }
}