use bevy::prelude::*;

use crate::camera_components::CameraTarget;
use common::log_targets::LIGHTING_INIT;
use tilemap_shared::{DimensionEntityMap, DimensionRef};


pub fn resolve_daylight_dimension_for_camera_target(
    camera_dimension: &Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: &Res<DimensionEntityMap>,
) -> Option<Entity> {
    let mut camera_dimensions = camera_dimension.iter();
    let Some(camera_dimension) = camera_dimensions.next() else {
        return None;
    };
    if camera_dimensions.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: the camera target is duplicated");
        return None;
    }

    let Ok(dimension_ent) = dimension_map.0.get_cloned(camera_dimension.0) else {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: camera target dimension hash {:?} is missing from DimensionEntityMap", camera_dimension.0);
        return None;
    };

    Some(dimension_ent)
}
