use ::tilemap_shared::ChunkPos;
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct RetainedTargetChunkTrailEntry {
    pub chunk_pos: ChunkPos,
    pub stale_timer: Timer,
}

#[derive(Component, Debug, Clone, Default)]
pub struct RetainedChasePathSnapshot {
    pub chunk_positions: Vec<ChunkPos>,
    pub target_chunk_trail: Vec<RetainedTargetChunkTrailEntry>,
    pub last_target_chunk_pos: ChunkPos,
}
