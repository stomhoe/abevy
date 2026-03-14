use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::{AssetScoped, HotReload, Prefix};
use game_common::game_common_samplers::CappedNormalDist;

#[derive(Component, serde::Serialize, serde::Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Pack"), AssetScoped, HotReload)]
pub struct Pack;

#[derive(Component, Debug, Clone, MapEntities, Default)]
pub struct PackRaceMembers(#[entities] pub Vec<Entity>);

#[derive(Component, Debug, Clone, MapEntities, Default)]
pub struct PackBitMembers(#[entities] pub Vec<Entity>);

#[derive(Component, Debug, Clone, Default)]
pub struct PackBehavior(pub String);

#[derive(Component, Debug, Clone)]
pub struct PackInitialSpawnNormalDist(pub CappedNormalDist);
impl PackInitialSpawnNormalDist {
    pub fn sample_count(&self, rng: &mut impl rand::Rng) -> usize {
        self.0.sample(rng).round().max(1.0) as usize
    }
}
