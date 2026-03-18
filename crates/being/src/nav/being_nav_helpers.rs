use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use ::tilemap_shared::{ChunkPos, GlobalTilePos};
use super::being_nav_structs::AiNavGridCache;
use super::being_nav_components::RetainedTargetChunkTrailEntry;
use std::time::Duration;

pub const RETAINED_TARGET_TRAIL_STALE_TIMEOUT_MIN_SECS: f32 = 1.5;
pub const RETAINED_TARGET_TRAIL_STALE_TIMEOUT_BASE_SECS: f32 = 5.0;
pub const RETAINED_TARGET_TRAIL_STALE_TIMEOUT_MAX_SECS: f32 = 12.0;

fn new_retained_target_chunk_trail_entry(
    chunk_pos: ChunkPos,
    stale_timeout: Duration,
) -> RetainedTargetChunkTrailEntry {
    RetainedTargetChunkTrailEntry {
        chunk_pos,
        stale_timer: Timer::new(stale_timeout, TimerMode::Once),
    }
}

pub fn retained_target_trail_stale_timeout(chaser_speed: f32) -> Duration {
    let clamped_speed = chaser_speed.max(0.1);
    Duration::from_secs_f32(
        (RETAINED_TARGET_TRAIL_STALE_TIMEOUT_BASE_SECS / clamped_speed)
            .clamp(
                RETAINED_TARGET_TRAIL_STALE_TIMEOUT_MIN_SECS,
                RETAINED_TARGET_TRAIL_STALE_TIMEOUT_MAX_SECS,
            ),
    )
}

fn sync_target_chunk_trail_timeouts(
    target_chunk_trail: &mut [RetainedTargetChunkTrailEntry],
    stale_timeout: Duration,
) {
    for entry in target_chunk_trail.iter_mut() {
        let elapsed = entry.stale_timer.elapsed().min(stale_timeout);
        entry.stale_timer.set_duration(stale_timeout);
        entry.stale_timer.set_elapsed(elapsed);
    }
}

pub fn rebuild_shortest_connected_chunk_path(chunk_positions: &mut Vec<ChunkPos>, start: ChunkPos, end: ChunkPos) {
    chunk_positions.clear();
    chunk_positions.push(start);
    append_chunk_path_suffix(chunk_positions, start, end);
}

pub fn append_chunk_path_suffix(chunk_positions: &mut Vec<ChunkPos>, start: ChunkPos, end: ChunkPos) {
    let mut current = start;
    if current == end {
        if chunk_positions.last().copied() != Some(end) {
            chunk_positions.push(end);
        }
        return;
    }

    while current != end {
        let delta = end.0 - current.0;
        let step = cardinal_step_toward(delta);
        if step == IVec2::ZERO {
            break;
        }
        current = ChunkPos(current.0 + step);
        if chunk_positions.last().copied() != Some(current) {
            chunk_positions.push(current);
        }
    }
}

pub fn retained_target_chunk_trails_match_positions(
    lhs: &[RetainedTargetChunkTrailEntry],
    rhs: &[RetainedTargetChunkTrailEntry],
) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .map(|entry| entry.chunk_pos)
            .eq(rhs.iter().map(|entry| entry.chunk_pos))
}

pub fn extend_target_chunk_trail(
    target_chunk_trail: &mut Vec<RetainedTargetChunkTrailEntry>,
    next_target_chunk_pos: ChunkPos,
    stale_timeout: Duration,
) {
    sync_target_chunk_trail_timeouts(target_chunk_trail, stale_timeout);
    let Some(last_target_chunk_pos) = target_chunk_trail.last().map(|entry| entry.chunk_pos) else {
        target_chunk_trail.push(new_retained_target_chunk_trail_entry(next_target_chunk_pos, stale_timeout));
        return;
    };

    let mut current = last_target_chunk_pos;
    if current == next_target_chunk_pos {
        if let Some(last_entry) = target_chunk_trail.last_mut() {
            last_entry.stale_timer.reset();
        }
        return;
    }

    while current != next_target_chunk_pos {
        let delta = next_target_chunk_pos.0 - current.0;
        let step = cardinal_step_toward(delta);
        if step == IVec2::ZERO {
            break;
        }
        let chunk_pos = ChunkPos(current.0 + step);
        if target_chunk_trail.last().map(|entry| entry.chunk_pos) == Some(chunk_pos) {
            current = chunk_pos;
            continue;
        }
        target_chunk_trail.push(new_retained_target_chunk_trail_entry(chunk_pos, stale_timeout));
        current = chunk_pos;
    }
}

