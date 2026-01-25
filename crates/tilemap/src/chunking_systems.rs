
use core::error;

use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, tiles::TileStorage};
use camera::camera_components::CameraTarget;
use common::common_components::{StrId, StrId20B};
use dimension_shared::DimensionRef
;
use tilemap_shared::{ChunkPos, GlobalTilePos, HashablePosVec};

use crate::{chunking_components::*, chunking_resources::*, regioning::{regioning_components::Region, regioning_resources::LoadedRegions}, tile::tile_messages::SavedTileHadChunkDespawn};


#[allow(unused_parens, )]
pub fn visit_chunks_around_activators(
    mut cmd: Commands, 
    mut query: Query<(&GlobalTransform, &mut ActivatingChunks, &DimensionRef), ()>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
    mut loaded_regions: ResMut<LoadedRegions>,
    mut reader: MessageReader<ReactivateChunksFor>,
) {
    let mut comps_for_chunk_ents = Vec::new();
    let mut comps_for_region_ents = Vec::new();
    let cnt = tilemap_settings.discovery_range as i32;   
    
    for msg in reader.read() {
        let Ok((transform, mut activates_chunks, &dimension_ref)) = query.get_mut(msg.0) else {
            error!(target: "chunk_activation", "Activator entity {:?} not found when reactivating chunks", msg.0);
            continue;
        };
        let center_chunk_pos = ChunkPos::from(transform.translation().xy());
        
        activates_chunks.reactivation_timer.reset();
        
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
                                StrId20B::trunc(format!("Region({}, {})", region_pos.0.x, region_pos.0.y)),
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
                        StrId20B::trunc(format!("Chunk({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        Transform::from_translation(chunk_pos.to_pixelpos().extend(0.0)),
                        chunk_pos,
                        ChildOf(region_ent),
                        dimension_ref,
                    )));
                    chunk_ent
                });
                if !activates_chunks.entities.contains(&chunk_ent) {
                    activates_chunks.entities.push(chunk_ent);
                }
            }
        }
    }
    cmd.try_insert_batch(comps_for_region_ents);
    cmd.try_insert_batch(comps_for_chunk_ents);
    
}

#[allow(unused_parens)]
pub fn activate_chunks_every_second( 
    mut query: Query<(Entity, &mut ActivatingChunks),()>,
    time: Res<Time>,
    mut writer: MessageWriter<ReactivateChunksFor>,
) {
    let mut to_reactivate = Vec::new();
    for (entity, mut activates_chunks) in query.iter_mut() {
        activates_chunks.reactivation_timer.tick(time.delta());
        if activates_chunks.reactivation_timer.is_finished() {
            to_reactivate.push(ReactivateChunksFor(entity));
        }
    }
    writer.write_batch(to_reactivate);
}
#[allow(unused_parens, )]
pub fn detect_activators_with_changes(
    query: Query<(Entity), 
    (Or<(Changed<GlobalTransform>, Changed<DimensionRef>, Added<ActivatingChunks>,)>, With <ActivatingChunks>)>,
    mut writer: MessageWriter<ReactivateChunksFor>,
) {
    let msgs: Vec<ReactivateChunksFor> = query.iter().map(|activator_ent| ReactivateChunksFor(activator_ent)).collect();
    writer.write_batch(msgs);
}

#[derive(Message, Debug, Clone, )]
pub struct ReactivateChunksFor(pub Entity);



