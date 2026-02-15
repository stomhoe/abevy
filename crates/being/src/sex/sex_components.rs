#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::Prefix;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Sex"))]
pub struct Sex;

#[derive(Component, Debug, Default, Clone)]
pub struct SexWeight(pub f32);
