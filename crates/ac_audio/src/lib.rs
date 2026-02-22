use std::{fs, path::Path};

use bevy::prelude::*;
use common::log_targets::SPRITE_ANIMATION_INIT;
use serde::Deserialize;

#[derive(Resource, Clone, Debug, Deserialize)]
pub struct SpatialAudioSettings {
    #[serde(default = "default_pixels_per_meter")]
    pub pixels_per_meter: f32,
    #[serde(default = "default_reference_distance_m")]
    pub reference_distance_m: f32,
    #[serde(default = "default_max_distance_m")]
    pub max_distance_m: f32,
    #[serde(default = "default_rolloff_exponent")]
    pub rolloff_exponent: f32,
}

impl Default for SpatialAudioSettings {
    fn default() -> Self {
        Self {
            pixels_per_meter: default_pixels_per_meter(),
            reference_distance_m: default_reference_distance_m(),
            max_distance_m: default_max_distance_m(),
            rolloff_exponent: default_rolloff_exponent(),
        }
    }
}

impl SpatialAudioSettings {
    pub fn gain_for_distance_px(&self, distance_px: f32) -> f32 {
        let ppm = self.pixels_per_meter.max(0.001);
        let distance_m = (distance_px / ppm).max(0.0);
        if distance_m <= self.reference_distance_m {
            return 1.0;
        }
        if distance_m >= self.max_distance_m {
            return 0.0;
        }
        let exponent = self.rolloff_exponent.max(0.0);
        let gain = (self.reference_distance_m / distance_m).powf(exponent);
        gain.clamp(0.0, 1.0)
    }
}

fn default_pixels_per_meter() -> f32 { 32.0 }
fn default_reference_distance_m() -> f32 { 1.0 }
fn default_max_distance_m() -> f32 { 40.0 }
fn default_rolloff_exponent() -> f32 { 2.0 }

pub fn load_spatial_audio_settings(mut settings: ResMut<SpatialAudioSettings>) {
    let path = Path::new("assets/ron/audio/settings.ron");
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(loaded) = ron::from_str::<SpatialAudioSettings>(&contents) else {
        error!(target: SPRITE_ANIMATION_INIT, "Failed parsing '{}'", path.display());
        return;
    };
    *settings = loaded;
}
