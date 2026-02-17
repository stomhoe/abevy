use bevy::{platform::collections::HashSet, prelude::*};
use common::{common_components::HashId, common_tag_components::HashedTagsVec};

use serde::Deserialize;
use ::tilemap_shared::*;
use std::{f32::{INFINITY, NEG_INFINITY}, hash::Hash};




#[derive(Message, Debug, Clone)]
pub struct TerrainProbe {
    pub dimension_ref: DimensionRef,
    pub search_start_pos: GlobalTilePos,
    pub opfilter_ent: Entity,
    pub probe_pattern: ProbePattern,
    pub step_size: u16,
    pub curr_iteration_batch_i: i16,//se puede cambiar a otra cosa para empezar alejado del centro
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub max_emitted_results: u16,
}
impl TerrainProbe{
    pub fn standard_spiral_probe(dimension_ref: DimensionRef, operation_filter: Entity, search_start_pos: GlobalTilePos) -> TerrainProbe {
        TerrainProbe {
            dimension_ref,
            search_start_pos,
            opfilter_ent: operation_filter,
            probe_pattern: ProbePattern::spiral(search_start_pos),
            max_batches: 1000,
            ..Default::default()
        }
    }
    pub fn standard_sun_probe(dimension_ref: DimensionRef, operation_filter: Entity, search_start_pos: GlobalTilePos) -> TerrainProbe {
        TerrainProbe {
            dimension_ref,
            search_start_pos,
            opfilter_ent: operation_filter,
            probe_pattern: ProbePattern::sun(),
            ..Default::default()
        }
    }
}
impl Default for TerrainProbe {
    fn default() -> Self {
        TerrainProbe {
            dimension_ref: DimensionRef(Entity::PLACEHOLDER),
            search_start_pos: GlobalTilePos::default(),
            probe_pattern: ProbePattern::spiral(GlobalTilePos::default()),
            step_size: 1,
            curr_iteration_batch_i: 0,
            max_batches: 1000,
            iterations_per_batch: 10000,
            opfilter_ent: Entity::PLACEHOLDER,
            max_emitted_results: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProbePattern {
    Radial(Option<f32>),
    /// curr_length_in_dir, steps_taken, dir_vec, pos, turn parity
    Spiral(u64, u64, IVec2, GlobalTilePos, bool),
}
impl ProbePattern {
    pub fn sun() -> Self { ProbePattern::Radial(None) }
    pub fn spiral(start_pos: GlobalTilePos) -> Self {
        ProbePattern::Spiral(1, 0, IVec2::new(0, 1), start_pos, false)
    }
}


#[derive(Debug, Clone, Message, )]
pub struct SuitablePosFound { pub op_filter_ent: Entity, pub val: f32, pub found_pos: GlobalTilePos, }


#[derive(Debug, Clone, Message, )]
pub struct SearchFailed (pub Entity);

#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: DimensionRootOplist, pub dimension_ref: DimensionRef, pub gpos: GlobalTilePos,
    pub filtered_op: Entity, pub max_emitted_results: u16}
