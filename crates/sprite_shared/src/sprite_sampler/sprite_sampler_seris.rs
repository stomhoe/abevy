#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct SpriteWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
}
