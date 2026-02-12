use bevy::prelude::*;
use serde::Deserialize;

use crate::game_common_components::CappedNormalDist;

#[derive(Asset, TypePath, Default, Debug, Clone, Deserialize, )]
pub struct NormalDistSeri {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
}
impl From<CappedNormalDist> for NormalDistSeri {
    fn from(value: CappedNormalDist) -> Self {
        Self {
            min: value.min,
            max: value.max,
            mean: value.mean,
            std_dev: value.std_dev,
        }
    }
}
