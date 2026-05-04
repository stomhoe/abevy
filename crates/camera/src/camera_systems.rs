use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_firefly::prelude::*;
use bevy_kira_audio::SpatialAudioReceiver;
use common::log_targets::LIGHTING_INIT;
use ac_input::ac_input_actions::CameraZoomAction;
use tilemap_shared::GlobalTilePos;

use crate::camera_daylight::*;
use crate::camera_components::*;

#[allow(unused_parens, )]
pub fn spawn_camera(mut commands: Commands, ) {
    debug!(target: LIGHTING_INIT, "Spawning 2D camera without lighting so UI can keep rendering outside ActiveGame");

    commands.spawn((Camera2d::default(), SpatialAudioReceiver, GlobalTilePos::default(), Transform::default()));
}

#[allow(unused_parens, )]
pub fn enable_firefly_lighting(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, Without<FireflyConfig>)>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    daylight_query: Query<&DimensionDaylightSeri>,
) {
    let mut cameras = camera.iter();
    let Some(camera) = cameras.next() else {
        return;
    };
    if cameras.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to enable firefly lighting: more than one camera exists without FireflyConfig");
        return;
    }

    let mut camera_dimensions = camera_dimension.iter();
    let Some(camera_dimension) = camera_dimensions.next() else {
        return;
    };
    if camera_dimensions.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to enable firefly lighting: the camera target is duplicated");
        return;
    }

    let Some(daylight) = resolve_daylight_settings_for_dimension(camera_dimension, &dimension_map, &daylight_query) else {
        return;
    };

    debug!(target: LIGHTING_INIT, "Enabling ambient daylight brightness={:.3} color={:?}", daylight.ambient_brightness_for_time(), daylight.ambient_color_for_time());
    commands.entity(camera).insert(daylight.firefly_config());
}

#[allow(unused_parens, )]
pub fn sync_firefly_lighting(
    mut camera_query: Query<&mut FireflyConfig, (With<Camera>)>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    daylight_query: Query<&DimensionDaylightSeri>,
) {
    let mut camera_configs = camera_query.iter_mut();
    let Some(mut firefly_config) = camera_configs.next() else {
        return;
    };
    if camera_configs.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to sync firefly lighting: more than one camera has a FireflyConfig");
        return;
    }

    let Some(daylight) = resolve_daylight_settings_for_camera_target(&camera_dimension, &dimension_map, &daylight_query) else {
        return;
    };

    *firefly_config = daylight.firefly_config();
    trace!(target: LIGHTING_INIT, "Updated ambient daylight brightness={:.3} color={:?}", firefly_config.ambient_brightness, firefly_config.ambient_color);
}

#[allow(unused_parens, )]
pub fn disable_firefly_lighting(mut commands: Commands, camera: Single<Entity, With<Camera>>, ) {
    debug!(target: LIGHTING_INIT, "Disabling Firefly lighting outside ActiveGame");

    commands.entity(*camera).remove::<FireflyConfig>();
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

use tilemap_shared::{Dimension, DimensionDaylightSeri, DimensionEntityMap, DimensionRef};

#[allow(unused_parens, )]
pub fn hide_nonvisualized_dimension(
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
            *visibility = Visibility::Visible;
        }
    }
}
