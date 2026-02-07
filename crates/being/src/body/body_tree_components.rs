use bevy::ecs::entity::{EntityHashSet, MapEntities};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
#[require(
    SparedFromHotReloading,
    AssetScoped,
    Replicated,
    Prefix::trunc("BodyTree")
)]
pub struct BodyTree;

#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities)]
#[relationship(relationship_target = Bodies)]
pub struct BodyOf {
    #[relationship]
    #[entities]
    pub being: Entity,
}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = BodyOf)]
pub struct Bodies(Vec<Entity>);
impl Bodies {
    pub fn entities(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, MapEntities)]
pub struct BodyTreeToBuild(#[entities] pub Entity);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct BodyHealth {
    pub total_hp: f32,
    pub current_hp: f32,
    pub blood: f32,
    pub blood_capacity: f32,
    pub bleed_rate: f32,
    pub consciousness: f32,
    pub pain: f32,
    pub vision: f32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyDead;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Reflect, Message)]
pub struct BodyDamage {
    pub body: Entity,
    pub amount: f32,
}
