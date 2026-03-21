#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use bevy::ecs::entity::EntityHashMap;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Copy, Clone, Deserialize, Serialize)]
#[require(AssetScoped, Replicated, Prefix::trunc("Biome"), HotReload)]
pub struct Biome;

#[derive(Component, Debug, Clone, Default, )]
pub struct CreatureSampler(pub EntityHashMap<f32>);

impl CreatureSampler {
    pub fn add_affinity(&mut self, biome_ent: Entity, weight: f32) {
        if weight == 0.0 {
            return;
        }
        *self.0.entry(biome_ent).or_insert(0.0) += weight;
    }
}
