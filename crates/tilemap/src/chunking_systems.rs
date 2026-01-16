
use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, tiles::TileStorage};
use camera::camera_components::CameraTarget;
use common::common_components::{StrId, StrId20B};
use dimension_shared::DimensionRef
;
use tilemap_shared::{ChunkPos, GlobalTilePos, HashablePosVec};

use crate::{chunking_components::*, chunking_resources::*, regioning::{regioning_components::Region, regioning_resources::LoadedRegions}, tile::tile_messages::SavedTileHadChunkDespawn};

//TODO HACERLO MÁS EVENT-DRIVEN

#[allow(unused_parens, )]
pub fn visit_chunks_around_activators(
    mut cmd: Commands, 
    mut query: Query<(&GlobalTransform, &mut ActivatingChunks, &DimensionRef), 
    (Or<(Changed<GlobalTransform>, Changed<DimensionRef>, Added<ActivatingChunks>)>, )>,//TODO USAR EVENTOS?
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
    mut loaded_regions: ResMut<LoadedRegions>,
) {
    let cnt = tilemap_settings.discovery_range as i32;   
    let mut comps_for_chunk_ents = Vec::new();
    let mut comps_for_region_ents = Vec::new();

    for (transform, mut activates_chunks, &dimension_ref) in query.iter_mut() {

        let center_chunk_pos = ChunkPos::from(transform.translation().xy());

        for y in (center_chunk_pos.y() - cnt + 1)..(center_chunk_pos.y() + cnt) {
            for x in (center_chunk_pos.x() - cnt + 1)..(center_chunk_pos.x() + cnt) {

                let chunk_pos = ChunkPos::new(x, y);
                let key = (dimension_ref, chunk_pos);
                let chunk_ent = loaded_chunks.0.get(&key).copied().unwrap_or_else(|| {

                    let region_ent = {
                        let region_pos = chunk_pos.to_region_pos();
                        let region_key = (dimension_ref, region_pos);
                        loaded_regions.0.entry(region_key).or_insert_with(|| {
                            let region_ent = cmd.spawn_empty().id();
                            comps_for_region_ents.push((region_ent, (
                                region_pos,
                                Region,
                                StrId20B::new_truncated(format!("Region({}, {})", region_pos.0.x, region_pos.0.y)),
                                Transform::default(),
                                ChildOf(dimension_ref.0),
                                dimension_ref,
                            )));
                            region_ent
                        }).clone()
                    };
                    
                    let chunk_ent = cmd.spawn_empty().id();
                    loaded_chunks.0.insert(key, chunk_ent);
                    comps_for_chunk_ents.push((chunk_ent, (
                        Chunk { region_ent, },
                        StrId20B::new_truncated(format!("Chunk({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        Transform::from_translation(chunk_pos.to_pixelpos().extend(0.0)),
                        chunk_pos,
                        ChildOf(region_ent),
                        dimension_ref,
                    )));
                    chunk_ent
                });
                if !activates_chunks.0.contains(&chunk_ent) {
                    activates_chunks.0.push(chunk_ent);
                }
            }
        }
    }
    cmd.try_insert_batch(comps_for_region_ents);
    cmd.try_insert_batch(comps_for_chunk_ents);
}
#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&GlobalTransform, &mut ActivatingChunks), (Or<(Changed<GlobalTransform>, )>, )>,
    chunks_query: Query<(&ChunkPos), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut ewriter: MessageWriter<CheckChunkDespawn>,
) {
    let mut to_despawn = Vec::new();
    for (act_transform, mut activate_chunks) in activator_query.iter_mut() {
        let act_chunk_pos = ChunkPos::from(act_transform.translation().xy());
        let mut i = 0;
        while i < activate_chunks.0.len() {
            let chunk_ent = activate_chunks.0[i];
            if let Ok((&chunk_pos)) = chunks_query.get(chunk_ent) {
                let keep = !(chunkrange_settings.out_of_active_range(act_transform, chunk_pos) &&
                    chunkrange_settings.out_of_discovery_range(act_chunk_pos, chunk_pos));
                if keep {
                    i += 1;
                } else {
                    activate_chunks.0.swap_remove(i);
                    to_despawn.push(CheckChunkDespawn(chunk_ent, 0));
                }
            } else {
                activate_chunks.0.swap_remove(i);
            }
        }
    }
    ewriter.write_batch(to_despawn);
}


#[allow(unused_parens, )]
pub fn clear_chunks_on_dim_change(
    mut activator_query: Query<(&mut ActivatingChunks), (Changed<DimensionRef>, )>,
    mut ewriter: MessageWriter<CheckChunkDespawn>,
) {
    let mut check_if_despawn = Vec::new();
    for (mut activate_chunks) in activator_query.iter_mut() { 
        for &entity in activate_chunks.0.iter() {
            check_if_despawn.push(CheckChunkDespawn(entity, 0));
        }
        activate_chunks.0.clear();
    }
    ewriter.write_batch(check_if_despawn);
}

#[derive(Debug, Message)]
pub struct CheckChunkDespawn (pub Entity, pub u8,);//u8 = retransmission count


#[allow(unused_parens)]
pub fn despawn_unreferenced_chunks(
    mut commands: Commands,
    activator_query: Query<(&DimensionRef, &ActivatingChunks, ), >,
    chunks_query: Query<(&DimensionRef, &ChunkPos, &Children, &TilesToSave), >,
    tmaps: Query<&TileStorage>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut despawn_events: ResMut<Messages<CheckChunkDespawn>>,
    mut tosave_event_writer: MessageWriter<SavedTileHadChunkDespawn>,
) {
    let mut tosave_events = Vec::new();
    let mut despawn_retransmitted_events = Vec::new();

    for CheckChunkDespawn(chunk_ent, retransmission_count) in despawn_events.drain() {
        let Ok((&chunk_dimension, &chunk_pos, children, tiles_to_save)) = chunks_query.get(chunk_ent) else {
            continue; 
        };

        let referenced = activator_query.iter().any(|(&dimension_ref, activates_chunks)| {
            dimension_ref == chunk_dimension && activates_chunks.0.contains(&chunk_ent)
        });

        if !referenced {
            loaded_chunks.0.remove(&(chunk_dimension, chunk_pos));

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
                    
                    commands.entity(child).try_remove::<ChildOf>();//esto hace q el sistema limpiador la borre, hay q hacer algo
                    tosave_events.push(SavedTileHadChunkDespawn(child));
                } else{//HACE FALTA
                    commands.entity(child).try_despawn();
                }
            }
            commands.entity(chunk_ent).try_despawn();
        }
    }
    tosave_event_writer.write_batch(tosave_events);
    despawn_events.write_batch(despawn_retransmitted_events);
}


#[allow(unused_parens)]
pub fn show_or_hide_chunks(
    camera_query: Single<(&GlobalTransform), (With<CameraTarget>, Or<(Changed<GlobalTransform>, Added<CameraTarget>, Changed<DimensionRef>, )>, )>,
    mut chunks_query: Query<(&mut Visibility, &ChunkPos, &Children), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut event_writer: MessageWriter<DrawTilemap>,
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
                //TODO HACER TILES Q ESTEN EN ESA POS DEL CHUNK PERO NO SEAN CHILD SE VUELVAN OCULTAS TAMBIEN, VA A HABER Q REGISTRARLAS EN UN MAPA<CHUNKPOS, VEC<ENTITY>>
                //MUCHAS COSAS PARA DIBUJAR PUEDEN LAGGEAR AUNQUE NO SE VEAN
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

