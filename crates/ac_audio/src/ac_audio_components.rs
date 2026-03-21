use bevy::prelude::*;
use bevy_kira_audio::AudioInstance;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AnimationFrameSfxState {
    pub last_frame: usize,
    pub frame_changes_acc: f32,
}

#[derive(Component, Clone, Debug, Default)]
pub struct AnimationSeriSfxConfig {
    pub sound_paths: Vec<String>,
    pub every_n_frame_changes: f32,
}

#[derive(Component, Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct AnimationSeriSfxState {
    pub last_frame: usize,
    pub frame_changes_acc: f32,
}

#[derive(Component, Clone, Debug, Default)]
pub struct SpriteLoopSfxState {
    pub instances: Vec<Handle<AudioInstance>>,
}

#[derive(Component, Clone, Debug, Default)]
pub struct SpriteTimedSfxState {
    pub elapsed_secs: f32,
}
