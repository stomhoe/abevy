use bevy::prelude::*;
use common::common_tag_components::HashedTagsVec;

use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct OpFilter{
    pub tags: HashedTagsVec,
    pub op_i: Option<u16>,
    pub min_val: f32,
    pub max_val: f32,
}
impl Hash for OpFilter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tags.hash(state);
        self.op_i.hash(state);
        self.min_val.to_bits().hash(state);
        self.max_val.to_bits().hash(state);
    }
}
impl PartialEq for OpFilter {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags &&
        self.op_i == other.op_i &&
        self.min_val.to_bits() == other.min_val.to_bits() &&
        self.max_val.to_bits() == other.max_val.to_bits()
    }
}
impl Eq for OpFilter {}
