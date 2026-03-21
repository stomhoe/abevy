use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct StepDistanceSfxState {
    pub last_pos_px: Vec2,
    pub accumulated_distance_m: f32,
    pub last_sfx_path_hash: u64,
}
