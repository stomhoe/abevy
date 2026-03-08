use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_kira_audio::SpatialAudioReceiver;
use ac_input::ac_input_actions::CameraZoomAction;

use crate::camera_components::*;

pub fn spawn_camera(mut commands: Commands, ) {
    commands.spawn((Camera2d::default(), SpatialAudioReceiver));
}

pub fn delete_prev_camera_target(
    mut commands: Commands,
    new_camera: Single<Entity, Added<CameraTarget>>,
    existing_cameras: Query<Entity, With<CameraTarget>>,
) {
    for existing in existing_cameras.iter() {
        if existing != *new_camera {
            commands.entity(existing).try_remove::<CameraTarget>();
        }
    }
}

pub fn camera_follow_target(
    target: Query<&Transform, With<CameraTarget>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<CameraTarget>)>,
) {
    if target.is_empty() {
        return;
    }
    let target = target.iter().next().unwrap();
    if camera_query.is_empty() {
        error!("Camera is missing");
        return;
    }
    let Ok(mut camera_query) = camera_query.single_mut() else {
        error!("Failed to get camera query");
        return;
    };
    camera_query.translation.x = target.translation.x; camera_query.translation.y = target.translation.y;
    camera_query.translation.z = 0.0;
}

pub fn camera_zoom_system(
    zoom: Single<&Action<CameraZoomAction>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let zoom_speed = 0.1; let min_zoom = 0.0001; let max_zoom = 100.0;

    let zoom_delta = ***zoom;

    if zoom_delta.abs() > f32::EPSILON {
        for mut transform in camera_query.iter_mut() {
            let new_scale = (transform.scale.x - zoom_delta * zoom_speed)
                .clamp(min_zoom, max_zoom);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

use tilemap_shared::{Dimension, DimensionRef };

#[allow(unused_parens, )]
pub fn hide_nonvisualized_dimension(
    camera_curr_dimension: Query<&DimensionRef, With<CameraTarget>>,
    mut dimensions: Query<(Entity, &mut Visibility, ), (With<Dimension>)>,
) {
    if camera_curr_dimension.is_empty() {
        return;
    }
    let Ok(camera_curr_dimension) = camera_curr_dimension.single() else {
        error!("Failed to get camera current dimension, multiple camera targets");
        return;
    };
    for (dimension_ent, mut visibility, ) in dimensions.iter_mut() {
        if camera_curr_dimension.0 != dimension_ent {
            *visibility = Visibility::Hidden;
        }
        else {
            *visibility = Visibility::Visible;
        }
    }
}
