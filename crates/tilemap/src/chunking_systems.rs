
use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, tiles::TileStorage};
use camera::camera_components::CameraTarget;
use common::common_components::{AnyDisabling, AssetScoped, StrId20B};
use dimension_shared::DimensionRef;
use game_common::game_common_components::DespawnTimer;
use std::{collections::{HashMap, HashSet}, time::Duration};
use tilemap_shared::{ChunkPos, HashablePosVec, RegionPos};

use crate::{chunking_components::*, chunking_resources::*, regioning::{regioning_components::Region, regioning_resources::LoadedRegions}, tile::tile_messages::SavedTileHadChunkDespawn};



#[allow(unused_parens, )]
pub fn spawn_chunks_around_activators(
    mut cmd: Commands, 
    mut query: Query<(&GlobalTransform, &mut ActivatingChunks, &DimensionRef), ()>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
    mut loaded_regions: ResMut<LoadedRegions>,
    mut reader: MessageReader<ReactivateChunksFor>,
) {
    let cnt = tilemap_settings.discovery_range as i32;   
    let range_len = (2 * cnt - 1).max(0) as usize;
    let approx_chunks = range_len.saturating_mul(range_len);
    let mut comps_for_region_ents = Vec::new();
    let mut comps_for_chunk_ents = Vec::new();
    
    
    for msg in reader.read() {
        let Ok((transform, mut activates_chunks, &dimension_ref)) = query.get_mut(msg.0) else {
            error!(target: "chunk_activation", "Activator entity {:?} not found when reactivating chunks", msg.0);
            continue;
        };
        comps_for_chunk_ents.reserve(approx_chunks);
        comps_for_region_ents.reserve(approx_chunks / 8);
        let center_chunk_pos = ChunkPos::from(transform.translation().xy());
        
        activates_chunks.reactivation_timer.reset();
        
        for y in (center_chunk_pos.y() - cnt + 1)..(center_chunk_pos.y() + cnt) {
            for x in (center_chunk_pos.x() - cnt + 1)..(center_chunk_pos.x() + cnt) {
                
                let chunk_pos = ChunkPos::new(x, y);
                let key = (dimension_ref, chunk_pos);
                let chunk_ent = loaded_chunks.0.get(&key)
                .copied()
                .or_else(|| loaded_chunks.0.get(&key).copied())
                .unwrap_or_else(|| {
                    let region_ent = {
                        let region_pos = chunk_pos.to_region_pos();
                        let region_key = (dimension_ref, region_pos);
                        loaded_regions.0.get(&region_key)
                        .copied()
                        .or_else(|| loaded_regions.0.get(&region_key).copied())
                        .unwrap_or_else(|| {
                            let region_ent = cmd.spawn_empty().id();
                            comps_for_region_ents.push((region_ent, (
                                region_pos,
                                Region,
                                StrId20B::trunc(format!("Region({}, {})", region_pos.0.x, region_pos.0.y)),
                                Transform::default(),
                                ChildOf(dimension_ref.0),
                                dimension_ref,
                            )));
                            loaded_regions.0.insert(region_key, region_ent);
                            region_ent
                        })
                    };
                    
                    let chunk_ent = cmd.spawn_empty().id();
                    loaded_chunks.0.insert(key, chunk_ent);
                    comps_for_chunk_ents.push((chunk_ent, (
                        Chunk { region_ent, },
                        Visibility::Hidden,
                        AssetScoped,
                        TilesToSave::default(),
                        StrId20B::trunc(format!("Chunk({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        Transform::default(),
                        chunk_pos,
                        ChildOf(region_ent),
                        dimension_ref,
                        DespawnTimer::new(6.0),//LEAVE AT 6.0 FOR SLOW PCs
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

#[derive(Message, Debug, Clone, )]
pub struct ReactivateChunksFor(pub Entity);

#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&GlobalTransform, &mut ActivatingChunks, &DimensionRef), (Or<(Changed<GlobalTransform>, Changed<DimensionRef>)>, )>,
    chunks_query: Query<(&ChunkPos, &DimensionRef), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut ewriter: MessageWriter<CheckChunkDespawn>,
    mut to_despawn: Local<Vec<CheckChunkDespawn>>,
) {
    for (act_transform, mut activates_chunks, &act_dim) in activator_query.iter_mut() {
        let act_chunk_pos = ChunkPos::from(act_transform.translation().xy());
        let mut i = 0;
        while i < activates_chunks.entities.len() {
            let chunk_ent = activates_chunks.entities[i];
            let Ok((&chunk_pos, &chunk_dim)) = chunks_query.get(chunk_ent) else {
                activates_chunks.entities.swap_remove(i);
                continue;
            };

            let keep = chunk_dim == act_dim && 
                !(chunkrange_settings.out_of_active_range(act_transform, chunk_pos)
                && chunkrange_settings.out_of_discovery_range(act_chunk_pos, chunk_pos));
            if !keep {
                to_despawn.push(CheckChunkDespawn(chunk_ent, 0));
                activates_chunks.entities.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
    ewriter.write_batch(to_despawn.drain(..));
}

#[derive(Debug, Message)]
pub struct CheckChunkDespawn (pub Entity, pub u8,);//u8 = retransmission count

#[derive(Debug, Message)]
pub struct ForceChunkDespawn (pub Entity, );

pub fn periodically_check_despawn_unreferenced_chunks(
    mut ewriter: MessageWriter<CheckChunkDespawn>,
    chunks_query: Query<Entity, With<Chunk>,>,
    mut to_check: Local<Vec<CheckChunkDespawn>>,
) {
    for chunk_ent in chunks_query.iter() {
        to_check.push(CheckChunkDespawn(chunk_ent, 0));
    }
    ewriter.write_batch(to_check.drain(..));
}

#[allow(unused_parens)]
pub fn despawn_chunks(
    mut cmd: Commands,
    activator_query: Query<(&DimensionRef, &ActivatingChunks, ), >,
    chunks_query: Query<(&DimensionRef, &ChunkPos, Option<&Children>, Option<&TilesToSave>), >,
    tmaps: Query<&TileStorage, AnyDisabling>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut despawn_events: ResMut<Messages<CheckChunkDespawn>>,
    mut tosave_event_writer: MessageWriter<SavedTileHadChunkDespawn>,
    mut force_despawn_reader: MessageReader<ForceChunkDespawn>,
    mut referenced_chunks: Local<HashSet<Entity>>,
    mut tosave_events: Local<Vec<SavedTileHadChunkDespawn>>,
    mut chunks_to_despawn: Local<Vec<Entity>>,
) {
    referenced_chunks.clear();
    referenced_chunks.reserve(activator_query.iter().map(|(_, a)| a.entities.len()).sum());
    for (_, activates_chunks) in activator_query.iter() {
        referenced_chunks.extend(activates_chunks.entities.iter().copied());
    }
    tosave_events.clear();
    
    for CheckChunkDespawn(chunk_ent, _retransmission_count) in despawn_events.drain() {
        
        let referenced = referenced_chunks.contains(&chunk_ent);
        
        if !referenced {
            
            chunks_to_despawn.push(chunk_ent);
        }
    }
    chunks_to_despawn.extend(force_despawn_reader.read().map(|msg| msg.0));
    
    for chunk_ent in chunks_to_despawn.drain(..) {
        let Ok((&chunk_dimension, &chunk_pos, children, _tiles_to_save)) = chunks_query.get(chunk_ent) else {
            //cmd.entity(chunk_ent).try_despawn();
            continue; 
        };
        loaded_chunks.0.remove(&(chunk_dimension, chunk_pos));
            
        if let Some(children) = children.as_ref() {
            for child in children.iter() {
                if let Ok(tile_storage) = tmaps.get(child) {
                    for tile_entity in tile_storage.iter() {
                        if let Some(tile_entity) = tile_entity {
                            cmd.entity(*tile_entity).try_despawn();
                        }
                    }
                }
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
    tosave_event_writer.write_batch(tosave_events.drain(..));

}



#[allow(unused_parens)]
pub fn update_chunk_visib(
    mut reader: MessageReader<RecheckChunksVisibility>,
    camera_query: Single<(&GlobalTransform), (With<CameraTarget>)>,
    mut chunks_query: Query<(&mut Visibility, &ChunkPos, &Children), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut event_writer: MessageWriter<DrawTilemap>,
    mut to_draw: Local<Vec<DrawTilemap>>,
) {
    if reader.is_empty() {
        return;
    }
    let camera_transform = *camera_query;
    
    let camera_chunk_pos = ChunkPos::from(camera_transform.translation().xy());
    to_draw.reserve(reader.read().size_hint().0*3);
    
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
    event_writer.write_batch(to_draw.drain(..));
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

pub fn periodically_recheck_chunk_visibility(
    mut recheck_writer: MessageWriter<RecheckChunksVisibility>,
) {
    recheck_writer.write(RecheckChunksVisibility);
    trace!(target: "chunk_visibility", "Rechecking chunk visibility due to timer.");
}

#[allow(unused_parens)]
pub fn activate_chunks_every_second( //TODO borrar esto y hacer que se haga 1 segundo despues del ultimo movimiento
    mut query: Query<(Entity, &mut ActivatingChunks),()>,
    time: Res<Time>,
    mut writer: MessageWriter<ReactivateChunksFor>,
    mut to_reactivate: Local<Vec<ReactivateChunksFor>>,
) {
    for (being_entity, mut activates_chunks) in query.iter_mut() {
        activates_chunks.reactivation_timer.tick(time.delta());
        if activates_chunks.reactivation_timer.is_finished() {
            to_reactivate.push(ReactivateChunksFor(being_entity));
        }
    }
    writer.write_batch(to_reactivate.drain(..));
}
#[allow(unused_parens, )]
pub fn detect_activators_with_pos_changes(
    query: Query<(Entity), 
    (Or<(Changed<GlobalTransform>, Changed<DimensionRef>, Added<ActivatingChunks>,)>, With <ActivatingChunks>)>,
    mut writer: MessageWriter<ReactivateChunksFor>,
    mut msgs: Local<Vec<ReactivateChunksFor>>,
) {
    msgs.extend(query.iter().map(ReactivateChunksFor));
    writer.write_batch(msgs.drain(..));
}
