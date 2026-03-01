#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct WeightedColorsSeri {
    pub id: String,
    pub weights: Vec<([u8; 4], f32)>,
}
impl WeightedColorsSeri {
    pub const MIN_ID_LENGTH: u8 = 3;
}