pub fn rebuild_connected_chase_chunk_path(
    connected_path_chunks: &mut Vec<ChunkPos>,
    seen_chunks: &mut HashSet<ChunkPos>,
    chaser_gpos: Option<GlobalTilePos>,
    nav_path_tiles: Option<&[GlobalTilePos]>,
    chaser_chunk_pos: ChunkPos,
    target_chunk_pos: ChunkPos,
) {
    connected_path_chunks.clear();
    if let (Some(chaser_gpos), Some(nav_path_tiles)) = (chaser_gpos, nav_path_tiles.filter(|tiles| !tiles.is_empty())) {
        seen_chunks.clear();
        collect_retained_path_chunks(chaser_gpos, nav_path_tiles, seen_chunks, connected_path_chunks);
        return;
    }

    rebuild_shortest_connected_chunk_path(connected_path_chunks, chaser_chunk_pos, target_chunk_pos);
}

pub fn rebuild_retained_chase_chunk_positions(
    chunk_positions: &mut Vec<ChunkPos>,
    connected_path_chunks: &[ChunkPos],
    seen_chunks: &mut HashSet<ChunkPos>,
    target_chunk_trail: &mut Vec<RetainedTargetChunkTrailEntry>,
    stale_timeout: Duration,
    trail_age_delta: Duration,
) {
    sync_target_chunk_trail_timeouts(target_chunk_trail, stale_timeout);

    seen_chunks.clear();
    seen_chunks.extend(connected_path_chunks.iter().copied());

    for entry in target_chunk_trail.iter_mut() {
        entry.stale_timer.tick(trail_age_delta);
    }
    if let Some(last_entry) = target_chunk_trail.last_mut() {
        last_entry.stale_timer.reset();
    }

    let mut retained_start_ix = 0usize;
    for ix in (0..target_chunk_trail.len().saturating_sub(1)).rev() {
        if seen_chunks.contains(&target_chunk_trail[ix].chunk_pos) {
            retained_start_ix = ix;
            break;
        }
    }
    if retained_start_ix > 0 {
        target_chunk_trail.drain(..retained_start_ix);
    }

    while target_chunk_trail.len() > 1
        && target_chunk_trail
            .first()
            .is_some_and(|entry| entry.stale_timer.is_finished())
        && target_chunk_trail
            .first()
            .is_some_and(|entry| seen_chunks.contains(&entry.chunk_pos))
    {
        target_chunk_trail.remove(0);
    }

    if target_chunk_trail.len() > 1 {
        let prey_chunk_pos = target_chunk_trail.last().map(|entry| entry.chunk_pos);
        let anchor_chunk_pos = target_chunk_trail.first().map(|entry| entry.chunk_pos);
        if let (Some(prey_chunk_pos), Some(anchor_chunk_pos)) = (prey_chunk_pos, anchor_chunk_pos) {
            let connected_path_end_matches_prey = connected_path_chunks.last().copied() == Some(prey_chunk_pos);
            let anchor_ix = connected_path_chunks.iter().position(|&chunk_pos| chunk_pos == anchor_chunk_pos);
            if connected_path_end_matches_prey {
                if let Some(anchor_ix) = anchor_ix {
                    let connected_suffix_len = connected_path_chunks.len().saturating_sub(anchor_ix);
                    if target_chunk_trail.len() > connected_suffix_len {
                        target_chunk_trail.clear();
                        target_chunk_trail.push(new_retained_target_chunk_trail_entry(prey_chunk_pos, stale_timeout));
                    }
                } else {
                    // The current corridor already reaches the prey chunk, so a stale trail
                    // with no remaining rejoin point is redundant and should be collapsed.
                    target_chunk_trail.clear();
                    target_chunk_trail.push(new_retained_target_chunk_trail_entry(prey_chunk_pos, stale_timeout));
                }
            }
        }
    }

    chunk_positions.clear();
    seen_chunks.clear();

    for &chunk_pos in connected_path_chunks.iter() {
        if seen_chunks.insert(chunk_pos) {
            chunk_positions.push(chunk_pos);
        }
    }

    for entry in target_chunk_trail.iter().rev() {
        if seen_chunks.insert(entry.chunk_pos) {
            chunk_positions.push(entry.chunk_pos);
        }
    }
}

