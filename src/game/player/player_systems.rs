

use bevy::prelude::*;

use crate::game::{beings::beings_components::{Being, ControlledBySelf, PlayerDirectControllable, InputMoveDirection}, player::{player_components::*, player_resources::KeyboardInputMappings}};

// fn setup(
//     mut commsands: Commands, 
//     asset_server: Res<AssetServer>, 
//     window_query: Query<&Window, (With<Being>)>,
// ) {

// }





pub fn enforce_single_camera_target(
    mut commands: Commands,
    new_camera: Single<Entity, Added<CameraTarget>>,
    existing_cameras: Query<Entity, With<CameraTarget>>,
) {
    for existing in existing_cameras.iter() {
        if existing != *new_camera {
            commands.entity(existing).remove::<CameraTarget>();
        }
    }
}

pub fn camera_follow_target(
    target: Single<&Transform, With<CameraTarget>>,
    mut camera_query: Single<&mut Transform, (With<Camera>, Without<CameraTarget>)>,
) {
    camera_query.translation.x = target.translation.x;
    camera_query.translation.y = target.translation.y;
}


pub fn update_move_input_dir(
    keys: Res<ButtonInput<KeyCode>>,
    input_mappings: Res<KeyboardInputMappings>,
    mut move_input_dir: Query<&mut InputMoveDirection, (With<ControlledBySelf>, With<Being>)>,
) {
    let mut input_dir = Vec3::ZERO;

    if keys.pressed(input_mappings.move_up) {input_dir.y += 1.0;}
    if keys.pressed(input_mappings.move_down) {input_dir.y -= 1.0;}
    if keys.pressed(input_mappings.move_left) {input_dir.x -= 1.0;}
    if keys.pressed(input_mappings.move_right) {input_dir.x += 1.0;}
    if keys.pressed(input_mappings.jump_or_fly) {input_dir.z += 1.0;}
    if keys.pressed(input_mappings.duck) {input_dir.z -= 1.0;}
    
    
    if input_dir != Vec3::ZERO {
        input_dir = input_dir.normalize();
    }

    for mut move_input_dir in move_input_dir.iter_mut() {
        move_input_dir.0 = input_dir;
    }
}

