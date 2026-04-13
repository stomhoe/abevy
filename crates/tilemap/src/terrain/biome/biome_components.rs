#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use tilemap_shared::*;
use common::common_types::HashIdToEntityMap;

#[derive(Component, Debug, Default, Copy, Clone, Deserialize, Serialize)]
#[require(AssetScoped, Replicated, Prefix::trunc("Biome"), SelectedForHotReload)]
pub struct Biome;

#[derive(Component, Debug, Clone, Default, )]
pub struct CreatureSampler(pub HashIdWeightedSampler);

impl CreatureSampler {
    pub fn add_affinity(&mut self, biome_hash_id: common::common_components::HashId, weight: f32) {
        if weight <= 0.0 {
            return;
        }
        if let Err(negative_item) = self.0.add_or_accumulate_weight(biome_hash_id, weight) {
            error!(target: "biome_components", "Weighted sampler {:?} encountered a negative weight for value {:?}; rejected", biome_hash_id, negative_item);
        }
    }

    pub fn sample_pack_or_race_or_bit_entity(
        &self,
        rng: &mut impl rand::Rng,
        entity_maps: &[&HashIdToEntityMap],
    ) -> Option<Entity> {
        let sampled_hash_id = self.0.sample_with_rng(rng)?;
        for entity_map in entity_maps {
            if let Some(entity) = entity_map.get_opt(sampled_hash_id).copied() {
                return Some(entity);
            }
        }
        None
    }
}