pub fn cardinal_step_toward(delta: IVec2) -> IVec2 {
    if delta == IVec2::ZERO {
        IVec2::ZERO
    } else if delta.x.abs() >= delta.y.abs() {
        IVec2::new(delta.x.signum(), 0)
    } else {
        IVec2::new(0, delta.y.signum())
    }
}

pub fn rebuild_dynamic_blocking(
    dynamic_blocking: &mut HashMap<UVec3, Entity>,
    cache: &AiNavGridCache,
    chaser_ent: Entity,
    target_ent: Entity,
    start: UVec3,
    goal: UVec3,
) {
    dynamic_blocking.clear();
    dynamic_blocking.reserve(cache.occupied.len());
    for (&pos, &ent) in cache.occupied.iter() {
        if ent == chaser_ent || ent == target_ent {
            continue;
        }
        dynamic_blocking.insert(pos, ent);
    }
    dynamic_blocking.remove(&goal);
    dynamic_blocking.remove(&start);
}

pub fn sync_chase_retained_chunks(
    activating_chunks: Option<Mut<tilemap::chunking::chunking_components::ActivatingChunks>>,
    desired_chunks: &[ChunkPos],
    chaser_ent: Entity,
    to_insert: &mut Vec<(Entity, tilemap::chunking::chunking_components::ActivatingChunks)>,
) {
    use tilemap::chunking::chunking_components::ActivatingChunks;
    match activating_chunks {
        Some(mut existing) => {
            if existing.chunk_positions != desired_chunks {
                existing.chunk_positions.clear();
                existing.chunk_positions.extend(desired_chunks.iter().copied());
            }
        }
        None if !desired_chunks.is_empty() => {
            to_insert.push((
                chaser_ent,
                ActivatingChunks {
                    chunk_positions: desired_chunks.to_vec(),
                },
            ));
            debug!(target: common::log_targets::BEING_SYSTEM, "Added chase chunk retention for {:?}, chunks: {:?}", chaser_ent, desired_chunks.len());
        }
        None => {}
    }
}

pub fn collect_retained_path_chunks(
    current_gpos: GlobalTilePos,
    path_tiles: &[GlobalTilePos],
    seen_chunks: &mut HashSet<ChunkPos>,
    desired_chunks: &mut Vec<ChunkPos>,
) {
    let current_chunk_pos = ChunkPos::from(current_gpos);
    if seen_chunks.insert(current_chunk_pos) {
        desired_chunks.push(current_chunk_pos);
    }

    let mut prev_chunk = current_chunk_pos;
    for &step in path_tiles.iter() {
        let step_chunk = ChunkPos::from(step);
        let prev_len = desired_chunks.len();
        append_chunk_path_suffix(desired_chunks, prev_chunk, step_chunk);
        for &chunk_pos in desired_chunks[prev_len..].iter() {
            seen_chunks.insert(chunk_pos);
        }

        prev_chunk = step_chunk;
    }
}
