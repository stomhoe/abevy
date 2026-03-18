use crate::being_components::{Being, Chaser};
use crate::nav::{ChaserNavPlans, RetainedChasePathSnapshot, extend_target_chunk_trail, rebuild_connected_chase_chunk_path, rebuild_retained_chase_chunk_positions, retained_target_chunk_trails_match_positions, retained_target_trail_stale_timeout};
use ::being_shared::*;
use ::tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos};
use bevy::{platform::collections::HashSet, prelude::*};
use common::log_targets::BEING_SYSTEM;
use faction::faction_components::BelongsToAPlayerFaction;
use movement::movement_components::SpeedMagnitude;
use tilemap::chunking::{ActivateChunksAround, ActivatingChunks, BeingsWithinChunk, BeingChunkDespawned, Chunk};
use std::time::Duration;

fn unload_being_for_chunk_despawn(commands: &mut Commands, being_ent: Entity) {
    let mut entity = commands.entity(being_ent);
    entity.try_insert(BackgroundSimulated);
    entity.try_remove::<(Name, ActivateChunksAround, ActivatingChunks, RetainedChasePathSnapshot)>();
}

fn cancels_chunk_despawn(
    chaser: Option<&Chaser>,
    targets: &Query<(&ChunkPos, Has<BelongsToAPlayerFaction>), With<Being>>,
) -> bool {
    let Some(chaser) = chaser else {
        return false;
    };
    let Ok((_, target_is_player_faction)) = targets.get(chaser.target) else {
        return false;
    };
    target_is_player_faction
}

fn refresh_retained_chase_snapshot(
    commands: &mut Commands,
    being_ent: Entity,
    chaser_gpos: GlobalTilePos,
    chaser_chunk_pos: ChunkPos,
    target_chunk_pos: ChunkPos,
    nav_path_tiles: Option<&[GlobalTilePos]>,
    chaser_speed: f32,
    activating_chunks: Option<&ActivatingChunks>,
    snapshot: Option<&RetainedChasePathSnapshot>,
    desired_chunks: &mut Vec<ChunkPos>,
    corridor_chunks: &mut Vec<ChunkPos>,
    seen_chunks: &mut HashSet<ChunkPos>,
) {
    let stale_timeout = retained_target_trail_stale_timeout(chaser_speed);
    let mut target_chunk_trail = snapshot
        .map(|snapshot| snapshot.target_chunk_trail.clone())
        .unwrap_or_default();
    if target_chunk_trail.is_empty() {
        extend_target_chunk_trail(&mut target_chunk_trail, target_chunk_pos, stale_timeout);
    } else if target_chunk_trail.last().map(|entry| entry.chunk_pos) != Some(target_chunk_pos) {
        extend_target_chunk_trail(&mut target_chunk_trail, target_chunk_pos, stale_timeout);
    }

    rebuild_connected_chase_chunk_path(
        corridor_chunks,
        seen_chunks,
        Some(chaser_gpos),
        nav_path_tiles,
        chaser_chunk_pos,
        target_chunk_pos,
    );

    rebuild_retained_chase_chunk_positions(
        desired_chunks,
        corridor_chunks,
        seen_chunks,
        &mut target_chunk_trail,
        stale_timeout,
        Duration::ZERO,
    );

    let needs_snapshot = snapshot.map(|snapshot| snapshot.chunk_positions.as_slice()) != Some(desired_chunks.as_slice())
        || snapshot.map(|snapshot| retained_target_chunk_trails_match_positions(&snapshot.target_chunk_trail, &target_chunk_trail)) != Some(true)
        || snapshot.map(|snapshot| snapshot.last_target_chunk_pos) != Some(target_chunk_pos);
    let needs_activating_chunks = activating_chunks.map(|chunks| chunks.chunk_positions.as_slice()) != Some(desired_chunks.as_slice());

    let mut entity = commands.entity(being_ent);
    entity.try_remove::<BackgroundSimulated>();
    entity.try_remove::<ActivateChunksAround>();
    if needs_snapshot || needs_activating_chunks {
        entity.try_insert((
            RetainedChasePathSnapshot {
                chunk_positions: desired_chunks.clone(),
                target_chunk_trail,
                last_target_chunk_pos: target_chunk_pos,
            },
            ActivatingChunks {
                chunk_positions: desired_chunks.clone(),
            },
        ));
    }
}

