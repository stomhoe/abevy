use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(serde::Deserialize, Default)]
pub struct SfxEveryNframesSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default = "default_animation_sfx_every_n_frame_changes")]
    pub n: f32,
}

#[derive(serde::Deserialize, Default)]
pub struct SfxTimeIntervalSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub condition: String,
    #[serde(default = "default_sfx_interval_secs")]
    pub secs: f32,
    #[serde(default)]
    pub shorten_with_anim_playing_speed: bool,
}

#[derive(serde::Deserialize, Default)]
pub struct SfxLoopSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub condition: String,
}

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct SpriteConfigSeri {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub fallback_img_path: String,
    #[serde(default = "default_fallback_scalar")]
    pub z: f32,
    #[serde(default = "default_fallback_scalar")]
    pub y_sort: f32,
    #[serde(default)]
    pub mapped_anims: HashMap<(String, String, String, String), String>,
    #[serde(default)]
    pub parent_cat: String,
    #[serde(default)]
    pub tags: HashSet<String>,
    #[serde(default)]
    pub shares_tag: Vec<bool>,
    #[serde(default)]
    pub children_sprites: Vec<String>,
    #[serde(default)]
    pub sfx_every_n_frames: SfxEveryNframesSeri,
    #[serde(default)]
    pub loop_sfx: SfxLoopSeri,
    #[serde(default)]
    pub interval_sfx: SfxTimeIntervalSeri,
    #[serde(default)]
    pub directionable: bool,
    #[serde(default)]
    pub movement_based: bool,
    #[serde(default)]
    pub grounding_based: bool,
    pub visibility: Option<u8>,
    #[serde(default)]
    pub offset4children: HashMap<String, (f32, f32, String)>,
    #[serde(default)]
    pub exclude_from_sys: bool,
    #[serde(default = "default_baseline_move_speed")]
    pub baseline_move_speed: f32,
    #[serde(default)]
    pub exclude_from_normal_size_modifier: bool,
    #[serde(default)]
    pub offset: (f32, f32),
    #[serde(default = "default_scale_2d")]
    pub scale: (f32, f32),
    #[serde(default = "default_scale_2d")]
    pub scale_up_down: (f32, f32),
    #[serde(default = "default_scale_2d")]
    pub scale_sideways: (f32, f32),
    pub flip_horiz_if_dir: Option<u8>,
    #[serde(default)]
    pub offset_up_down: (f32, f32),
    #[serde(default)]
    pub offset_down: (f32, f32),
    #[serde(default)]
    pub offset_up: (f32, f32),
    #[serde(default)]
    pub offset_sideways: (f32, f32),
    pub extra_y_offset_per_scale_inc: Option<f32>,
}

fn default_scale_2d() -> (f32, f32) { (1.0, 1.0) }
fn default_baseline_move_speed() -> f32 { 0.0 }
fn default_animation_sfx_every_n_frame_changes() -> f32 { 1.0 }
fn default_sfx_interval_secs() -> f32 { 0.35 }
fn default_fallback_scalar() -> f32 { f32::NAN }
