use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{common::common_components::DisplayName, game::{player::{player_components::*, player_resources::KeyboardInputMappings, player_systems::*}, IngameSystems}};

// Module player
pub mod player_components;
mod player_systems;
mod player_resources;
//mod player_events;
//mod player_styles;


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerInputSystems;

pub struct PlayerPlugin;
#[allow(unused_parens)]
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {

        app
            .add_systems(Update, ((
                on_control_change,
                camera_follow_target, 
                react_on_control_removal,
                enforce_single_camera_target,
                (update_move_input_dir, camera_zoom_system).in_set(PlayerInputSystems)
            ).in_set(IngameSystems),
            ))
            .replicate_bundle::<(Player,DisplayName)>()
            .replicate::<Player>()
            .replicate::<HostPlayer>()

            .init_resource::<KeyboardInputMappings>();
    }
}