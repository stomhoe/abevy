use bevy::prelude::*;
use serde::Deserialize;

use crate::game_common_samplers::CappedNormalDist;


#[derive(Asset, TypePath, Debug, Clone, Deserialize, )]
pub struct NormalDistSeri {
    pub min_dev: f32,
    pub max_dev: f32,
    pub mean: f32,
    pub std_dev: f32,
}
impl NormalDistSeri {
    pub fn sentinel() -> Self {
        Self {
            min_dev: f32::NAN,
            max_dev: f32::NAN,
            mean: f32::NAN,
            std_dev: f32::NAN,
        }
    }
    pub fn is_sentinel(&self) -> bool {
        self.min_dev.is_nan() && self.max_dev.is_nan() && self.mean.is_nan() && self.std_dev.is_nan()
    }
    pub fn disabled() -> Self {
        Self::default()
    }
    #[inline]
    pub fn is_disabled(&self) -> bool {
        self.min_dev == 0.0 && self.max_dev == 0.0 && self.mean == 0.0 && self.std_dev == 0.0
    }
}
impl Default for NormalDistSeri {
    fn default() -> Self {
        Self {
            min_dev: f32::NAN,
            max_dev: f32::NAN,
            mean: f32::NAN,
            std_dev: f32::NAN,
        }
    }
}
impl From<CappedNormalDist> for NormalDistSeri {
    fn from(value: CappedNormalDist) -> Self {
        Self {
            min_dev: value.min_dev,
            max_dev: value.max_dev,
            mean: value.mean,
            std_dev: value.std_dev,
        }
    }
}
