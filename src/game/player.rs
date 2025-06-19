use bevy::prelude::*;

use crate::game::player::player_systems::*;

// Module player
pub mod player_components;
mod player_systems;
mod player_resources;
//mod player_events;
//mod player_styles;

pub struct PlayerPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerInputSystemSet;

#[allow(unused_parens)]
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {

        app
            .add_systems(Update, (
                (camera_follow_target, enforce_single_camera_target),
                (update_move_input_dir).in_set(PlayerInputSystemSet)))
        ;
    }
}