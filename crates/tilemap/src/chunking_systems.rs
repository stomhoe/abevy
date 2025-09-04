
use bevy::prelude::*;
use camera::camera_components::CameraTarget;
use dimension_shared::DimensionRef
;
use tilemap_shared::ChunkPos;

use crate::{chunking_components::*, chunking_resources::*};



#[allow(unused_parens, )]
pub fn visit_chunks_around_activators(
    mut commands: Commands, 
    mut query: Query<(&Transform, &mut ActivatingChunks, &DimensionRef), >,//TODO HACER Q HAY ACTIVATO
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
) {
    let cnt = tilemap_settings.chunk_show_range as i32;   
    for (transform, mut activates_chunks, &dimension_ref) in query.iter_mut() {

        let center_chunk_pos = ChunkPos::from(transform.translation.xy());

        for y in (center_chunk_pos.y() - cnt)..(center_chunk_pos.y() + cnt+1) {
            for x in (center_chunk_pos.x() - cnt)..(center_chunk_pos.x() + cnt+1) {

                let chunk_pos = ChunkPos::new(x, y);

                if ! loaded_chunks.0.contains_key(&(dimension_ref, chunk_pos)) {

                    let chunk_ent = commands.spawn_empty().id();
                    activates_chunks.0.insert(chunk_ent);
                    commands.entity(chunk_ent).insert((
                        Chunk,
                        Name::new(format!("Chunk ({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        Transform::from_translation((chunk_pos.to_pixelpos()).extend(0.0)),
                        chunk_pos,
                        ChildOf(dimension_ref.0)

                    ));
                    loaded_chunks.0.insert((dimension_ref, chunk_pos), chunk_ent);
                }
                else {
                    activates_chunks.0.insert(loaded_chunks.0[&(dimension_ref, chunk_pos)]);
                }
            }
        }
    }
}
#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&DimensionRef, &Transform, &mut ActivatingChunks, ), >,
    mut chunks_query: Query<(&ChildOf, Entity, &ChunkPos, &Transform, ), >,
    tilemap_settings: Res<AaChunkRangeSettings>,
) {
    for (dimension_ref, act_transform, mut activate_chunks) in activator_query.iter_mut() {

        let act_chunk_pos = ChunkPos::from(act_transform.translation.xy());

        for (chunk_dimension_ref, entity, &chunk_pos, chunk_transform) in chunks_query.iter_mut() {
            if chunk_dimension_ref.parent() != dimension_ref.0 {
                activate_chunks.0.remove(&entity);
                continue;
            }

            let chunk_cont_pos = chunk_transform.translation.xy();
            let distance = act_transform.translation.xy().distance(chunk_cont_pos);
            
            let show_range = tilemap_settings.chunk_show_range as u32;
            let chunk_delta: UVec2 = (act_chunk_pos - chunk_pos).0.abs().as_uvec2();
            if distance > tilemap_settings.chunk_active_max_dist && (chunk_delta.x > show_range || chunk_delta.y > show_range) {
                activate_chunks.0.remove(&entity);
                //info!("Removed chunk {:?} (pos: {:?}) from activator", entity, chunk_pos, );
            }
        }
    }
}
//TODO REHACER CON EVENTOS Y PONER UN HASHSET DE ENTITIES EN EL CHUNK

#[allow(unused_parens)]
pub fn despawn_unreferenced_chunks(
    mut commands: Commands,
    activator_query: Query<(&ActivatingChunks, ), >,
    mut chunks_query: Query<(&ChildOf, Entity, &ChunkPos, &Children), With<Chunk>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {

    for (child_of, chunk_ent, &chunk_pos, children) in chunks_query.iter_mut() {
        let referenced = activator_query.iter().any(|(activates_chunks, )| activates_chunks.0.contains(&chunk_ent));
        
        if !referenced {
            trace!("Despawning chunk {:?} at pos: {:?}", chunk_ent, chunk_pos);

            loaded_chunks.0.remove(&(DimensionRef(child_of.parent()), chunk_pos));

        

            for child in children.iter() {//HACE FALTA
                commands.entity(child).try_despawn();
            }
            commands.entity(chunk_ent).try_remove::<Chunk>()
            .try_despawn();//DEJAR EL REMOVE
        }
    }
}
#[allow(unused_parens)]
pub fn show_chunks_around_camera(
    camera_query: Single<(&DimensionRef, &Transform), (With<CameraTarget>)>,
    mut chunks_query: Query<&mut Visibility, (With<Chunk>)>,
    loaded_chunks: Res<LoadedChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
) {
    let (&cam_dimension_ref, camera_transform) = *camera_query;
    let cnt = tilemap_settings.chunk_show_range as i32;
    let camera_chunk_pos = ChunkPos::from(camera_transform.translation.xy());
    for y in (camera_chunk_pos.y() - cnt)..(camera_chunk_pos.y() + cnt + 1) {
        for x in (camera_chunk_pos.x() - cnt)..(camera_chunk_pos.x() + cnt + 1) {

            let adj_chunk_pos = ChunkPos::new(x, y);

            loaded_chunks.0.get(&(cam_dimension_ref, adj_chunk_pos)).map(|ent| {
                if let Ok(mut visibility) = chunks_query.get_mut(*ent) {
                    *visibility = Visibility::Inherited;
                }
            });
            
        }
    }
}

#[allow(unused_parens)]
pub fn hide_outofrange_chunks(
    camera_query: Single<(&Transform), (With<CameraTarget>)>,
    mut chunks_query: Query<(&Transform, &mut Visibility), With<Chunk>>,
    tilemap_settings: Res<AaChunkRangeSettings>,
) {
    let camera_transform = *camera_query;
    for (chunk_transform, mut visibility) in chunks_query.iter_mut() {

        let chunk_cont_pos = chunk_transform.translation.xy();

        let distance = camera_transform.translation.xy().distance(chunk_cont_pos);
        
        if distance > tilemap_settings.chunk_visib_max_dist {
            *visibility = Visibility::Hidden;
        }
    }
}


