#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::Prefix;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize, )]
#[require(Replicated, Prefix::trunc("Sex"))]
pub struct Sex;

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct SexWeight(pub f32);


