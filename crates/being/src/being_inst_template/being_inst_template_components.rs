use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::Prefix;
use game_common::game_common_components_samplers::EntityWeightedSampler;
use serde::{Deserialize, Serialize};
use sprite::SpriteCfgEntityMap;



#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect, )]
pub struct BitHealthMultiplier(pub f32);