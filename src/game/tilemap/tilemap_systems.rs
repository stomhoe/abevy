use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::*, tiles::*, TilemapBundle};
use rand::rand_core::le;

use crate::game::{beings::beings_components::Being, factions::factions_components::SelfFaction, tilemap::{tilemap_components::*, tilemap_resources::*}};



const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 64.0, y: 64.0 };

//TODO HACER DEL TAMAÑO DE LO QUE ES VISIBLE EN PANTALLA
const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };


const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

const CNT: i32 = 3;

pub fn visit_chunks_around_activators(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut query: Query<(&Transform, &mut ActivatesChunks), (With<SelfFaction>)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for (transform, mut activates_chunks) in query.iter_mut() {
        let center_chunk_pos = contpos_to_chunk_pos(&transform.translation.xy());

        for y in (center_chunk_pos.y - CNT)..(center_chunk_pos.y + CNT+1) {
            for x in (center_chunk_pos.x - CNT)..(center_chunk_pos.x + CNT+1) {

                let adjacent_chunk_pos = IVec2::new(x, y);

                if ! chunk_manager.loaded_chunks.contains_key(&adjacent_chunk_pos) {
                    let chunk_ent: Entity = spawn_single_chunk(&mut commands, &asset_server, adjacent_chunk_pos);
                    chunk_manager.loaded_chunks.insert(adjacent_chunk_pos, chunk_ent);
                    activates_chunks.0.insert(chunk_ent);
                }
                else {
                    activates_chunks.0.insert(
                        chunk_manager.loaded_chunks[&adjacent_chunk_pos]
                    );
                }
            }
        }
    }
}

pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&Transform, &mut ActivatesChunks), (With<SelfFaction>)>,
    mut chunks_query: Query<(Entity, &Transform), With<Chunk>>,
    tilemap_settings: Res<TilemapSettings>,
) {
    for (transform, mut activate_chunks) in activator_query.iter_mut() {
        for (entity, chunk_transform) in chunks_query.iter_mut() {
            let chunk_cont_pos = chunk_transform.translation.xy();
            let distance = transform.translation.xy().distance(chunk_cont_pos);
            
            if distance > tilemap_settings.chunk_active_min_dist {
              //  println!("Removing chunk at position: {:?}", contpos_to_chunk_pos(&chunk_cont_pos));
                
                activate_chunks.0.remove(&entity);
            }
        }
    }
}

pub fn despawn_unreferenced_chunks(
    mut commands: Commands,
    activator_query: Query<(&ActivatesChunks), (With<SelfFaction>)>,
    mut chunks_query: Query<(Entity, &Transform), With<Chunk>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {

    let mut referenced_chunks: HashSet<Entity> = HashSet::new();

    for (activates_chunks) in activator_query.iter() {
        for chunk_entity in activates_chunks.0.iter() {
            referenced_chunks.insert(*chunk_entity);
        }
    }

    for (entity, chunk_transform) in chunks_query.iter_mut() {
        if !referenced_chunks.contains(&entity) {
            let chunk_cont_pos = chunk_transform.translation.xy();
            let chunk_pos = contpos_to_chunk_pos(&chunk_cont_pos);

            //println!("Despawning chunk at position: {:?}", chunk_pos);
            chunk_manager.loaded_chunks.remove(&chunk_pos);
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_single_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) 
-> Entity
{
    let chunk_tilemap_layer_entity: Entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
    
    let corner_cont_pos = chunk_pos_to_contpos(&chunk_pos);

    // SPAWNEO DE CADA TILE <---
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_cont_pos = Vec2::new(
                corner_cont_pos.x + (x as f32 * TILE_SIZE.x),
                corner_cont_pos.y + (y as f32 * TILE_SIZE.y),
            );
            //hacer lo del mod para extraer la textura

            let tile_pos: TilePos = TilePos { x, y };
            let tile_entity: Entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(chunk_tilemap_layer_entity),
                    ..Default::default()
                })
                .id();
            commands.entity(chunk_tilemap_layer_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
//            println!("Spawning tile at position: {:?}", tile_pos);

        }
    }

    let chunk_transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
        0.0,
    ));
    let texture_handle: Handle<Image> = asset_server.load("textures/world/bushes/bush.png");
    commands.entity(chunk_tilemap_layer_entity).insert((
        TilemapBundle {
                grid_size: TILE_SIZE.into(),
                size: CHUNK_SIZE.into(),
                storage: tile_storage,
                texture: TilemapTexture::Single(texture_handle),
                tile_size: TILE_SIZE,
                transform: chunk_transform,
                render_settings: TilemapRenderSettings {
                    render_chunk_size: RENDER_CHUNK_SIZE,
                    ..Default::default()
                },
                ..Default::default()
            },
        Chunk{},
    ));
    chunk_tilemap_layer_entity
}

