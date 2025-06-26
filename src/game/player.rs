use bevy::prelude::*;

use crate::game::{player::{player_resources::KeyboardInputMappings, player_systems::*}, IngameSystems};

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
                camera_follow_target, 
                enforce_single_camera_target).in_set(IngameSystems),
                (update_move_input_dir, camera_zoom_system).in_set(PlayerInputSystems).in_set(IngameSystems)
            
            ))

            .init_resource::<KeyboardInputMappings>();
    }
}