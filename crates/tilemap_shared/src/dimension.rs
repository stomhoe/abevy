use std::{hash::{Hash, }};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_lit::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
#[allow(unused_imports, )]
use bevy::platform::collections::{HashSet, HashMap};
use common::common_tag_components::TagSet;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, )]
#[require(Replicated, AssetScoped, Prefix::trunc("DIMENSION"),  )]
pub struct Dimension;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, )]
pub struct Gravity(pub f32);
impl Default for Gravity {
    fn default() -> Self {
        Self(9.81)
    }
}
impl Gravity {
    pub fn mass_to_newtons(&self, mass_kg: f32) -> f32 {
        mass_kg.max(0.0) * self.0.max(0.0)
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, )]
#[serde(default)]
pub struct DimensionDaylightSeri {
    pub day_length_minutes: f32,
    pub minute_offset: f32,
    pub paused_daylight: bool,
    pub ambient_color_rgb: [f32; 3],
    pub ambient_brightness: f32,
    pub night_color_rgb: [f32; 3],
    pub dawn_dusk_color_rgb: [f32; 3],
    pub day_curve_exponent: f32,
    pub dawn_dusk_curve_exponent: f32,
    pub ambient_min_brightness_factor: f32,
    pub ambient_max_brightness_factor: f32,
    pub disable_directional_light: bool,
    pub directional_light_color_rgb: [f32; 3],
    pub height_min: f32,
    pub height_max: f32,
    pub height_curve: f32,
}

impl Default for DimensionDaylightSeri {
    fn default() -> Self {
        Self {
            day_length_minutes: 30.0,
            minute_offset: 0.0,
            paused_daylight: false,
            ambient_color_rgb: [0.5764706, 0.77254903, 0.99215686],
            ambient_brightness: 0.18,
            night_color_rgb: [0.14, 0.16, 0.26],
            dawn_dusk_color_rgb: [1.0, 0.66, 0.36],
            day_curve_exponent: 0.3,
            dawn_dusk_curve_exponent: 1.5,
            ambient_min_brightness_factor: 0.7,
            ambient_max_brightness_factor: 1.1,
            disable_directional_light: false,
            directional_light_color_rgb: [1.0, 1.0, 1.0],
            height_min: 0.3,
            height_max: 2.4,
            height_curve: 3.0,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, )]
#[serde(default)]
pub struct DimensionDaylightRuntime {
    pub time_of_day_minutes: f32,
}

impl Default for DimensionDaylightRuntime {
    fn default() -> Self {
        Self {
            time_of_day_minutes: 12.0,
        }
    }
}

impl DimensionDaylightRuntime {
    pub fn normalize(&mut self, day_length_minutes: f32) {
        self.time_of_day_minutes = self.time_of_day_minutes.rem_euclid(day_length_minutes.max(1.0));
    }
}

#[derive(Resource, Clone, Debug)]
pub struct DirectionalLight2dOverride {
    pub color_enabled: bool,
    pub color_rgb: [f32; 3],
    pub height_enabled: bool,
    pub height: f32,
    pub direction_enabled: bool,
    pub direction_xy: [f32; 2],
    pub tile_size_enabled: bool,
    pub tile_size: f32,
}

impl Default for DirectionalLight2dOverride {
    fn default() -> Self {
        Self {
            color_rgb: [1.0, 1.0, 1.0],
            color_enabled: false,
            height_enabled: false,
            height: 1.0,
            direction_enabled: false,
            direction_xy: [0.0, 1.0],
            tile_size_enabled: false,
            tile_size: 32.0,
        }
    }
}

impl DirectionalLight2dOverride {
    pub fn apply_to(&self, directional_light: &DirectionalLight2d) -> DirectionalLight2d {
        DirectionalLight2d {
            color: if self.color_enabled {
                Color::srgb(self.color_rgb[0], self.color_rgb[1], self.color_rgb[2])
            } else {
                directional_light.color
            },
            height: if self.height_enabled { self.height } else { directional_light.height },
            direction: if self.direction_enabled {
                Vec2::new(self.direction_xy[0], self.direction_xy[1]).normalize_or_zero()
            } else {
                directional_light.direction
            },
            tile_size: if self.tile_size_enabled { self.tile_size } else { directional_light.tile_size },
        }
    }
}

impl DimensionDaylightSeri {
    pub fn normalize(&mut self) {
        self.day_length_minutes = self.day_length_minutes.max(1.0);
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
        for value in &mut self.directional_light_color_rgb {
            *value = value.clamp(0.0, 1.0);
        }
    }

    pub fn advance_runtime(&self, runtime: &mut DimensionDaylightRuntime, delta_minutes: f32) {
        if self.paused_daylight {
            return;
        }
        runtime.time_of_day_minutes = (runtime.time_of_day_minutes + delta_minutes)
            .rem_euclid(self.day_length_minutes.max(1.0));
    }

    pub fn effective_time_of_day_minutes(&self, runtime: &DimensionDaylightRuntime) -> f32 {
        runtime.time_of_day_minutes + self.minute_offset
    }

    pub fn day_progress(&self, runtime: &DimensionDaylightRuntime) -> f32 {
        self.effective_time_of_day_minutes(runtime) / self.day_length_minutes.max(1.0)
    }

    pub fn ambient_day_factor(&self, runtime: &DimensionDaylightRuntime) -> f32 {
        let day_progress = self.day_progress(runtime).rem_euclid(1.0);
        let sun_height = (day_progress * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2).sin().clamp(-1.0, 1.0);
        ((sun_height + 1.0) * 0.5).powf(self.day_curve_exponent)
    }

