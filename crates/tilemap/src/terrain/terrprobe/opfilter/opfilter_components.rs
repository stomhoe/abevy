use bevy::prelude::*;
use common::{common_components::HashId, common_tag_components::HashedTagsVec};

use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct OpFilter{
    pub tags: HashedTagsVec,
    pub var_name_hash: Option<HashId>,
    pub min_val: f32,
    pub max_val: f32,
}
impl Hash for OpFilter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tags.hash(state);
        self.var_name_hash.hash(state);
        self.min_val.to_bits().hash(state);
        self.max_val.to_bits().hash(state);
    }
}
impl PartialEq for OpFilter {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags &&
        self.var_name_hash == other.var_name_hash &&
        self.min_val.to_bits() == other.min_val.to_bits() &&
        self.max_val.to_bits() == other.max_val.to_bits()
    }
}
impl Eq for OpFilter {}
