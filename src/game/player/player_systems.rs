

use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::game::{being::{being_components::{Being, ControlledBy, ControlledBySelf, PlayerDirectControllable}, movement::movement_components::InputMoveVector},  player::{player_components::*, player_resources::KeyboardInputMappings}};




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
    camera_query.translation.x = target.translation.x; camera_query.translation.y = target.translation.y;
    camera_query.translation.z = 0.0;
}

#[allow(unused_parens)]
pub fn on_control_change(mut commands: Commands, 
    self_player: Single<Entity, With<OfSelf>>,
    query: Query<(Entity, &ControlledBy),(Changed<ControlledBy>)>,) {
    for (ent, controlled_by) in query.iter() {
        if controlled_by.0 == *self_player{
            commands.entity(ent).insert(ControlledBySelf);
        }
        else {
            commands.entity(ent).remove::<ControlledBySelf>();
        }
    }
}
pub fn react_on_control_removal(mut commands: Commands, mut removed: RemovedComponents<ControlledBy>) {
    for entity in removed.read() {
        commands.entity(entity).remove::<ControlledBySelf>();
    }
}


#[allow(unused_parens, dead_code)]
pub fn update_move_input_dir(
    keys: Res<ButtonInput<KeyCode>>,
    input_mappings: Res<KeyboardInputMappings>,
    mut move_input_dir: Query<(&mut InputMoveVector), (With<ControlledBySelf>)>,
) {
    let mut input_dir = Vec3::ZERO;

    if keys.pressed(input_mappings.move_up) {input_dir.y += 1.0;}
    if keys.pressed(input_mappings.move_down) {input_dir.y -= 1.0;}
    if keys.pressed(input_mappings.move_left) {input_dir.x -= 1.0;}
    if keys.pressed(input_mappings.move_right) {input_dir.x += 1.0;}
    if keys.pressed(input_mappings.jump_or_fly) {input_dir.z += 1.0;}
    if keys.pressed(input_mappings.duck) {input_dir.z -= 1.0;}
    
    if input_dir != Vec3::ZERO {input_dir = input_dir.normalize();}

    for mut move_input_dir in move_input_dir.iter_mut() {
        move_input_dir.0 = input_dir;
    }
}

pub fn camera_zoom_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let zoom_speed = 0.1; let min_zoom = 0.0001; let max_zoom = 100.0; 

    let mut zoom_delta = 0.0;
    for event in mouse_wheel_events.read() {
        zoom_delta += event.y;
    }

    if zoom_delta.abs() > f32::EPSILON {
        for mut transform in camera_query.iter_mut() {
            let new_scale = (transform.scale.x - zoom_delta * zoom_speed)
                .clamp(min_zoom, max_zoom);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}
