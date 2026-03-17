use bevy::prelude::*;
use ::tilemap_shared::{ChunkPos, GlobalTilePos};
use bevy::platform::collections::HashMap;
use super::being_nav_structs::AiNavGridCache;

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
