
use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor, tiles::TileStorage};
use camera::camera_components::CameraTarget;
use common::common_components::{AnyDisabling, AssetScoped, StrId20B};
use dimension_shared::DimensionRef;
use game_common::game_common_components::DespawnTimer;
use std::{collections::{HashMap, HashSet}, time::Duration};
use tilemap_shared::{ChunkPos, ForceAllChunksDespawn, HashablePosVec, RegionPos};

use super::chunking_components::*;
use super::chunking_resources::*;
use crate::{regioning::{regioning_components::Region, regioning_resources::LoadedRegions}, tile::tile_messages::SavedTileHadChunkDespawn, tilemap_resources::{MassCollectedTiles, }};


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
                to_despawn.push(CheckChunkDespawn(chunk_ent));
                activates_chunks.entities.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
    ewriter.write_batch(to_despawn.drain(..));
}

#[derive(Debug, Message)]
pub struct CheckChunkDespawn (pub Entity);


#[allow(unused_parens)]
pub fn on_message_signal_despawn_all_chunks(
    mut reader: MessageReader<ForceAllChunksDespawn>,
    chunks_query: Query<(Entity),(With<Chunk>)>,
    mut writer: MessageWriter<ForceChunkDespawn>,
) {
    if reader.is_empty() {
        return;
    }
    reader.clear();
    let mut evs = Vec::with_capacity(chunks_query.iter().size_hint().0);
    for ent in chunks_query.iter() {
        evs.push(ForceChunkDespawn(ent));
    }

    writer.write_batch(evs);
}


#[derive(Debug, Message)]
pub struct ForceChunkDespawn (pub Entity, );

pub fn periodically_check_despawn_unreferenced_chunks(
    mut ewriter: MessageWriter<CheckChunkDespawn>,
    chunks_query: Query<Entity, With<Chunk>,>,
    mut to_check: Local<Vec<CheckChunkDespawn>>,
) {
    for chunk_ent in chunks_query.iter() {
        to_check.push(CheckChunkDespawn(chunk_ent));
    }
    ewriter.write_batch(to_check.drain(..));
}

#[allow(unused_parens)]
pub fn despawn_chunks(//DEJARLO DE ESTA FORMA PARA CENTRALIZAR EL SISTEMA DONDE PUEDEN DESPAWNEAR LOS CHUNKS, PARA RESPETAR EL ORDEN DE SISTEMAS
    mut cmd: Commands,
    mut activator_query: Query<(&DimensionRef, &mut ActivatingChunks, ), >,
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
    
    for CheckChunkDespawn(chunk_ent) in despawn_events.drain() {
        
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
        for (_, mut activates_chunks) in activator_query.iter_mut() {
            let mut i = 0;
            while i < activates_chunks.entities.len() {
                if activates_chunks.entities[i] == chunk_ent {
                    activates_chunks.entities.swap_remove(i);
                } else {
                    i += 1;
                }
            }
        }
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
