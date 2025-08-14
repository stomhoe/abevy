
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
    mut tile_comps: Query<(Entity, &StrId, &TilePos, &OplistSize, Option<&TileIdsHandles>, Option<&MyZ>, Option<&TileShaderRef>, Option<&mut Transform>),(With<Disabled>)>,
    image_size_map: Res<ImageSizeMap>,
) -> Result {
    let mut layers: Map = HashMap::with_capacity(10);

    #[allow(unused_mut)]
    for (chunk_ent, mut produced_tiles) in chunk_query.iter_mut() {
        for &tile_ent in produced_tiles.produced_tiles().iter() {

            let (tile_ent, tile_str_id, &tile_pos, &oplist_size, tile_handles, tile_z_index, shader_ref, transf) = tile_comps.get_mut(tile_ent)?;
            trace!(target: "tilemap", "Producing tile {:?} at pos {:?} in chunk {:?}", tile_str_id, tile_pos, chunk_ent);

            let tile_ent = cmd.entity(tile_ent)
                .remove::<Disabled>()
                .insert_if_new(TileBundle::default())
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
                    Tilemap, ChildOf(chunk_ent),
                    TilemapTileSize::from(tile_size.as_vec2()) ,
                    TilemapGridSize::from(Tile::PIXELS.as_vec2()*oplist_size.inner().as_vec2()),
                    TilemapSize::from(ChunkInitState::SIZE/oplist_size.inner()),
                    TilemapRenderSettings {render_chunk_size: ChunkInitState::SIZE*2/oplist_size.inner(), y_sort: false},
                ))
                .id();
            
                cmd.entity(tile_ent).insert((ChildOf(tmap_ent), TilemapId(tmap_ent)));

                let mut storage = TileStorage::empty((ChunkInitState::SIZE/oplist_size.inner()).into());
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
            warn!(target:"tilemap", "No tiles produced for chunk {:?}", chunk_ent);
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
                            TileShader::TexRepeat(handle) => {
                                MaterialTilemapHandle::from(texture_overley_mat.add(
                                    handle.clone()
                                ))
    
                            },
                            TileShader::TwoTexRepeat(handle) => todo!(),
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


