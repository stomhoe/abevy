use bevy::{ecs::entity::{EntityHashMap, MapEntities}, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::{AssetScoped, HotReload, Prefix};
use game_common::game_common_samplers::{CappedNormalDist, EntityWeightedSampler};

#[derive(Component, serde::Serialize, serde::Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Pack"), AssetScoped, HotReload)]
pub struct Pack;

#[derive(Component, Debug, Clone, MapEntities, Default)]
#[component(map_entities)]
pub struct PackBeingSampler(#[entities] pub EntityWeightedSampler);
impl PackBeingSampler {
    pub fn insert(&mut self, entity: Entity, weight: f32) {
        self.0.insert(entity, weight);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct PackBeingLeaderPriority(pub EntityHashMap<f32>);
impl PackBeingLeaderPriority {
    pub fn insert(&mut self, entity: Entity, leader_priority: f32) {
        self.0.insert(entity, leader_priority);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct PackBehavior(pub String);

#[derive(Component, Debug, Clone)]
pub struct PackInitialSize(pub CappedNormalDist);
impl PackInitialSize {
    pub fn sample_count(&self, rng: &mut impl rand::Rng) -> usize {
        self.0.sample(rng).round().max(1.0) as usize
    }
}
