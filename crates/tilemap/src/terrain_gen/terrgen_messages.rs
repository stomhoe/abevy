use bevy::{prelude::*};
use common::{common_components::HashId, common_tag_components::HashedTagsVec};

use serde::Deserialize;
use ::tilemap_shared::*;
use std::hash::Hash;

#[derive(Debug, Clone, Component, TypePath)]
pub struct OpFilter{
    pub start_oplist: Entity,
    pub tags: HashedTagsVec,
    pub op_i: i16,
    pub min_val: f32,
    pub max_val: f32,
    pub search_start_pos: GlobalTilePos,
}
impl Hash for OpFilter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start_oplist.hash(state);
        self.tags.hash(state);
        self.op_i.hash(state);
        self.min_val.to_bits().hash(state);
        self.max_val.to_bits().hash(state);
    }
}
impl PartialEq for OpFilter {
    fn eq(&self, other: &Self) -> bool {
        self.start_oplist == other.start_oplist &&
        self.tags == other.tags &&
        self.op_i == other.op_i &&
        self.min_val.to_bits() == other.min_val.to_bits() &&
        self.max_val.to_bits() == other.max_val.to_bits()
    }
}
impl Eq for OpFilter {}
#[derive(Deserialize, Asset, TypePath, )]
pub struct OpFilterSeri {
    pub tags: Vec<String>,
    pub op_i: i16,
    pub min_val: f32,
    pub max_val: f32,
}

#[derive(Message, Debug, Clone)]
pub struct TerrainProbe {
    pub dimension_ref: DimensionRef,
    pub operation_filter: Entity,
    pub step_size: u16,
    pub curr_iteration_batch_i: i16,//se puede cambiar a otra cosa para empezar alejado del centro
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub probe_pattern: ProbePattern,
}
impl TerrainProbe{
    pub fn standard_spiral_probe(dimension_ref: DimensionRef, operation_filter: Entity, search_start_pos: GlobalTilePos) -> TerrainProbe {
        TerrainProbe {
            dimension_ref,
            step_size: 1,
            curr_iteration_batch_i: 0,
            max_batches: 100,
            iterations_per_batch: 10000,
            probe_pattern: ProbePattern::new_spiral(search_start_pos),
            operation_filter,
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
    pub fn new_radial() -> Self { ProbePattern::Radial(None) }
    pub fn new_spiral(start_pos: GlobalTilePos) -> Self {
        ProbePattern::Spiral(1, 0, IVec2::new(0, 1), start_pos, false)
    }
}


#[derive(Debug, Clone, Message, )]
pub struct SuitablePosFound { pub op_filter_ent: Entity, pub val: f32, pub found_pos: GlobalTilePos, }


#[derive(Debug, Clone, Message, )]
pub struct SearchFailed (pub Entity);

#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: Entity, pub dimension_ref: DimensionRef, pub gpos: GlobalTilePos,
    pub filtered_op: Entity
}
