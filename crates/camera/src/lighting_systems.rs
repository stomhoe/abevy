
use bevy::prelude::*;
use bevy_lit::prelude::*;
use common::log_targets::LIGHTING_INIT;

use crate::camera_daylight::*;
use crate::camera_components::*;
use ::tilemap_shared::*;

#[derive(Component, Debug, Default, Copy, Clone, )]
pub struct Sun;

#[allow(unused_parens, )]
pub fn enable_lighting(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, Without<Lighting2dSettings>, )>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    daylight_query: Query<(&DimensionDaylightSeri, &DimensionDaylightRuntime)>,
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
}

#[allow(unused_parens, )]
pub fn sync_dir_light_angle(
    mut commands: Commands,
    mut camera_query: Query<(&mut Lighting2dSettings, &mut AmbientLight2d), (With<Camera>, )>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    daylight_query: Query<(&DimensionDaylightSeri, &DimensionDaylightRuntime)>,
    sun_query: Query<Entity, With<Sun>>,
    mut directional_light_query: Query<&mut DirectionalLight2d>,
    directional_light_override: Option<Res<DirectionalLight2dOverride>>,
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

    let Ok(directional_light_entity) = sun_query.single() else {
        error_once!(target: LIGHTING_INIT, "Unable to find directional light entity with Sun component");
        return;
    };

    let light_entity = directional_light_entity;
    if daylight.disable_directional_light {
        commands.entity(light_entity).try_remove::<DirectionalLight2d>();
        trace!(target: LIGHTING_INIT, "Directional light disabled for active dimension");
        return;
    }

    let directional_light_next = directional_light_override
        .as_ref()
        .map_or_else(|| daylight.next_directional_light(daylight_runtime), |override_settings| override_settings.apply_to(&daylight.next_directional_light(daylight_runtime)));
    if let Ok(mut directional_light) = directional_light_query.get_mut(light_entity) {
        *directional_light = directional_light_next;
    } else {
        commands.entity(light_entity).try_insert(directional_light_next);
    }
    trace!(target: LIGHTING_INIT, "Updated ambient daylight intensity={:.3} color={:?}", ambient_light.intensity, ambient_light.color);
}

#[allow(unused_parens, )]
pub fn disable_lighting(mut commands: Commands, camera: Single<Entity, With<Camera>>, sun_query: Query<Entity, With<Sun>>, ) {
    let Ok(directional_light_entity) = sun_query.single() else {
        error_once!(target: LIGHTING_INIT, "Unable to find directional light entity with Sun component");
        return;
    };

    commands.entity(*camera).try_remove::<(AmbientLight2d)>();
    commands.entity(directional_light_entity).try_remove::<DirectionalLight2d>();
}

