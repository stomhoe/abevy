use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(AssetScoped, Replicated, Prefix::trunc("BodyWSampler"), )]
pub struct BodyWeightedSampler;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(AssetScoped, Replicated, Prefix::trunc("BodySamplerHolder"), )]
pub struct EguiBodySamplerHolder;

#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct SampleBodyFromStrId(StrId);
impl SampleBodyFromStrId {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        Self(StrId::trunc(id.as_ref()))
    }
    pub fn id(&self) -> &StrId {
        &self.0
    }
}

#[derive(Component, Debug, Clone, Reflect, MapEntities)]
pub struct SampleTreeEnt(#[entities]pub Entity);
impl SampleTreeEnt {
    pub fn new(entity: Entity) -> Self { Self(entity) }
    pub fn entity(&self) -> &Entity { &self.0 }
}
