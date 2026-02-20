use bevy::ecs::entity::{EntityHashSet, MapEntities};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(
    SparedFromHotReloading,
    AssetScoped,
    Replicated,
    Prefix::trunc("BodyTree")
)]
pub struct BodyTree;

#[derive(Component, Debug, Clone)]
#[relationship(relationship_target = Bodies)]
pub struct BodyOf {
    #[relationship]
    #[entities]
    pub being: Entity,
}

#[derive(Component, Debug, Reflect, Clone)]
#[relationship_target(relationship = BodyOf)]
pub struct Bodies(Vec<Entity>);
impl Bodies {
    pub fn entities(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component, Debug, Clone, MapEntities)]
pub struct BodyTreeToBuild(#[entities] pub Entity);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyTreeMassKg(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BeingMassKg(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyTreeDistributedTotals {
    pub hp_capacity: f32,
    pub hp_regen_rate: f32,
    pub blood_capacity: f32,
    pub blood_pumping: f32,
    pub walk_speed: f32,
    pub swim_speed: f32,
    pub fly_speed: f32,
    pub manipulation: f32,
    pub vision: f32,
    pub pain_sensitivity: f32,
    pub caloric_burn_rate: f32,
    pub caloric_capacity: f32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct BodyDead;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Message)]
pub struct BodyDamage {
    pub body: Entity,
    pub amount: f32,
}
