use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, AnimationProgress, Spritesheet};
use common::{common_components::*, };
use serde::{Deserialize, Serialize};
use tilemap_shared::CardinalDirection;




#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, )]
pub struct PlayingSpeed(pub f32);

impl Default for PlayingSpeed {
    fn default() -> Self {
        PlayingSpeed(1.0)
    }
}

#[derive(Component, Debug, Default, Clone)]
//va en cada sprite, no en las entities de las animations porque estas son compartidas por multiples sprites
pub struct AcAnimationProgresses(
    pub HashMap<Handle<Animation>, AnimationProgress>,
);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct MoveAnimActive(bool);
impl MoveAnimActive {
    pub fn set(&mut self, state: bool, being_ent: Entity, hash_set: &mut HashSet<BeingChangedMoveState>) {
        if self.0 != state {
            self.0 = state;
            hash_set.insert(BeingChangedMoveState(being_ent));
        }
    }
    pub fn get(&self) -> bool {
        self.0
    }
}

impl From<&str> for MoveAnimActive {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "move" | "walk"=> MoveAnimActive(true),
            _ => MoveAnimActive(false),
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
//not replicated
pub struct AnimExtraState(pub HashId);
impl AnimExtraState {
    pub fn new(id: impl Into<HashId>) -> Self {
        AnimExtraState(id.into())
    }
}

impl std::fmt::Display for AnimExtraState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Debug, Default, Clone, )]
pub struct AnimationHandle(pub Handle<Animation>,);

#[derive(Component, Debug, Clone, )]
pub struct AnimationSheet(pub Spritesheet,);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(HotReload, AssetScoped, Replicated, Prefix::trunc("Animation"),   )]
pub struct AcAnimation;



common::define_entity_map_systems!(
    AcAnimation
);
#[derive(Message, Clone, PartialEq, Eq, Hash)]
pub struct BeingChangedMoveState(pub Entity);

// TODO: hacer shaders aplicables? (para meditacion por ej)
// TODO: hacer que se puedan aplicar colorses sobre máscaras como en humanoid alien races del rimworld. hacer un mapa color-algo
#[derive(Component, Deserialize, Asset, TypePath, Default, Clone)]
pub struct MultipleAnimationSeri (pub Vec<AnimationSeri>);

#[derive(Component, Deserialize, Serialize, Asset, TypePath, Default, Clone)]
pub struct AnimationSeri {
    pub id: String,
    pub img_path: String,
    pub clips: Vec<ClipConfig>,

    //optional in case you want to reuse a predefined animation format
    pub anim_format_id: Option<String>,
    #[serde(default = "default_rows_cols")]
    pub rows_cols: (usize, usize), //rows, cols
    #[serde(default)]
    pub save_animation_progress: bool,
    pub alternating_start_frames: Option<(usize, usize)>,
    //None: forward, Some(true): backward, Some(false): ping-pong
    pub dir: Option<bool>,
    pub reps: Option<usize>, //0: infinite, n>0: n repetitions
    pub dur_frame: Option<u32>, //milliseconds
    pub dur_rep: Option<u32>, //milliseconds
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
    #[serde(default = "default_cardinal_direction")]
    pub cardinal_rotation: CardinalDirection,
    #[serde(default = "default_animation_speed")]
    pub speed: f32,
    #[serde(default)]
    pub sound_effects: Vec<String>,
    #[serde(default = "default_sound_effects_every_n_frames")]
    pub sound_effects_every_n_frames: f32,
}
#[derive(Deserialize, Serialize, TypePath, Default, Clone)]
/// Configuration for a sprite animation sequence.
/// Animation config for a row or column.
/// - `target`: Row/column index.
/// - `is_row`: True for row, false for column.
/// - `partial`: Optional [start, end] indices.
/// - `start_frame`: Optional starting frame index (default: 0).
/// - `dir`: None=forward, Some(true)=backward, Some(false)=ping-pong.
/// - `reps`: None=infinite, Some(n)=fixed repeats.
/// - `dur_frame`: Optional ms per frame.
/// - `dur_rep`: Optional ms per repetition.
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
fn default_cardinal_direction() -> CardinalDirection { CardinalDirection::South }
fn default_animation_speed() -> f32 { 1.0 }
fn default_sound_effects_every_n_frames() -> f32 { 1.0 }
