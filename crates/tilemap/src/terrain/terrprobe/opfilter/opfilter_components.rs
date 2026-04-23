use bevy::prelude::*;
use common::{common_components::{HashId, HashIdMap}, common_tag_components::HashedTagsVec};

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
impl OpFilter {
    pub fn passes_filter_value(&self, computed_vars: &HashIdMap<f32>, output_value: f32) -> bool {
        if let Some(var_name_hash) = self.var_name_hash {
            let Ok(val) = computed_vars.get(var_name_hash) else {
                return false;
            };
            return (self.min_val..=self.max_val).contains(val);
        }

        (self.min_val..=self.max_val).contains(&output_value)
    }

    pub fn sampled_value_from_filter(&self, computed_vars: &HashIdMap<f32>, output_value: f32) -> f32 {
        if let Some(var_name_hash) = self.var_name_hash
            && let Ok(val) = computed_vars.get(var_name_hash)
        {
            return *val;
        }
        output_value
    }
}

impl Eq for OpFilter {}
