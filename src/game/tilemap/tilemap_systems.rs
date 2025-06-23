use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::*, tiles::*, MaterialTilemapBundle, TilemapBundle};
use rand::rand_core::le;
use strum::IntoEnumIterator;

use crate::game::{beings::beings_components::Being, factions::factions_components::SelfFaction, tilemap::{formation_generation::formation_generation_utils::BaseZLevels, tilemap_components::*, tilemap_resources::*}};




pub fn visit_chunks_around_activators(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut query: Query<(&Transform, &mut ActivatesChunks), (With<SelfFaction>)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<TilemapSettings>,
) {
    for (transform, mut activates_chunks) in query.iter_mut() {
        let center_chunk_pos = contpos_to_chunkpos(transform.translation.xy());

        for y in (center_chunk_pos.y - CNT)..(center_chunk_pos.y + CNT+1) {
            for x in (center_chunk_pos.x - CNT)..(center_chunk_pos.x + CNT+1) {

                let adjacent_chunk_pos = IVec2::new(x, y);

                if ! loaded_chunks.0.contains_key(&adjacent_chunk_pos) {
                    let chunk_id = commands.spawn((Chunk(adjacent_chunk_pos), Visibility::Hidden, )).id();

                    loaded_chunks.0.insert(adjacent_chunk_pos, chunk_id);
                    activates_chunks.0.insert(chunk_id);
                }
                else {
                    activates_chunks.0.insert(loaded_chunks.0[&adjacent_chunk_pos]);
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
    mut loaded_chunks: ResMut<LoadedChunks>,
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
            let chunk_pos = contpos_to_chunkpos(chunk_cont_pos);

            //println!("Despawning chunk at position: {:?}", chunk_pos);
            loaded_chunks.0.remove(&chunk_pos);
            commands.entity(entity).despawn();
        }
    }
}

#[allow(unused_parens)]
pub fn show_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<&mut Visibility, (With<Chunk>)>,
    loaded_chunks: Res<LoadedChunks>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = contpos_to_chunkpos(transform.translation.xy());
        for y in (camera_chunk_pos.y - CNT)..(camera_chunk_pos.y + CNT+1) {
            for x in (camera_chunk_pos.x - CNT)..(camera_chunk_pos.x + CNT+1) {
                
                let chunk_pos = IVec2::new(x, y);

                loaded_chunks.0.get(&chunk_pos).map(|ent| {
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

            let distance = camera_transform.translation.xy().distance(chunk_cont_pos);
            
            if distance > tilemap_settings.chunk_visib_min_dist {
                *visibility = Visibility::Hidden;//PARA DEBUGGEAR HACER Q RECOLORICE LA TILE
            }
        }
    }
}