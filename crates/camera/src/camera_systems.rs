use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_lit::prelude::*;
use bevy_kira_audio::SpatialAudioReceiver;
use common::log_targets::LIGHTING_INIT;
use ac_input::ac_input_actions::CameraZoomAction;
use ::tilemap_shared::*;

use crate::camera_components::*;
use crate::lighting_systems::Sun;

#[allow(unused_parens, )]
pub fn spawn_camera(mut commands: Commands, ) {
    debug!(target: LIGHTING_INIT, "Spawning 2D camera without lighting so UI can keep rendering outside ActiveGame");

    commands.spawn((
        Camera2d::default(),
        SpatialAudioReceiver,
        GlobalTilePos::default(),
        Transform::default(),
    ));
    commands.spawn((
        DirectionalLight2d {
            tile_size: 32.0,
            ..default()
        },
        Name::new("Daylight"),
        Sun,
    ));
}



pub fn camera_follow_target(
    target: Query<&GlobalTransform, (With<CameraTarget>, Changed<GlobalTransform>)>,
    mut camera_query: Query<(&mut Transform, &mut GlobalTilePos), (With<Camera>, Without<CameraTarget>)>,
) {
    if target.is_empty() {
        return;
    }
    let target = target.iter().next().unwrap();
    if camera_query.is_empty() {
        error!("Camera is missing");
        return;
    }
    let Ok((mut camera_tansform, mut camera_global_tile_pos)) = camera_query.single_mut() else {
        error!("There's more than one Camera entity");
        return;
    };
    camera_tansform.translation.x = target.translation().x; camera_tansform.translation.y = target.translation().y;
    let calculated_gpos = GlobalTilePos::from(target.translation().xy());
    if calculated_gpos != *camera_global_tile_pos {
        *camera_global_tile_pos = calculated_gpos;
    }
    camera_tansform.translation.z = 0.0;
}

pub fn camera_zoom_system(
    zoom: Single<&Action<CameraZoomAction>>,
    mut camera_query: Query<&mut Projection, With<Camera>>,
) {
    let zoom_speed = 0.1; let min_zoom = 0.0001; let max_zoom = 100.0;

    let zoom_delta = ***zoom;

    if zoom_delta.abs() > f32::EPSILON {
        for mut projection in camera_query.iter_mut() {
            let Projection::Orthographic(orthographic) = &mut *projection else {
                continue;
            };

            orthographic.scale = (orthographic.scale - zoom_delta * zoom_speed)
                .clamp(min_zoom, max_zoom);
        }
    }
}


#[allow(unused_parens, )]
pub fn hide_noncurrent_dimension(
    camera_curr_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    mut dimensions: Query<(Entity, &mut Visibility, ), (With<Dimension>)>,
) {
    if camera_curr_dimension.is_empty() {
        return;
    }
    let Ok(camera_curr_dimension) = camera_curr_dimension.single() else {
        error!("Failed to get camera current dimension, multiple camera targets");
        return;
    };
    let Ok(camera_dimension_ent) = dimension_map.0.get_cloned(camera_curr_dimension.0) else {
        return;
    };
    for (dimension_ent, mut visibility, ) in dimensions.iter_mut() {
        if camera_dimension_ent != dimension_ent {
            *visibility = Visibility::Hidden;
        }
        else {
            *visibility = Visibility::Inherited;
        }
    }
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