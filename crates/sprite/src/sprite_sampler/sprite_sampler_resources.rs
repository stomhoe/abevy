use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct SpriteWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>, // sprite_id or sampler_id* with weight
}

pub use crate::sprite_sampler::SpriteWeightedSamplerHandles;
