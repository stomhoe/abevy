use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(AssetScoped, Replicated, Prefix::trunc("BodyWSampler"), )]
pub struct BodyWeightedSampler;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(AssetScoped, Replicated, Prefix::trunc("BodySamplerHolder"), )]
pub struct EguiBodySamplerHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct SampleBodyFromStrId(StrId);
impl SampleBodyFromStrId {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        Self(StrId::trunc(id.as_ref()))
    }
    pub fn id(&self) -> &StrId {
        &self.0
    }
}


#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, MapEntities, )]
pub struct SampleBody(#[entities]pub Entity);
impl SampleBody {
    pub fn new(entity: Entity) -> Self { Self(entity) }
    pub fn entity(&self) -> &Entity { &self.0 }
}