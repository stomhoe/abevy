use bevy::{ecs::entity::MapEntities, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::{Prefix, StrId};
use game_common::game_common_components_samplers::EntityWeightedSampler;
use serde::{Deserialize, Serialize};



#[derive(Component, Debug, PartialEq, Eq, Hash, Clone)]
#[require(Replicated, Prefix::trunc("Race"))]
pub struct Race;



#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, MapEntities)]
pub struct SpritesPool(#[entities] pub Vec<Entity>);

//Usar DisplayName para cada grupo

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, MapEntities)]
pub struct PlayerSelectableSprites(#[entities] pub Vec<Entity>);


#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, MapEntities)]
pub struct SexesSampler(#[entities] pub EntityWeightedSampler);
impl SexesSampler {
    pub fn new(weights: &Vec<(Entity, f32)>) -> Self {
        Self(EntityWeightedSampler::new(&weights))
    }
}