#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&GlobalTransform, &mut ActivatingChunks), (Or<(Changed<GlobalTransform>, )>, )>,
    chunks_query: Query<(&ChunkPos), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut ewriter: MessageWriter<CheckChunkDespawn>,
) {
    let mut to_despawn = Vec::new();
    for (act_transform, mut activates_chunks) in activator_query.iter_mut() {
        let act_chunk_pos = ChunkPos::from(act_transform.translation().xy());
        let mut i = 0;
        while i < activates_chunks.entities.len() {
            let chunk_ent = activates_chunks.entities[i];
            if let Ok((&chunk_pos)) = chunks_query.get(chunk_ent) {
                let keep = !(chunkrange_settings.out_of_active_range(act_transform, chunk_pos) &&
                chunkrange_settings.out_of_discovery_range(act_chunk_pos, chunk_pos));
                if keep {
                    i += 1;
                } else {
                    activates_chunks.entities.swap_remove(i);
                    to_despawn.push(CheckChunkDespawn(chunk_ent, 0));
                }
            } else {
                activates_chunks.entities.swap_remove(i);
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
    activator_query.iter_mut().for_each(|mut activated_chunks| { 
        for &entity in activated_chunks.entities.iter() {
            check_if_despawn.push(CheckChunkDespawn(entity, 0));
        }
        activated_chunks.entities.clear();
    });
    ewriter.write_batch(check_if_despawn);
}

#[derive(Debug, Message)]
pub struct CheckChunkDespawn (pub Entity, pub u8,);//u8 = retransmission count


#[allow(unused_parens)]
pub fn despawn_unreferenced_chunks(
    mut cmd: Commands,
    activator_query: Query<(&DimensionRef, &ActivatingChunks, ), >,
    chunks_query: Query<(&DimensionRef, &ChunkPos, Option<&Children>, Option<&TilesToSave>), >,
    tmaps: Query<&TileStorage>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut despawn_events: ResMut<Messages<CheckChunkDespawn>>,
    mut tosave_event_writer: MessageWriter<SavedTileHadChunkDespawn>,
) {
    let mut tosave_events = Vec::new();
    let mut despawn_retransmitted_events = Vec::new();
    
    for CheckChunkDespawn(chunk_ent, retransmission_count) in despawn_events.drain() {
        let Ok((&chunk_dimension, &chunk_pos, children, tiles_to_save)) = chunks_query.get(chunk_ent) else {
            //cmd.entity(chunk_ent).try_despawn();
            continue; 
        };
        
        let referenced = activator_query.iter().any(|(&dimension_ref, activates_chunks)| {
            dimension_ref == chunk_dimension && activates_chunks.entities.contains(&chunk_ent)
        });
        
        if !referenced {
            loaded_chunks.0.remove(&(chunk_dimension, chunk_pos));
            
            if let Some(children) = children.as_ref() {
                for child in children.iter() {
                    // if let Ok(tile_storage) = tmaps.get(child) {
                    //     for pos in tile_storage.iter() {
                    //         if let Some(tile_entity) = pos {
                    //             if tiles_to_save.entities().contains(tile_entity) {
                    
                    //             } else{
                    //                 cmd.entity(*tile_entity).try_despawn();
                    //             }
                    //         }
                    //     }
                    // }
                    // if tiles_to_save.entities().contains(&child) {
                    
                    //     cmd.entity(child).try_remove::<ChildOf>();//esto hace q el sistema limpiador la borre, hay q hacer algo
                    //     tosave_events.push(SavedTileHadChunkDespawn(child));
                    // } else{//HACE FALTA
                    //     cmd.entity(child).try_despawn();
                    // }
                }
            }
            cmd.entity(chunk_ent).try_despawn();
        }
    }
    tosave_event_writer.write_batch(tosave_events);
    despawn_events.write_batch(despawn_retransmitted_events);
}



#[allow(unused_parens)]
pub fn update_chunk_visib(
    mut reader: MessageReader<RecheckChunksVisibility>,
    camera_query: Single<(&GlobalTransform), (With<CameraTarget>)>,
    mut chunks_query: Query<(&mut Visibility, &ChunkPos, &Children), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut event_writer: MessageWriter<DrawTilemap>,
) {
    if reader.read().next().is_none() {
        return;
    }
    let camera_transform = *camera_query;
    
    let camera_chunk_pos = ChunkPos::from(camera_transform.translation().xy());
    let mut to_draw = Vec::with_capacity(chunks_query.iter().len()/2);
    
    chunks_query.iter_mut().for_each(|(mut visibility, &chunk_pos, children)| {
        let out_of_visible = chunkrange_settings.out_of_visible_range(camera_transform, chunk_pos);
        let out_of_discovery = chunkrange_settings.out_of_discovery_range(camera_chunk_pos, chunk_pos);
        
        if out_of_visible && out_of_discovery {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        } else if *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
            to_draw.extend(children.iter().map(|child| DrawTilemap(child)));
        }
    });
    event_writer.write_batch(to_draw);
}

#[derive(Message, Debug, Clone, )]
pub struct RecheckChunksVisibility;

#[allow(unused_parens)]
pub fn detect_camera_change_pos(
    _: Single<(&CameraTarget), (Or<(Changed<GlobalTransform>, Added<CameraTarget>, Changed<DimensionRef>, )>, )>,
    mut recheck_writer: MessageWriter<RecheckChunksVisibility>,
) {
    recheck_writer.write(RecheckChunksVisibility);
    trace!(target: "chunk_visibility", "Camera position or dimension changed, rechecking chunk visibility.");
}

pub fn recheck_chunk_visibility(
    mut recheck_writer: MessageWriter<RecheckChunksVisibility>,
) {
    recheck_writer.write(RecheckChunksVisibility);
    trace!(target: "chunk_visibility", "Rechecking chunk visibility due to timer.");
}