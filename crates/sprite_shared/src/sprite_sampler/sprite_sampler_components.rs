#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Copy, Clone)]
#[require(AssetScoped, Prefix::trunc("SpriteWSampler"))]
pub struct SpriteWeightedSampler;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(AssetScoped, Prefix::trunc("SpriteSamplerHolder"))]
pub struct EguiSpriteSamplerHolder;
