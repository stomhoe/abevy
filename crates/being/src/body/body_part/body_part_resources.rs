use crate::body::BodyPart;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)]
use bevy::prelude::*;
use game_common::game_common_components::EntityZero;

#[derive(Asset, serde::Deserialize, TypePath, Default, Debug, Clone)]
/// TODO hacer que el peso/hitpoints de cada bodypart se le pueda aplicar un multiplier por el body size del animal para reducir o aumentar su respectivo valor. asi no hay que crear tantas bodyparts similares que lo unico que cambia es el peso y hp y la blood capacity
pub struct BodyPartSeri {
    pub id: String,
    pub name: Option<String>,
    pub parent: Option<String>,
    pub slots: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub coverage_weight: u16,
    #[serde(default)]
    pub hp_capacity: f32,
    #[serde(default)]
    pub hp_capacity_weight: f32,
    #[serde(default)]
    pub hp_regen_rate: f32, //default 0 -> unset
    #[serde(default)]
    pub hp_regen_rate_weight: f32,
    pub depth: Option<String>,
    #[serde(default)]
    pub vital: bool,
    #[serde(default)]
    pub pain_sensitivity: f32,
    #[serde(default)]
    pub pain_sensitivity_weight: f32,
    pub kind: Option<String>,
    #[serde(default)]
    pub bleed_rate: f32,
    #[serde(default)]
    pub blood_capacity: f32,
    #[serde(default)]
    pub blood_capacity_weight: f32,

    #[serde(default)]
    pub blood_pumping: f32,
    #[serde(default)]
    pub blood_pumping_weight: f32,
    #[serde(default)]
    pub manipulation: f32,
    #[serde(default)]
    pub manipulation_weight: f32,
    #[serde(default)]
    pub walk_speed: f32,
    #[serde(default)]
    pub walk_speed_weight: f32,
    #[serde(default)]
    pub swim_speed: f32,
    #[serde(default)]
    pub swim_speed_weight: f32,
    #[serde(default)]
    pub fly_speed: f32,
    #[serde(default)]
    pub fly_speed_weight: f32,

    /// Tag used to enable synergistic modifiers across multiple matching parts.
    pub synergy_tag: Option<String>,
    /// Portion [0..1] of other matching-tag modifiers sumadded into ours.
    pub synergy_copy_mult: Option<f32>,
    /// Flat bonus applied when another matching-tag modifier is present.
    pub synergy_offset: Option<f32>,

    pub extra_modifiers_on_body_holder: Option<HashMap<String, (String, String)>>,

    #[serde(default)]
    pub vision: f32,
    #[serde(default)]
    pub vision_weight: f32,

    #[serde(default)]
    pub caloric_burn_rate: f32, //calories burned per second when this body part is active
    #[serde(default)]
    pub caloric_burn_rate_weight: f32,
    #[serde(default)]
    pub caloric_capacity: f32,  // calories that can be stored in this body part, if any
    #[serde(default)]
    pub caloric_capacity_weight: f32,

    #[serde(default)]
    pub mass_kg: f32,
    #[serde(default)]
    pub mass_weight: f32,
}

common::define_entity_map_systems!(
    BodyPart,
    With<EntityZero>,
    BodyPartSeri, "seri.being.body.part", "bodypart.ron",
);
