use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ::tilemap_shared::*;

#[derive(Message, Debug, Clone)]
pub struct TerrProbeJob {
    pub requester: Entity,
    pub dimension_ref: DimensionRef,
    pub search_start_pos: GlobalTilePos,
    pub templ_ent: Entity,
    pub structuregen_whitelist: Vec<Entity>,
    pub structuregen_blacklist: Vec<Entity>,
    pub min_result_distance: u16,
    pub curr_iteration_batch_i: i16,
}
impl Default for TerrProbeJob {
    fn default() -> Self {
        TerrProbeJob {
            requester: Entity::PLACEHOLDER,
            dimension_ref: DimensionRef(Entity::PLACEHOLDER),
            search_start_pos: GlobalTilePos::default(),
            templ_ent: Entity::PLACEHOLDER,
            structuregen_whitelist: Vec::new(),
            structuregen_blacklist: Vec::new(),
            min_result_distance: 0,
            curr_iteration_batch_i: 0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProbePattern {
    Concentric {
        radius_step: f32,
        sample_spacing: f32,
    },
    /// curr_length_in_dir, steps_taken, dir_vec, pos, turn parity
    Spiral(u64, u64, IVec2, GlobalTilePos, bool),
    Chunk(ChunkPos),
    Region(u16),
}
impl ProbePattern {
    pub fn spiral(start_pos: GlobalTilePos) -> Self {
        ProbePattern::Spiral(1, 0, IVec2::new(0, 1), start_pos, false)
    }
    pub fn concentric(radius_step: f32, sample_spacing: f32) -> Self {
        ProbePattern::Concentric {
            radius_step: radius_step.max(0.0001),
            sample_spacing: sample_spacing.max(0.0001),
        }
    }
    pub fn chunk(chunk_pos: ChunkPos) -> Self {
        ProbePattern::Chunk(chunk_pos)
    }
    pub fn region(spacing: u16) -> Self {
        ProbePattern::Region(spacing.max(1))
    }
}

#[derive(Debug, Clone, Message)]
pub struct SuitablePosFound {
    pub requester: Entity,
    pub val: f32,
    pub found_pos: GlobalTilePos,
}

#[derive(Debug, Clone, Message)]
pub struct SearchFailed(pub Entity);