    fn lerp_rgb(a: [f32; 3], b: [f32; 3], factor: f32) -> [f32; 3] {
        [
            a[0] + (b[0] - a[0]) * factor,
            a[1] + (b[1] - a[1]) * factor,
            a[2] + (b[2] - a[2]) * factor,
        ]
    }

    pub fn ambient_color_for_time(&self, runtime: &DimensionDaylightRuntime) -> Color {
        let factor = self.ambient_day_factor(runtime);
        let sun_height = (self.day_progress(runtime) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2).sin().clamp(-1.0, 1.0);
        let orange_factor = (1.0 - sun_height.abs()).clamp(0.0, 1.0).powf(self.dawn_dusk_curve_exponent);
        let night_color = self.night_color_rgb;
        let dawn_dusk_color = Self::lerp_rgb(night_color, self.dawn_dusk_color_rgb, orange_factor);
        let day_color = Self::lerp_rgb(night_color, self.ambient_color_rgb, factor);
        let final_color = Self::lerp_rgb(dawn_dusk_color, day_color, factor);

        Color::srgb(final_color[0], final_color[1], final_color[2])
    }

    pub fn ambient_brightness_for_time(&self, runtime: &DimensionDaylightRuntime) -> f32 {
        let factor = self.ambient_day_factor(runtime);
        let min_brightness = self.ambient_brightness * self.ambient_min_brightness_factor;
        let max_brightness = self.ambient_brightness * self.ambient_max_brightness_factor;
        min_brightness + (max_brightness - min_brightness) * factor
    }

    pub fn lighting_settings(&self) -> Lighting2dSettings {
        Lighting2dSettings {
            blur: 4,
            edge_intensity: 8.0,
            ..default()
        }
    }

    pub fn ambient_light(&self, runtime: &DimensionDaylightRuntime) -> AmbientLight2d {
        AmbientLight2d {
            color: self.ambient_color_for_time(runtime),
            intensity: self.ambient_brightness_for_time(runtime),
        }
    }

    pub fn next_directional_light(&self, runtime: &DimensionDaylightRuntime) -> DirectionalLight2d {
        let day_progress = self.day_progress(runtime).rem_euclid(1.0);
        let dawn = 0.05;
        let dusk = 0.95;
        let day_cycle = (day_progress - dawn) / (dusk - dawn);
        let phase = (day_cycle.clamp(0.0, 1.0) * std::f32::consts::PI).clamp(0.0, std::f32::consts::PI);
        let sun_height = phase.sin().max(0.0);

        let x = phase.cos();
        let y = 0.2 + sun_height * 0.9;
        let peak_distance = ((phase - std::f32::consts::FRAC_PI_2).abs() / std::f32::consts::FRAC_PI_2).clamp(0.0, 1.0);
        let height_factor = (1.0 - peak_distance).powf(1.8 + (self.height_curve - 1.0) * 0.2);
        let height = self.height_min + (self.height_max - self.height_min) * height_factor;

        DirectionalLight2d {
            color: Color::srgb(
                self.directional_light_color_rgb[0],
                self.directional_light_color_rgb[1],
                self.directional_light_color_rgb[2],
            ),
            height,
            direction: Vec2::new(x, y).normalize_or_zero(),
            tile_size: 32.0,
        }
    }
}

common::define_entity_map_systems!(
    main_component: Dimension,
    with_filters: (),
    abbreviation: Dimension,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "dimension",
    despawn_trigger: Dimension,
    id_type: common::common_components::StrId,
    assets: [(DimensionSeri, "seri.dimension", "dimension.ron")],
);
impl Dimension{
    pub fn overworld() -> StrId{
        StrId::trunc("ow")
    }
}

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct DimensionSeri {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub root_oplist: String,
    #[serde(default = "default_dimension_gravity")]
    pub gravity: f32,
    #[serde(default)]
    pub daylight: DimensionDaylightSeri,
    /// this dimension's tags, used for whoever needs it
    #[serde(default)]
    pub tags: HashSet<String>,

    #[serde(default)]
    pub whitelisted_structure_gen_tags: Vec<String>,
    #[serde(default)]
    pub blacklisted_structure_gen_tags: Vec<String>,
}
fn default_dimension_gravity() -> f32 { 9.81 }

impl DimensionStrIdRef {

    pub fn overworld_fallback() -> Self {
        warn!("Using overworld fallback for DimensionStrIdRef");
        DimensionStrIdRef(Dimension::overworld())
    }
}



#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DimensionSystems;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq)]
pub struct DimensionRootOplist(pub HashId);



#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct MultipleDimensionStringRefs(Vec<String>);

impl MultipleDimensionStringRefs {
    pub fn new(strings: Vec<String>) -> Self {
        let filtered = strings.into_iter().filter(|s| !s.is_empty()).collect();
        MultipleDimensionStringRefs(filtered)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }
}

#[derive(Component, Debug, Default, Serialize, Deserialize, Clone)]
pub struct MultipleDimensionRefs(pub HashSet<HashId>);

#[derive(Debug, Message)]
pub struct ReassignDimensionToEntity (pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct WhitelistedStructureGenTags(pub TagSet);

common::impl_tag_wrapper_deref!(WhitelistedStructureGenTags, TagSet);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct BlacklistedStructureGenTags(pub TagSet);

common::impl_tag_wrapper_deref!(BlacklistedStructureGenTags, TagSet);
