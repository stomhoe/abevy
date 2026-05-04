use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;
use bevy_firefly::prelude::*;

use crate::camera_components::CameraTarget;
use common::log_targets::LIGHTING_INIT;
use tilemap_shared::{DimensionDaylightSeri, DimensionEntityMap, DimensionRef};

#[derive(Clone, Debug)]
pub struct DaylightSettings {
    pub day_length_minutes: f32,
    pub time_of_day_minutes: f32,
    pub ambient_color_rgb: [f32; 3],
    pub ambient_brightness: f32,
    pub night_color_rgb: [f32; 3],
    pub dawn_dusk_color_rgb: [f32; 3],
    pub day_curve_exponent: f32,
    pub dawn_dusk_curve_exponent: f32,
    pub ambient_min_brightness_factor: f32,
    pub ambient_max_brightness_factor: f32,
}

impl Default for DaylightSettings {
    fn default() -> Self {
        DimensionDaylightSeri::default().into()
    }
}

impl From<DimensionDaylightSeri> for DaylightSettings {
    fn from(value: DimensionDaylightSeri) -> Self {
        let mut settings = Self {
            day_length_minutes: value.day_length_minutes,
            time_of_day_minutes: value.time_of_day_minutes,
            ambient_color_rgb: value.ambient_color_rgb,
            ambient_brightness: value.ambient_brightness,
            night_color_rgb: value.night_color_rgb,
            dawn_dusk_color_rgb: value.dawn_dusk_color_rgb,
            day_curve_exponent: value.day_curve_exponent,
            dawn_dusk_curve_exponent: value.dawn_dusk_curve_exponent,
            ambient_min_brightness_factor: value.ambient_min_brightness_factor,
            ambient_max_brightness_factor: value.ambient_max_brightness_factor,
        };
        settings.normalize();
        settings
    }
}

impl DaylightSettings {
    pub fn normalize(&mut self) {
        self.day_length_minutes = self.day_length_minutes.max(1.0);
        self.time_of_day_minutes = self.time_of_day_minutes.rem_euclid(self.day_length_minutes);
        self.ambient_brightness = self.ambient_brightness.max(0.0);
        self.day_curve_exponent = self.day_curve_exponent.max(0.01);
        self.dawn_dusk_curve_exponent = self.dawn_dusk_curve_exponent.max(0.01);
        self.ambient_min_brightness_factor = self.ambient_min_brightness_factor.clamp(0.0, 10.0);
        self.ambient_max_brightness_factor = self.ambient_max_brightness_factor.max(self.ambient_min_brightness_factor);

        for value in &mut self.ambient_color_rgb {
            *value = value.clamp(0.0, 1.0);
        }
        for value in &mut self.night_color_rgb {
            *value = value.clamp(0.0, 1.0);
        }
        for value in &mut self.dawn_dusk_color_rgb {
            *value = value.clamp(0.0, 1.0);
        }
    }

    pub fn day_progress(&self) -> f32 {
        self.time_of_day_minutes / self.day_length_minutes.max(1.0)
    }

    pub fn ambient_day_factor(&self) -> f32 {
        let day_progress = self.day_progress().rem_euclid(1.0);
        let sun_height = (day_progress * TAU - FRAC_PI_2).sin().clamp(-1.0, 1.0);
        ((sun_height + 1.0) * 0.5).powf(self.day_curve_exponent)
    }

    fn lerp_rgb(a: [f32; 3], b: [f32; 3], factor: f32) -> [f32; 3] {
        [
            a[0] + (b[0] - a[0]) * factor,
            a[1] + (b[1] - a[1]) * factor,
            a[2] + (b[2] - a[2]) * factor,
        ]
    }

    pub fn ambient_color(&self) -> Color {
        Color::srgb(self.ambient_color_rgb[0], self.ambient_color_rgb[1], self.ambient_color_rgb[2])
    }

    pub fn ambient_color_for_time(&self) -> Color {
        let factor = self.ambient_day_factor();
        let sun_height = (self.day_progress() * TAU - FRAC_PI_2).sin().clamp(-1.0, 1.0);
        let orange_factor = (1.0 - sun_height.abs()).clamp(0.0, 1.0).powf(self.dawn_dusk_curve_exponent);
        let night_color = self.night_color_rgb;
        let dawn_dusk_color = Self::lerp_rgb(night_color, self.dawn_dusk_color_rgb, orange_factor);
        let day_color = Self::lerp_rgb(night_color, self.ambient_color_rgb, factor);
        let final_color = Self::lerp_rgb(dawn_dusk_color, day_color, factor);

        Color::srgb(final_color[0], final_color[1], final_color[2])
    }

    pub fn ambient_brightness_for_time(&self) -> f32 {
        let factor = self.ambient_day_factor();
        let min_brightness = self.ambient_brightness * self.ambient_min_brightness_factor;
        let max_brightness = self.ambient_brightness * self.ambient_max_brightness_factor;
        min_brightness + (max_brightness - min_brightness) * factor
    }

    pub fn firefly_config(&self) -> FireflyConfig {
        FireflyConfig {
            ambient_color: self.ambient_color_for_time(),
            ambient_brightness: self.ambient_brightness_for_time(),
            ..default()
        }
    }
}

pub fn resolve_daylight_settings_for_dimension(
    camera_dimension: &DimensionRef,
    dimension_map: &Res<DimensionEntityMap>,
    daylight_query: &Query<&DimensionDaylightSeri>,
) -> Option<DaylightSettings> {
    let Ok(dimension_ent) = dimension_map.0.get_cloned(camera_dimension.0) else {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: camera target dimension hash {:?} is missing from DimensionEntityMap", camera_dimension.0);
        return None;
    };
    let Ok(daylight) = daylight_query.get(dimension_ent) else {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: dimension hash {:?} has no DimensionDaylightSeri component", camera_dimension.0);
        return None;
    };

    Some((*daylight).into())
}

pub fn resolve_daylight_settings_for_camera_target(
    camera_dimension: &Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: &Res<DimensionEntityMap>,
    daylight_query: &Query<&DimensionDaylightSeri>,
) -> Option<DaylightSettings> {
    let mut camera_dimensions = camera_dimension.iter();
    let Some(camera_dimension) = camera_dimensions.next() else {
        return None;
    };
    if camera_dimensions.next().is_some() {
        error_once!(target: LIGHTING_INIT, "Unable to resolve daylight settings: the camera target is duplicated");
        return None;
    }

    resolve_daylight_settings_for_dimension(camera_dimension, dimension_map, daylight_query)
}
