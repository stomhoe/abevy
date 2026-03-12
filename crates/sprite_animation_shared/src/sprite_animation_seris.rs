use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Deserialize, Asset, TypePath, Default, Clone)]
pub struct MultipleAnimationSeri(pub Vec<AnimationSeri>);

#[derive(Component, Debug, Deserialize, Serialize, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CardinalRotation {
    #[default]
    None,
    West,
    North,
    East,
}
impl CardinalRotation {
    pub fn angle(&self) -> Option<f32> {
        match self {
            CardinalRotation::None => None,
            CardinalRotation::West => Some(std::f32::consts::FRAC_PI_2),
            CardinalRotation::North => Some(std::f32::consts::PI),
            CardinalRotation::East => Some(-std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Component, Deserialize, Serialize, Asset, TypePath, Default, Clone)]
pub struct AnimationSeri {
    pub id: String,
    pub img_path: String,
    pub clips: Vec<ClipConfig>,

    pub anim_format_id: Option<String>,
    #[serde(default = "default_rows_cols")]
    pub rows_cols: (usize, usize),
    #[serde(default)]
    pub save_animation_progress: bool,
    pub alternating_start_frames: Option<(usize, usize)>,
    pub dir: Option<bool>,
    pub reps: Option<usize>,
    pub dur_frame: Option<u32>,
    pub dur_rep: Option<u32>,
    #[serde(default)]
    pub offset: [f32; 2],
    #[serde(default = "default_scale_2d")]
    pub scale: [f32; 2],
    pub y_sort: Option<f32>,
    pub z: f32,
    pub color: Option<[u8; 4]>,
    #[serde(default)]
    pub paused: bool,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub cardinal_rotation: CardinalRotation,
    #[serde(default = "default_animation_speed")]
    pub speed: f32,
    #[serde(default)]
    pub sound_effects: Vec<String>,
    #[serde(default = "default_sound_effects_every_n_frames")]
    pub sound_effects_every_n_frames: f32,
}

#[derive(Deserialize, Serialize, TypePath, Default, Clone)]
pub struct ClipConfig {
    pub target: usize,
    pub is_row: bool,
    pub partial: Option<(usize, usize)>,
    pub start_frame: Option<usize>,
    pub dir: Option<bool>,
    pub reps: Option<usize>,
    pub dur_frame: Option<u32>,
    pub dur_rep: Option<u32>,
}

fn default_scale_2d() -> [f32; 2] { [1.0, 1.0] }
fn default_rows_cols() -> (usize, usize) { (1, 1) }
fn default_animation_speed() -> f32 { 1.0 }
fn default_sound_effects_every_n_frames() -> f32 { 1.0 }