pub fn on_chunk_with_beings_attempt_unload(
    mut commands: Commands,
    mut reader: MessageReader<BeingChunkDespawned>,
    plans: Res<ChaserNavPlans>,
    chunks: Query<&BeingsWithinChunk, With<Chunk>>,
    beings: Query<(&GlobalTilePos, &ChunkPos, Option<&Chaser>, Option<&SpeedMagnitude>, Option<&ActivatingChunks>, Option<&RetainedChasePathSnapshot>), With<Being>>,
    targets: Query<(&ChunkPos, Has<BelongsToAPlayerFaction>), With<Being>>,
    mut desired_chunks: Local<Vec<ChunkPos>>,
    mut corridor_chunks: Local<Vec<ChunkPos>>,
    mut seen_chunks: Local<HashSet<ChunkPos>>,
) {
    for msg in reader.read() {
        let Ok(beings_within_chunk) = chunks.get(msg.chunk_ent) else {
            continue;
        };

        let should_cancel = beings_within_chunk.entities().iter().copied().any(|being_ent| {
            let Ok((_, _, chaser, _, _, _)) = beings.get(being_ent) else {
                return false;
            };
            cancels_chunk_despawn(chaser, &targets)
        });

        if should_cancel {
            for &being_ent in beings_within_chunk.entities() {
                let Ok((chaser_gpos, &chaser_chunk_pos, chaser, speed, activating_chunks, snapshot)) = beings.get(being_ent) else {
                    continue;
                };
                let Some(chaser) = chaser else {
                    continue;
                };
                let Ok((&target_chunk_pos, target_is_player_faction)) = targets.get(chaser.target) else {
                    continue;
                };
                if !target_is_player_faction {
                    continue;
                }

                refresh_retained_chase_snapshot(
                    &mut commands,
                    being_ent,
                    *chaser_gpos,
                    chaser_chunk_pos,
                    target_chunk_pos,
                    plans.by_ent.get(&being_ent).map(|plan| plan.path_tiles.as_slice()),
                    speed.map_or(1.0, |speed| speed.0),
                    activating_chunks,
                    snapshot,
                    &mut desired_chunks,
                    &mut corridor_chunks,
                    &mut seen_chunks,
                );
            }
            debug!(target: BEING_SYSTEM, "Canceled despawn for chunk {:?} because at least one resident must stay loaded", msg.chunk_ent);
            continue;
        }

        for &being_ent in beings_within_chunk.entities() {
            if beings.contains(being_ent) {
                unload_being_for_chunk_despawn(&mut commands, being_ent);
            }
        }

        commands.entity(msg.chunk_ent).try_despawn();
        debug!(target: BEING_SYSTEM, "Despawned chunk {:?} and backgrounded {} resident beings", msg.chunk_ent, beings_within_chunk.entities().len());
    }
}

pub fn extend_retained_chasepaths_for_moving_player_prey(
    time: Res<Time>,
    mut chasers: Query<
        (
            Entity,
            &Chaser,
            &GlobalTilePos,
            &ChunkPos,
            &DimensionRef,
            Option<&SpeedMagnitude>,
            &mut RetainedChasePathSnapshot,
            &mut ActivatingChunks,
        ),
        (With<Being>, LocalAiControlled, Without<ActivateChunksAround>),
    >,
    targets: Query<(&ChunkPos, &DimensionRef, Has<BelongsToAPlayerFaction>), (With<Being>, Changed<ChunkPos>)>,
    plans: Res<ChaserNavPlans>,
    mut corridor_chunks: Local<Vec<ChunkPos>>,
    mut seen_chunks: Local<HashSet<ChunkPos>>,
) {
    for (chaser_ent, chaser, chaser_gpos, &chaser_chunk_pos, &chaser_dim, speed, mut snapshot, mut activating_chunks) in chasers.iter_mut() {
        let Ok((&target_chunk_pos, &target_dim, target_is_player_faction)) = targets.get(chaser.target) else {
            continue;
        };
        if !target_is_player_faction || target_dim != chaser_dim {
            continue;
        }

        let stale_timeout = retained_target_trail_stale_timeout(speed.map_or(1.0, |speed| speed.0));

        if snapshot.target_chunk_trail.is_empty() {
            extend_target_chunk_trail(&mut snapshot.target_chunk_trail, target_chunk_pos, stale_timeout);
            snapshot.last_target_chunk_pos = target_chunk_pos;
        } else if target_chunk_pos != snapshot.last_target_chunk_pos {
            extend_target_chunk_trail(&mut snapshot.target_chunk_trail, target_chunk_pos, stale_timeout);
            snapshot.last_target_chunk_pos = target_chunk_pos;
        }

        rebuild_connected_chase_chunk_path(
            &mut corridor_chunks,
            &mut seen_chunks,
            Some(*chaser_gpos),
            plans.by_ent.get(&chaser_ent).map(|plan| plan.path_tiles.as_slice()),
            chaser_chunk_pos,
            target_chunk_pos,
        );

        let mut chunk_positions = std::mem::take(&mut snapshot.chunk_positions);
        rebuild_retained_chase_chunk_positions(
            &mut chunk_positions,
            &corridor_chunks,
            &mut seen_chunks,
            &mut snapshot.target_chunk_trail,
            stale_timeout,
            time.delta(),
        );
        snapshot.chunk_positions = chunk_positions;

        if activating_chunks.chunk_positions != snapshot.chunk_positions {
            activating_chunks.chunk_positions.clear();
            activating_chunks
                .chunk_positions
                .extend(snapshot.chunk_positions.iter().copied());
        }

        debug!(target: BEING_SYSTEM, "Updated retained chase snapshot for {:?} to prey chunk {:?}, retained {}, trail {}", chaser_ent, target_chunk_pos, snapshot.chunk_positions.len(), snapshot.target_chunk_trail.len());
    }
}
