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
    pub coverage_weight: Option<u16>,
    pub hp_capacity: Option<f32>,
    pub hp_regen_rate: Option<f32>, //default 1
    pub depth: Option<String>,
    pub vital: Option<bool>,
    pub pain_sensitivity: Option<f32>,
    pub kind: Option<String>,
    pub bleed_rate: Option<f32>,
    pub blood_capacity: Option<f32>,

    pub blood_pumping: Option<f32>,
    pub manipulation: Option<f32>,
    pub walk_speed: Option<f32>,
    pub swim_speed: Option<f32>,
    pub fly_speed: Option<f32>,

    /// Tag used to enable synergistic modifiers across multiple matching parts.
    pub synergy_tag: Option<String>,
    /// Portion [0..1] of other matching-tag modifiers sumadded into ours.
    pub synergy_copy_mult: Option<f32>,
    /// Flat bonus applied when another matching-tag modifier is present.
    pub synergy_offset: Option<f32>,

    pub extra_modifiers_on_body_holder: Option<HashMap<String, (String, String)>>,

    pub vision: Option<f32>,

    pub caloric_burn_rate: Option<f32>, //calories burned per second when this body part is active
    pub caloric_capacity: Option<f32>,  // calories that can be stored in this body part, if any

    pub mass: Option<f32>,
}

common::define_entity_map_systems!(
    BodyPart,
    With<EntityZero>,
    BodyPartSeri, "ron/being/body/part", "bodypart.ron",
);
