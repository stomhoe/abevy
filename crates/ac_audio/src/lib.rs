use std::{fs, path::Path};

use bevy::prelude::*;
use bevy_kira_audio::{AudioPlugin, DefaultSpatialRadius, SpatialAudioPlugin};
use common::common_states::AssetLoading;
use common::log_targets::SPRITE_ANIMATION_INIT;
use serde::Deserialize;

pub mod ac_audio_components;
pub mod ac_audio_systems;

use ac_audio_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AcAudioSystems;

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
    #[serde(default = "default_pan_distance_m")]
    pub pan_distance_m: f32,
    #[serde(default = "default_footstep_distance_m")]
    pub footstep_distance_m: f32,
    #[serde(default = "default_footstep_teleport_threshold_m")]
    pub footstep_teleport_threshold_m: f32,
}

impl Default for SpatialAudioSettings {
    fn default() -> Self {
        Self {
            pixels_per_meter: default_pixels_per_meter(),
            reference_distance_m: default_reference_distance_m(),
            max_distance_m: default_max_distance_m(),
            rolloff_exponent: default_rolloff_exponent(),
            pan_distance_m: default_pan_distance_m(),
            footstep_distance_m: default_footstep_distance_m(),
            footstep_teleport_threshold_m: default_footstep_teleport_threshold_m(),
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

    pub fn pan_for_delta_x_px(&self, delta_x_px: f32) -> f32 {
        let ppm = self.pixels_per_meter.max(0.001);
        let delta_x_m = delta_x_px / ppm;
        let pan_dist = self.pan_distance_m.max(0.001);
        (delta_x_m / pan_dist).clamp(-1.0, 1.0)
    }
}

fn default_pixels_per_meter() -> f32 { 32.0 }
fn default_reference_distance_m() -> f32 { 2.0 }
fn default_max_distance_m() -> f32 { 64.0 }
fn default_rolloff_exponent() -> f32 { 1.0 }
fn default_pan_distance_m() -> f32 { 8.0 }
fn default_footstep_distance_m() -> f32 { 0.85 }
fn default_footstep_teleport_threshold_m() -> f32 { 6.0 }

pub fn plugin(app: &mut App) {
    app
        .add_plugins((AudioPlugin, SpatialAudioPlugin))
        .configure_sets(Update, AcAudioSystems)
        .add_systems(Update, (
            play_sprite_animation_sfx_on_frame_change,
            play_animation_seri_sfx_on_frame_change,
            sync_sprite_loop_sfx,
            sync_sprite_timed_sfx,
            play_step_sfx_from_moved_distance,
        ).in_set(AcAudioSystems))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (load_spatial_audio_settings, apply_spatial_audio_settings).chain()
        ).in_set(AcAudioSystems))
        .init_resource::<SpatialAudioSettings>()
        .init_resource::<DefaultSpatialRadius>()
    ;
}

pub fn load_spatial_audio_settings(mut settings: ResMut<SpatialAudioSettings>) {
    let path = Path::new("assets/ron/audio/audio.settings.ron");
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(loaded) = ron::from_str::<SpatialAudioSettings>(&contents) else {
        error!(target: SPRITE_ANIMATION_INIT, "Failed parsing '{}'", path.display());
        return;
    };
    *settings = loaded;
}
