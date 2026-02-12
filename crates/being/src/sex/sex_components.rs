#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::Prefix;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize)]
#[require(Replicated, Prefix::trunc("Sex"))]
pub struct Sex;

#[derive(Component, Debug, Default, )]
pub struct SexWeight(pub f32);