#[allow(unused_parens)]
pub fn show_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<&mut Visibility, (With<Chunk>)>,
    chunk_manager: Res<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = contpos_to_chunk_pos(&transform.translation.xy());
        for y in (camera_chunk_pos.y - CNT)..(camera_chunk_pos.y + CNT+1) {
            for x in (camera_chunk_pos.x - CNT)..(camera_chunk_pos.x + CNT+1) {
                
                let chunk_pos = IVec2::new(x, y);

                chunk_manager.loaded_chunks.get(&chunk_pos).map(|ent| {
                    if let Ok(mut visibility) = chunks_query.get_mut(*ent) {
                        *visibility = Visibility::Visible;
  //                      println!("Showing chunk at position: {:?}", chunk_pos);
                    }
                });
            }
        }
    }
}

pub fn hide_outofrange_chunks(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<(&Transform, &mut Visibility), With<Chunk>>,
    tilemap_settings: Res<TilemapSettings>,
) {
    for camera_transform in camera_query.iter() {
        for (chunk_transform, mut visibility) in chunks_query.iter_mut() {
            let chunk_cont_pos = chunk_transform.translation.xy();
           // println!("Chunk cont position: {:?}", (&chunk_cont_pos));
            //println!("Camera cont position: {:?}", camera_transform.translation.xy());

            let distance = camera_transform.translation.xy().distance(chunk_cont_pos);
            
            if distance > tilemap_settings.chunk_visib_min_dist {
                *visibility = Visibility::Hidden;//PARA DEBUGGEAR, HACER Q RECOLORICE LA TILE
              //  println!("Hiding chunk at position: {:?}", contpos_to_chunk_pos(&chunk_cont_pos));
            }
        }
    }
}





// PARA Q LA TILE HAGA ALGO, 

fn contpos_to_chunk_pos(contpos: &Vec2) -> IVec2 {
    let camera_pos = contpos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn chunk_pos_to_contpos(chunk_pos: &IVec2) -> Vec2 {
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    Vec2::new(
        (chunk_pos.x * chunk_size.x * tile_size.x) as f32,
        (chunk_pos.y * chunk_size.y * tile_size.y) as f32,
    )
}

// pub fn access_tile(world_contpos: &Vec2, mut chunk_manager: Res<ChunkManager>) -> Option<Entity> {
//     let chunk_pos = contpos_to_chunk_pos(world_contpos);
//     if chunk_manager.spawned_chunks.contains(&chunk_pos) {
//         let tile_pos = TilePos {
//             x: (world_contpos.x / TILE_SIZE.x) as u32,
//             y: (world_contpos.y / TILE_SIZE.y) as u32,
//         };
//         let chunk_entity = TilemapId(chunk_pos_to_entity(chunk_pos));
//         let tile_storage = world_contpos.get_tile_storage(chunk_entity);
//         tile_storage.checked_get(&tile_pos)
//     } else {
//         None
//     }

// }


