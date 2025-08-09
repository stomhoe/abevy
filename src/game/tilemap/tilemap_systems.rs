
use bevy::{ecs::entity_disabling::Disabled, math::U16Vec2, platform::collections::HashMap, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::MaterialTilemapHandle, tiles::*, MaterialTilemapBundle, TilemapBundle};
use debug_unwraps::DebugUnwrapExt;

use crate::{common::common_components::MyZ, game::{game_resources::ImageSizeMap, tilemap::{chunking_components::*, chunking_resources::*, terrain_gen::terrain_materials::MonoRepeatTextureOverlayMat, tile::{tile_components::{TileShader, TileShaderRef}, tile_utils::TILE_SIZE_PXS, }, tilemap_components::*,}}};



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MapKey {z_index: MyZ, tile_size: U16Vec2, shader_ref: Option<TileShaderRef>,}
impl MapKey {
    pub fn new(z_index: MyZ, size: U16Vec2, shader_ref: Option<TileShaderRef>) -> Self {
        Self { z_index, tile_size: size, shader_ref }
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
    mut tile_comps: Query<(Entity, &TilePos, Option<&TileIdsHandles>, Option<&MyZ>, Option<&TileShaderRef>, Option<&mut Transform>),(With<Disabled>)>,
    image_size_map: Res<ImageSizeMap>,
) -> Result {
    let mut layers: Map = HashMap::with_capacity(10);

    #[allow(unused_mut)]
    for (chunk_ent, mut produced_tiles) in chunk_query.iter_mut() {
        for &tile_ent in produced_tiles.produced_tiles().iter() {

            let (tile_ent, tile_pos, tile_handles, tile_z_index, shader_ref, transf) = tile_comps.get_mut(tile_ent)?;
            let tile_ent = cmd.entity(tile_ent).clone_and_spawn()
                .remove::<Disabled>()
                .insert_if_new(TileBundle::default())
                .id();

            let tile_z_index = tile_z_index.cloned().unwrap_or_default();

            if let Some(mut transform) = transf {
                transform.translation.x += (tile_pos.x as f32 * TILE_SIZE_PXS.x as f32); 
                transform.translation.y += (tile_pos.y as f32 * TILE_SIZE_PXS.y as f32);
            }

            let (handles, tile_size) = match tile_handles {
                Some(handles) => (handles.clone_handles(), image_size_map.0.get(&handles.first_handle()).copied().unwrap_or(U16Vec2::ONE)),
                None => {
                    cmd.entity(tile_ent).insert(TileVisible(false));
                    (Vec::new(), U16Vec2::ONE)
                }
            };

            let map_key = MapKey::new(tile_z_index, tile_size, shader_ref.copied());

            if let Some(MapStruct { tmap_ent, handles, storage, tmap_hash_id_map }) = layers.get_mut(&map_key) {
                cmd.entity(tile_ent).insert((ChildOf(*tmap_ent), TilemapId(*tmap_ent)));

                storage.set(&tile_pos, tile_ent);

                 let Some(tile_handles) = tile_handles else { continue; };
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
                    Tilemap, ChildOf(chunk_ent),
                    TilemapTileSize { x: tile_size.x as f32, y: tile_size.y as f32 },
                ))
                .id();
            
                cmd.entity(tile_ent).insert((ChildOf(tmap_ent), TilemapId(tmap_ent)));
            
                let mut storage = TileStorage::empty(CHUNK_SIZE.into());
                storage.set(&tile_pos, tile_ent);
                layers.insert(map_key, MapStruct {
                    tmap_ent,
                    handles: TileMapHandles::new(handles),
                    storage,
                    tmap_hash_id_map: TmapHashIdtoTextureIndex::default(),
                });
            }
        }
        if layers.is_empty() {
            //warn!("No tiles produced for chunk {:?}", chunk_ent);
            cmd.entity(chunk_ent).insert(InitializedChunk);

        } else{
            for (map_key, MapStruct { tmap_ent, handles, storage, tmap_hash_id_map }) in layers.drain() {//TA BIEN DRAIN
                cmd.entity(tmap_ent).insert((map_key.z_index, handles, storage, tmap_hash_id_map));

                if let Some(shader_ref) = map_key.shader_ref {
                    cmd.entity(tmap_ent).insert(shader_ref);
                }
            }
            cmd.entity(chunk_ent).remove::<ProducedTiles>().insert(LayersReady);
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
        commands.entity(chunk).remove::<LayersReady>();

        children.iter().for_each(|child| {
            if let Ok((tmap_entity, mut handles, shader_ref)) = tilemaps_query.get_mut(child) {//DEJAR CON IF LET
    
                commands.entity(tmap_entity).remove::<TileMapHandles>();
    
                let shader = shader_ref.and_then(|shader_ref| shader_query.get(shader_ref.0).ok()).map(|(shader,)| shader.clone());
                let texture = TilemapTexture::Vector(handles.take_handles());
    
                if let Some(shader) = shader {
                    let material = 
                        match shader {
                            TileShader::TexRepeat(rep_texture) => {
                                MaterialTilemapHandle::from(texture_overley_mat.add(
                                    MonoRepeatTextureOverlayMat {
                                        texture_overlay: rep_texture.cloned_handle(),
                                        scale: rep_texture.scale_div_1e9(),
                                        mask_color: rep_texture.mask_color(),
                                    }
                                ))
    
                            },
                            TileShader::TwoTexRepeat(first_texture, second) => todo!(),
                        };
    
                    commands.entity(tmap_entity).insert_if_new(MaterialTilemapBundle{
                        texture, material,
                        ..Default::default()
                    });
                } 
                else{ commands.entity(tmap_entity).insert_if_new((TilemapBundle{texture, ..Default::default()})); }
            } 
        });

        commands.entity(chunk).insert(InitializedChunk);
        
        info!(target: "tilemap", "Filled tilemaps data in {:?}", _now.elapsed());
    }
}


