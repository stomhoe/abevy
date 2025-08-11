use crate::camera_systems::*;
use bevy::prelude::*;
use game_common::game_common::GameplaySystems;




pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            // Add any common plugins here
        ))
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (
            enforce_single_camera_target, camera_follow_target, camera_zoom_system
        ).in_set(GameplaySystems))
        ;
}