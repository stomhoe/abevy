
use bevy::{platform::collections::HashSet, prelude::*};

use crate::game::{faction::faction_components::BelongsToSelfPlayerFaction, tilemap::{chunking_components::*, chunking_resources::*}};

#[allow(unused_parens, )]
pub fn visit_chunks_around_activators(
    mut commands: Commands, 
    mut query: Query<(&Transform, &mut ActivatesChunks), (With<BelongsToSelfPlayerFaction>)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    let cnt = tilemap_settings.chunk_show_range as i32;   
    for (transform, mut activates_chunks) in query.iter_mut() {

        let center_chunk_pos = ChunkPos::from(transform.translation.xy());

        for y in (center_chunk_pos.y() - cnt)..(center_chunk_pos.y() + cnt+1) {
            for x in (center_chunk_pos.x() - cnt)..(center_chunk_pos.x() + cnt+1) {

                let chunk_pos = ChunkPos::new(x, y);

                if ! loaded_chunks.0.contains_key(&chunk_pos) {

                    let chunk_ent = commands.spawn((
                        Name::new(format!("Chunk ({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        UninitializedChunk,
                        Transform::from_translation((chunk_pos.to_pixelpos()).extend(0.0)),
                        chunk_pos,
                    )).id();
                    activates_chunks.0.insert(chunk_ent);
                    loaded_chunks.0.insert(chunk_pos, chunk_ent);
                }
                else {
                    activates_chunks.0.insert(loaded_chunks.0[&chunk_pos]);
                }
            }
        }
    }
}
#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&Transform, &mut ActivatesChunks), (With<BelongsToSelfPlayerFaction>)>,
    mut chunks_query: Query<(Entity, &ChunkPos ,&Transform), With<InitializedChunk>>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    for (act_transform, mut activate_chunks) in activator_query.iter_mut() {

        let act_chunk_pos = ChunkPos::from(act_transform.translation.xy());

        for (entity, &chunk_pos, chunk_transform) in chunks_query.iter_mut() {
            let chunk_cont_pos = chunk_transform.translation.xy();
            let distance = act_transform.translation.xy().distance(chunk_cont_pos);
            
            let show_range = tilemap_settings.chunk_show_range as u32;
            let chunk_delta: UVec2 = (act_chunk_pos - chunk_pos).0.abs().as_uvec2();
            if distance > tilemap_settings.chunk_active_max_dist && (chunk_delta.x > show_range || chunk_delta.y > show_range) {
                activate_chunks.0.remove(&entity);
            }
        }
    }
}
#[allow(unused_parens)]
pub fn despawn_unreferenced_chunks(
    mut commands: Commands,
    activator_query: Query<(&ActivatesChunks), (With<BelongsToSelfPlayerFaction>)>,
    mut chunks_query: Query<(Entity, &Transform,), With<InitializedChunk>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {

    for (entity, chunk_transform) in chunks_query.iter_mut() {
        let referenced = activator_query.iter().any(|activates_chunks| activates_chunks.0.contains(&entity));
        
        if !referenced {

            let chunk_pos = ChunkPos::from(chunk_transform.translation.xy());

            loaded_chunks.0.remove(&chunk_pos);
            commands.entity(entity).remove::<ChunkInitState>().despawn();
        }
    }
}

#[allow(unused_parens)]
pub fn show_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<&mut Visibility, (With<InitializedChunk>)>,
    loaded_chunks: Res<LoadedChunks>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    let cnt = tilemap_settings.chunk_show_range as i32;   
    for transform in camera_query.iter() {
        let camera_chunk_pos = ChunkPos::from(transform.translation.xy());
        for y in (camera_chunk_pos.y() - cnt)..(camera_chunk_pos.y() + cnt+1) {
            for x in (camera_chunk_pos.x() - cnt)..(camera_chunk_pos.x() + cnt+1) {

                let adj_chunk_pos = ChunkPos::new(x, y);

                loaded_chunks.0.get(&adj_chunk_pos).map(|ent| {
                    if let Ok(mut visibility) = chunks_query.get_mut(*ent) {
                        *visibility = Visibility::Visible;
                    }
                });
            }
        }
    }
}

pub fn hide_outofrange_chunks(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<(&Transform, &mut Visibility), With<InitializedChunk>>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    for camera_transform in camera_query.iter() {
        for (chunk_transform, mut visibility) in chunks_query.iter_mut() {
            let chunk_cont_pos = chunk_transform.translation.xy();

            let distance = camera_transform.translation.xy().distance(chunk_cont_pos);
            
            if distance > tilemap_settings.chunk_visib_max_dist {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
