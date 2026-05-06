use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_lit::prelude::*;
use bevy_kira_audio::SpatialAudioReceiver;
use common::log_targets::LIGHTING_INIT;
use ac_input::ac_input_actions::CameraZoomAction;
use tilemap_shared::GlobalTilePos;

use crate::camera_daylight::*;
use crate::camera_components::*;
use tilemap_shared::{DimensionDaylightRuntime, DimensionDaylightSeri, DirectionalLight2dOverride};

#[allow(unused_parens, )]
pub fn spawn_camera(mut commands: Commands, ) {
    debug!(target: LIGHTING_INIT, "Spawning 2D camera without lighting so UI can keep rendering outside ActiveGame");

    commands.spawn((
        Camera2d::default(),
        SpatialAudioReceiver,
        GlobalTilePos::default(),
        Transform::default(),
    ));
    let directional_light_entity = commands.spawn((
        DirectionalLight2d {
            tile_size: 32.0,
            ..default()
        },
        Name::new("Daylight"),
    )).id();
    commands.insert_resource(DaylightDirectionalLightEntity(directional_light_entity));
}

#[allow(unused_parens, )]
pub fn enable_lighting(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, Without<Lighting2dSettings>, )>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    daylight_query: Query<(&DimensionDaylightSeri, &DimensionDaylightRuntime)>,
    directional_light_override: Option<Res<DirectionalLight2dOverride>>,
    directional_light_entity: Res<DaylightDirectionalLightEntity>,
) {
    let mut cameras = camera.iter();
    let Some(camera) = cameras.next() else {
        return;
    };
    if cameras.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to enable 2D lighting: more than one camera exists without Lighting2dSettings");
        return;
    }

    let mut camera_dimensions = camera_dimension.iter();
    if camera_dimensions.next().is_none() {
        return;
    }
    if camera_dimensions.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to enable 2D lighting: the camera target is duplicated");
        return;
    }

    let Some(dimension_ent) = resolve_daylight_dimension_for_camera_target(&camera_dimension, &dimension_map) else {
        return;
    };

    let Ok((daylight, daylight_runtime)) = daylight_query.get(dimension_ent) else {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: dimension has no daylight runtime/config components");
        return;
    };

    let ambient_light = daylight.ambient_light(daylight_runtime);
    debug!(target: LIGHTING_INIT, "Enabling 2D lighting ambient_intensity={:.3} ambient_color={:?}", ambient_light.intensity, ambient_light.color);
    commands.entity(camera).insert((daylight.lighting_settings(), ambient_light));

    let light_entity = directional_light_entity.0;
    if daylight.disable_directional_light {
        commands.entity(light_entity).remove::<DirectionalLight2d>();
        return;
    }

    let directional_light_next = directional_light_override
        .as_ref()
        .map_or_else(|| daylight.directional_light(daylight_runtime), |override_settings| override_settings.apply_to(&daylight.directional_light(daylight_runtime)));
    commands.entity(light_entity).insert(directional_light_next);
}

#[allow(unused_parens, )]
pub fn sync_lighting(
    mut commands: Commands,
    mut camera_query: Query<(&mut Lighting2dSettings, &mut AmbientLight2d), (With<Camera>, )>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    daylight_query: Query<(&DimensionDaylightSeri, &DimensionDaylightRuntime)>,
    directional_light_override: Option<Res<DirectionalLight2dOverride>>,
    directional_light_entity: Res<DaylightDirectionalLightEntity>,
) {
    let mut camera_configs = camera_query.iter_mut();
    let Some((mut lighting_settings, mut ambient_light)) = camera_configs.next() else {
        return;
    };
    if camera_configs.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to sync 2D lighting: more than one camera has Lighting2dSettings");
        return;
    }

    let Some(dimension_ent) = resolve_daylight_dimension_for_camera_target(&camera_dimension, &dimension_map) else {
        return;
    };

    let Ok((daylight, daylight_runtime)) = daylight_query.get(dimension_ent) else {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: dimension has no daylight runtime/config components");
        return;
    };

    let ambient_light_next = daylight.ambient_light(daylight_runtime);
    *lighting_settings = daylight.lighting_settings();
    *ambient_light = ambient_light_next;

    let light_entity = directional_light_entity.0;
    if daylight.disable_directional_light {
        commands.entity(light_entity).remove::<DirectionalLight2d>();
        trace!(target: LIGHTING_INIT, "Directional light disabled for active dimension");
        return;
    }

    let directional_light_next = directional_light_override
        .as_ref()
        .map_or_else(|| daylight.directional_light(daylight_runtime), |override_settings| override_settings.apply_to(&daylight.directional_light(daylight_runtime)));
    commands.entity(light_entity).insert(directional_light_next);
    trace!(target: LIGHTING_INIT, "Updated ambient daylight intensity={:.3} color={:?}", ambient_light.intensity, ambient_light.color);
}

#[allow(unused_parens, )]
pub fn disable_lighting(mut commands: Commands, camera: Single<Entity, With<Camera>>, directional_light_entity: Res<DaylightDirectionalLightEntity>, ) {
    debug!(target: LIGHTING_INIT, "Disabling 2D lighting outside ActiveGame");

    commands.entity(*camera).remove::<Lighting2dSettings>();
    commands.entity(*camera).remove::<AmbientLight2d>();
    commands.entity(directional_light_entity.0).remove::<DirectionalLight2d>();
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

use tilemap_shared::{Dimension, DimensionEntityMap, DimensionRef};

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
