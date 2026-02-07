use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, AnimationProgress, Spritesheet};
use common::{common_components::{AssetScoped, Prefix, SparedFromHotReloading, StrId}, };
use serde::{Deserialize, Serialize};




#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, )]
pub struct PlayingSpeed(pub f32);
impl PlayingSpeed {
    pub fn new(speed: f32) -> Self {
        Self(speed)
    }
}
impl Default for PlayingSpeed {
    fn default() -> Self {
        PlayingSpeed(1.0)
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
//va en cada sprite, no en las entities de las animations porque estas son compartidas por multiples sprites
pub struct AcAnimationProgresses(
    pub HashMap<Handle<Animation>, AnimationProgress>,
);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, Hash, PartialEq, Eq, Default)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, PartialEq, Eq, Hash)]
//NO VA REPLICATED, SE HACE LOCALMENTE EN CADA PC SEGÚN LOS INPUTS RECIBIDOS DE OTROS PLAYERS
pub struct AnimationState(pub StrId);
impl AnimationState {

    pub fn new<S: AsRef<str>>(state: S) -> Self {
        Self(StrId::trunc(state.as_ref()))
    }
}
impl std::fmt::Display for AnimationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct AnimationHandle(pub Handle<Animation>,);

#[derive(Component, Debug, Clone, )]
pub struct AnimationSheet(pub Spritesheet,);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(SparedFromHotReloading, AssetScoped, Replicated, Prefix::trunc("Animation"),   )]
pub struct AcAnimation;



common::define_entity_map_systems!(
    AcAnimation,
    AnimationSeri, "ron/sprite/animation", "anim.ron"
);


#[derive(Message, Clone, PartialEq, Eq, Hash)]
pub struct BeingChangedMoveState(pub Entity);




// TODO: hacer shaders aplicables? (para meditacion por ej)
// TODO: hacer que se puedan aplicar colorses sobre máscaras como en humanoid alien races del rimworld. hacer un mapa color-algo

#[derive(Component, Deserialize, Serialize, Asset, Reflect, Default, )]
pub struct AnimationSeri {
    pub id: String,
    pub img_path: String,
    pub clips: Vec<ClipConfig>,

    //optional in case you want to reuse a predefined animation format
    pub anim_format_id: Option<String>,
    pub rows_cols: Option<(usize, usize)>, //rows, cols
    pub save_animation_progress: Option<bool>, //Some(true): save progress, Some(false)/None: don't save
    //None: forward, Some(true): backward, Some(false): ping-pong
    pub dir: Option<bool>,
    pub reps: Option<usize>, //0: infinite, n>0: n repetitions
    pub dur_frame: Option<u32>, //milliseconds
    pub dur_rep: Option<u32>, //milliseconds
    pub offset: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub y_sort: Option<f32>,
    pub z: f32,
    pub color: Option<[u8; 4]>,
}


#[derive(Deserialize, Serialize, Reflect, Default, Clone)]
/// Configuration for a sprite animation sequence.
/// Animation config for a row or column.
/// - `target`: Row/column index.
/// - `is_row`: True for row, false for column.
/// - `partial`: Optional [start, end] indices.
/// - `start_frame`: Optional starting frame index (default: 0).
/// - `alternating_start_frames`: Optional (frame1, frame2) for alternating between two start frames each time animation starts.
/// - `dir`: None=forward, Some(true)=backward, Some(false)=ping-pong.
/// - `reps`: None=infinite, Some(n)=fixed repeats.
/// - `dur_frame`: Optional ms per frame.
/// - `dur_rep`: Optional ms per repetition.
pub struct ClipConfig {
    pub target: usize,
    pub is_row: bool,
    pub partial: Option<(usize, usize)>,
    pub start_frame: Option<usize>,
    pub alternating_start_frames: Option<(usize, usize)>,
    pub dir: Option<bool>,
    pub reps: Option<usize>,
    pub dur_frame: Option<u32>,
    pub dur_rep: Option<u32>,
}
