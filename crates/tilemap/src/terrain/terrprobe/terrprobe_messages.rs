use bevy::prelude::*;
use common::common_components::HashId;
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
    pub collect_all_successes: bool,
    pub min_result_distance: u16,
    pub curr_iteration_batch_i: i16,
}
impl Default for TerrProbeJob {
    fn default() -> Self {
        TerrProbeJob {
            requester: Entity::PLACEHOLDER,
            dimension_ref: DimensionRef(HashId::default()),
            search_start_pos: GlobalTilePos::default(),
            templ_ent: Entity::PLACEHOLDER,
            structuregen_whitelist: Vec::new(),
            structuregen_blacklist: Vec::new(),
            collect_all_successes: false,
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
    Chunk(ChunkPos),
    Region {
        spacing: u16,
        region_multiplier: f32,
    },
}
impl ProbePattern {
    pub fn concentric(radius_step: f32, sample_spacing: f32) -> Self {
        ProbePattern::Concentric {
            radius_step: radius_step.max(0.0001),
            sample_spacing: sample_spacing.max(0.0001),
        }
    }
    pub fn chunk(chunk_pos: ChunkPos) -> Self {
        ProbePattern::Chunk(chunk_pos)
    }
    pub fn region(spacing: u16, region_multiplier: f32) -> Self {
        ProbePattern::Region {
            spacing: spacing.max(1),
            region_multiplier: region_multiplier.max(0.0001),
        }
    }
}

#[derive(Debug, Clone, Message)]
pub struct SuitablePosFound {
    pub requester: Entity,
    pub val: f32,
    pub found_pos: GlobalTilePos,
    pub is_last: bool,
}

#[derive(Debug, Clone)]
pub struct SampledValues {
    pub anchor_gpos: GlobalTilePos,
    pub matrix_size: UVec2,
    pub spacing: u16,
    pub values: Vec<Option<f32>>,
}
impl SampledValues {
    pub fn new(anchor_gpos: GlobalTilePos, matrix_size: UVec2, spacing: u16) -> Self {
        let spacing = spacing.max(1);
        let capacity = (matrix_size.x * matrix_size.y) as usize;
        let values = vec![None; capacity];
        Self {
            anchor_gpos,
            matrix_size,
            spacing,
            values,
        }
    }

    fn position_to_index(&self, gpos: GlobalTilePos) -> Option<usize> {
        let delta = gpos.0 - self.anchor_gpos.0;
        let spacing = self.spacing as i32;
        if spacing <= 0 {
            return None;
        }
        if delta.x % spacing != 0 || delta.y % spacing != 0 {
            return None;
        }
        let col = (delta.x / spacing) as i32;
        let row = (delta.y / spacing) as i32;
        if col < 0 || row < 0 {
            return None;
        }
        let col = col as u32;
        let row = row as u32;
        if col >= self.matrix_size.x || row >= self.matrix_size.y {
            return None;
        }
        Some((row * self.matrix_size.x + col) as usize)
    }

    pub fn flat_index(&self, gpos: GlobalTilePos) -> Option<usize> {
        self.position_to_index(gpos)
    }

    pub fn index_to_gpos(&self, index: usize) -> Option<GlobalTilePos> {
        if self.matrix_size.x == 0 {
            return None;
        }
        let col = (index as u32) % self.matrix_size.x;
        let row = (index as u32) / self.matrix_size.x;
        if row >= self.matrix_size.y {
            return None;
        }
        let offset = IVec2::new(
            col as i32 * self.spacing as i32,
            row as i32 * self.spacing as i32,
        );
        Some(GlobalTilePos(self.anchor_gpos.0 + offset))
    }

    pub fn iter(&self) -> impl Iterator<Item = (GlobalTilePos, Option<f32>)> + '_ {
        self.values.iter().enumerate().filter_map(move |(i, value)| {
            let Some(gpos) = self.index_to_gpos(i) else {
                return None;
            };
            Some((gpos, *value))
        })
    }

    pub fn get(&self, gpos: GlobalTilePos) -> Option<Option<f32>> {
        self.position_to_index(gpos)
            .and_then(|i| self.values.get(i).copied())
    }

    pub fn set(&mut self, gpos: GlobalTilePos, value: Option<f32>) -> bool {
        let Some(i) = self.position_to_index(gpos) else {
            return false;
        };
        let Some(slot) = self.values.get_mut(i) else {
            return false;
        };
        *slot = value;
        true
    }
}

#[derive(Debug, Clone, Message)]
pub struct SampledValuesCollected {
    pub requester: Entity,
    pub matrix: SampledValues,
}

#[derive(Debug, Clone, Message)]
pub struct SearchFailed(pub Entity);
