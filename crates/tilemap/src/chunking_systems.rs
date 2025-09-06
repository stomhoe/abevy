
use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, tiles::TileStorage};
use camera::camera_components::CameraTarget;
use dimension_shared::DimensionRef
;
use tilemap_shared::ChunkPos;

use crate::{chunking_components::*, chunking_resources::*, tile::tile_events::SavedTileHadChunkDespawn};



#[allow(unused_parens, )]
pub fn visit_chunks_around_activators(
    mut cmd: Commands, 
    mut query: Query<(&GlobalTransform, &mut ActivatingChunks, &DimensionRef), >,//TODO HACER Q HAY ACTIVATO
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
) {
    let cnt = tilemap_settings.discovery_range as i32;   

    let mut to_insert = Vec::new();

    for (transform, mut activates_chunks, &dimension_ref) in query.iter_mut() {

        let center_chunk_pos = ChunkPos::from(transform.translation().xy());

        for y in (center_chunk_pos.y() - cnt + 1)..(center_chunk_pos.y() + cnt) {
            for x in (center_chunk_pos.x() - cnt + 1)..(center_chunk_pos.x() + cnt) {

                let chunk_pos = ChunkPos::new(x, y);
                let key = (dimension_ref, chunk_pos);
                let chunk_ent = loaded_chunks.0.get(&key).copied().unwrap_or_else(|| {
                    let ent = cmd.spawn_empty().id();
                    to_insert.push((ent, (
                        Chunk,
                        Name::new(format!("Chunk ({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        Transform::from_translation(chunk_pos.to_pixelpos().extend(0.0)),
                        chunk_pos,
                        ChildOf(dimension_ref.0),
                    )));
                    loaded_chunks.0.insert(key, ent);
                    ent
                });
                activates_chunks.0.insert(chunk_ent);
            }
        }
    }
    cmd.insert_batch(to_insert);
}
#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&DimensionRef, &GlobalTransform, &mut ActivatingChunks, ), >,
    mut chunks_query: Query<(&ChildOf, Entity, &ChunkPos, ), >,
    chunkrange_settings: Res<AaChunkRangeSettings>,
) {
    for (dimension_ref, act_transform, mut activate_chunks) in activator_query.iter_mut() {

        let act_chunk_pos = ChunkPos::from(act_transform.translation().xy());

        for (chunk_dimension_ref, entity, &chunk_pos, ) in chunks_query.iter_mut() {
            if chunk_dimension_ref.parent() != dimension_ref.0 {
                activate_chunks.0.remove(&entity);
                continue;
            }

            if chunkrange_settings.out_of_active_range(act_transform, chunk_pos) && chunkrange_settings.out_of_discovery_range(act_chunk_pos, chunk_pos) {
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
    mut chunks_query: Query<(&ChildOf, Entity, &ChunkPos, &Children, &TilesToSave), With<Chunk>>,
    tmaps: Query<&TileStorage>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut event_writer: EventWriter<SavedTileHadChunkDespawn>,
) {
    let mut events = Vec::new();
    for (child_of, chunk_ent, &chunk_pos, children, tiles_to_save) in chunks_query.iter_mut() {
        let referenced = activator_query.iter().any(|(activates_chunks, )| activates_chunks.0.contains(&chunk_ent));
        
        if !referenced {
            trace!("Despawning chunk {:?} at pos: {:?}", chunk_ent, chunk_pos);

            loaded_chunks.0.remove(&(DimensionRef(child_of.parent()), chunk_pos));

            for child in children.iter() {
                if let Ok(tile_storage) = tmaps.get(child) {
                    for pos in tile_storage.iter() {
                        if let Some(tile_entity) = pos {
                            if tiles_to_save.entities().contains(tile_entity) {
                              
                            } else{
                                commands.entity(*tile_entity).try_despawn();
                            }
                        }
                    }
                }


                if tiles_to_save.entities().contains(&child) {
                    
                    commands.entity(child).try_remove::<ChildOf>();//TODO reajustar transform (ya no es childof)
                    events.push(SavedTileHadChunkDespawn(child));
                } else{//HACE FALTA
                    commands.entity(child).try_despawn();
                }
            }
            commands.entity(chunk_ent).try_despawn();
        }
    }
    event_writer.write_batch(events);
}


#[allow(unused_parens)]
pub fn show_or_hide_chunks(
    camera_query: Single<(&GlobalTransform), (With<CameraTarget>, Or<(Changed<GlobalTransform>, Added<CameraTarget>)>)>,
    mut chunks_query: Query<(&mut Visibility, &ChunkPos, &Children), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut event_writer: EventWriter<DrawTilemap>,
) {
    let camera_transform = *camera_query;

    let camera_chunk_pos = ChunkPos::from(camera_transform.translation().xy());
    let mut to_draw = Vec::new();


    for (mut visibility, &chunk_pos, children) in chunks_query.iter_mut() {

        let out_of_visible = chunkrange_settings.out_of_visible_range(camera_transform, chunk_pos);
        let out_of_discovery = chunkrange_settings.out_of_discovery_range(camera_chunk_pos, chunk_pos);

        if out_of_visible && out_of_discovery {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        } else if *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
            for child in children.iter() {
                to_draw.push(DrawTilemap(child));
            }
        }
    }
    event_writer.write_batch(to_draw);
}


