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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SampledValues {
    pub values: Vec<(GlobalTilePos, Option<f32>)>,
}
impl SampledValues {
    pub fn new(min: GlobalTilePos, matrix_size: UVec2, spacing: u16) -> Self {
        let spacing = spacing.max(1) as i32;
        let mut values = Vec::with_capacity((matrix_size.x * matrix_size.y) as usize);
        for row in 0..matrix_size.y {
            for col in 0..matrix_size.x {
                let gpos = GlobalTilePos(min.0 + IVec2::new(col as i32 * spacing, row as i32 * spacing));
                values.push((gpos, None));
            }
        }
        Self { values }
    }

    pub fn flat_index(&self, gpos: GlobalTilePos) -> Option<usize> {
        self.values
            .iter()
            .position(|(sample_pos, _)| *sample_pos == gpos)
    }

    pub fn get(&self, gpos: GlobalTilePos) -> Option<Option<f32>> {
        self.flat_index(gpos)
            .and_then(|i| self.values.get(i).map(|(_, val)| *val))
    }

    pub fn set(&mut self, gpos: GlobalTilePos, value: Option<f32>) -> bool {
        let Some(i) = self.flat_index(gpos) else {
            return false;
        };
        self.values[i].1 = value;
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
