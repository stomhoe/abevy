use crate::camera_systems::*;
use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::on_message;
use bevy::input::mouse::MouseWheel;
use game_common::game_common::GameplaySystems;



pub fn plugin(app: &mut App) {
    app
    .add_plugins((

    ))
    .add_systems(Startup, spawn_camera)
    .add_systems(Update, (
        delete_prev_camera_target, camera_follow_target, camera_zoom_system.run_if(on_message::<MouseWheel>), hide_nonvisualized_dimension,
    ).in_set(GameplaySystems))
    ;
}
