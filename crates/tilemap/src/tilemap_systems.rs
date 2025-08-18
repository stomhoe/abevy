
use bevy::{ecs::entity_disabling::Disabled, math::U16Vec2, platform::collections::HashMap, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::MaterialTilemapHandle, tiles::*, MaterialTilemapBundle, TilemapBundle};
use common::{common_components::StrId, common_resources::ImageSizeMap};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::MyZ;

use crate::{chunking_components::*, terrain_gen::terrgen_components::OplistSize, tile::{tile_components::*, tile_materials::*}, tilemap_components::*};




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
    mut chunk_query: Query<(Entity, &mut ProducedTiles,), (Without<Children>, Added<TilesReady>, Without<LayersReady>)>,
    mut tile_comps: Query<(Entity, &StrId, &TilePos, &OplistSize, Option<&TileIdsHandles>, Option<&MyZ>, Option<&TileShaderRef>, Option<&mut Transform>, ),(With<Disabled>, With<TilemapChild>, )>,
    image_size_map: Res<ImageSizeMap>,
) -> Result {
    let mut layers: Map = HashMap::with_capacity(10);

    #[allow(unused_mut)]
    'chunkfor: for (chunk_ent, mut produced_tiles) in chunk_query.iter_mut() {
        for &tile_ent in produced_tiles.produced_tiles().iter() {

            let (tile_ent, tile_str_id, &tile_pos, &oplist_size, tile_handles, tile_z_index, shader_ref, transf, ) = tile_comps.get_mut(tile_ent)?;
            trace!("Producing tile {:?} at pos {:?} in chunk {:?}", tile_str_id, tile_pos, chunk_ent);

            let tile_ent = cmd.entity(tile_ent)
                .remove::<TilemapChild>()
                .insert_if_new(TileBundle::default())
                .remove::<Disabled>()
                .id();

            let tile_z_index = tile_z_index.cloned().unwrap_or_default();

            
            if let Some(mut transform) = transf {//TODO USAR EL IMAGE

                let displacement: Vec2 = Vec2::from(tile_pos) * oplist_size.inner().as_vec2() * Tile::PIXELS.as_vec2();
                transform.translation += displacement.extend(0.0);
            
            }

            let (handles, tile_size) = match tile_handles {
                Some(handles) => (handles.clone_handles(), image_size_map.0.get(&handles.first_handle()).copied().unwrap_or(U16Vec2::ONE)),
                None => {
                    cmd.entity(tile_ent).insert(TileVisible(false));
                    (Vec::new(), U16Vec2::ONE)
                }
            };

            let map_key = MapKey::new(tile_z_index, oplist_size, tile_size, shader_ref.copied());

            if let Some(MapStruct { tmap_ent, handles, storage, tmap_hash_id_map }) = layers.get_mut(&map_key) {
                cmd.entity(tile_ent).insert((ChildOf(*tmap_ent), TilemapId(*tmap_ent)));

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
                    }
                }
                cmd.entity(tile_ent).insert(first_texture_index.unwrap_or_default());
                
            
            } else {
                let tmap_ent = cmd.spawn((
                   
                    TilemapConfig::new(oplist_size, tile_size),
                ))
                .id();
                if cmd.get_entity(chunk_ent).is_ok() {
                    cmd.entity(chunk_ent).add_child(tmap_ent);
                }else{
                    cmd.entity(tmap_ent).try_despawn();
                    continue 'chunkfor; 
                }
                
                //TODO HACER UN SYSTEM Q BORRE TILEMAPS HUÉRFANOS?

            
                cmd.entity(tile_ent).try_insert((ChildOf(tmap_ent), TilemapId(tmap_ent)));

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
            for (map_key, MapStruct { tmap_ent, handles, storage, tmap_hash_id_map }) in layers.drain() {//TA BIEN DRAIN
                cmd.entity(tmap_ent).try_insert((map_key.z_index, handles, storage, tmap_hash_id_map));

                if let Some(shader_ref) = map_key.shader_ref {
                    cmd.entity(tmap_ent).try_insert(shader_ref);
                }
            }
            cmd.entity(chunk_ent).try_remove::<ProducedTiles>().try_insert(LayersReady);
        }
    }
    Ok(())
}


#[allow(unused_parens)]
pub fn fill_tilemaps_data(
    mut commands: Commands,
    mut chunk_query: Query<(Entity, &Children), (Added<LayersReady>)>,
    mut tilemaps_query: Query<(Entity, &mut TileMapHandles, Option<&TileShaderRef>), (With<ChildOf>, )>,
    mut texture_overley_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
    shader_query: Query<(&TileShader, ), ( )>,
) {
    for (chunk, children) in chunk_query.iter_mut() {
        let _now = std::time::Instant::now();
        commands.entity(chunk).try_remove::<LayersReady>();

        for child in children.iter() {
            if let Ok((tmap_entity, mut handles, shader_ref)) = tilemaps_query.get_mut(child) {
                commands.entity(tmap_entity).try_remove::<TileMapHandles>();

                let shader = shader_ref
                    .and_then(|shader_ref| shader_query.get(shader_ref.0).ok())
                    .map(|(shader,)| shader.clone());
                let texture = TilemapTexture::Vector(handles.take_handles());

                if commands.get_entity(chunk).is_err() {
                    break;
                }

                if let Some(shader) = shader {
                    let material = match shader {
                        TileShader::TexRepeat(handle) => {
                            MaterialTilemapHandle::from(texture_overley_mat.add(handle.clone()))
                        }
                        TileShader::TwoTexRepeat(handle) => todo!(),
                    };

                    commands.entity(tmap_entity).try_insert_if_new(MaterialTilemapBundle {
                        texture,
                        material,
                        ..Default::default()
                    });
                } else {
                    commands.entity(tmap_entity)
                    .try_insert_if_new((TilemapBundle { texture, ..Default::default() }));
                }
            }
        }

        commands.entity(chunk).try_insert(InitializedChunk);
        
        trace!("Filled tilemaps data in {:?}", _now.elapsed());
    }
}


