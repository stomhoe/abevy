use std::{hash::{Hash, }};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
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

impl Default for DimensionDaylightSeri {
    fn default() -> Self {
        Self {
            day_length_minutes: 30.0,
            time_of_day_minutes: 12.0,
            ambient_color_rgb: [1.0, 1.0, 1.0],
            ambient_brightness: 0.,
            night_color_rgb: [0.14, 0.16, 0.26],
            dawn_dusk_color_rgb: [1.0, 0.66, 0.36],
            day_curve_exponent: 0.3,
            dawn_dusk_curve_exponent: 1.4,
            ambient_min_brightness_factor: 0.7,
            ambient_max_brightness_factor: 1.0,
        }
    }
}

impl DimensionDaylightSeri {
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
