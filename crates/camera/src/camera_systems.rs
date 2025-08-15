
use bevy::{input::mouse::MouseWheel, prelude::*};
use dimension::dimension_components::{Dimension, DimensionRef};
use game_common::game_common_components::VisibilityGameState;
use sprite::sprite_components::SpriteHolderRef;

use crate::camera_components::*;

pub fn spawn_camera(mut commands: Commands, ) {
    commands.spawn((Camera2d::default(), Camera {hdr: true, ..default()}, ));
}

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

#[allow(unused_parens, )]
pub fn hide_nonvisualized_dimension(
    camera_curr_dimension: Single<(&DimensionRef), (With<CameraTarget>,)>,
    mut dimensions: Query<(Entity, &mut Visibility, ), (With<Dimension>)>,
) {
    for (dimension_ent, mut visibility, ) in dimensions.iter_mut() {
        if camera_curr_dimension.0 != dimension_ent {
            *visibility = Visibility::Hidden;
        }
        else {
            *visibility = Visibility::Visible;
        }
    }
}
