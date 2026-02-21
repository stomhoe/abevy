use crate::body::BodyPart;
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use bevy::prelude::*;
use game_common::game_common_components::EntityZero;

#[derive(Asset, serde::Deserialize, TypePath, Default, Debug, Clone)]
/// TODO hacer que el peso/hitpoints de cada bodypart se le pueda aplicar un multiplier por el body size del animal para reducir o aumentar su respectivo valor. asi no hay que crear tantas bodyparts similares que lo unico que cambia es el peso y hp y la blood capacity
pub struct BodyPartSeri {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub parent: String,
    #[serde(default)]
    pub slots: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub coverage_weight: u16,
    #[serde(default)]
    pub depth: String,
    #[serde(default)]
    pub vital: bool,
    #[serde(default)]
    pub bleed_rate: f32,
    #[serde(default)]
    pub forced_stats: HashMap<String, f32>,
    #[serde(default)]
    pub weighted_stats: HashMap<String, f32>,

    /// Tag used to enable synergistic modifiers across multiple matching parts.
    #[serde(default)]
    pub synergy_tags: HashSet<String>,
    /// Portion [0..1] of other matching-tag modifiers sumadded into ours.
    #[serde(default)]
    pub synergy_copy_mult: f32,
    /// Flat bonus applied when another matching-tag modifier is present.
    #[serde(default)]
    pub synergy_offset: f32,

    #[serde(default)]
    pub extra_modifiers_on_body_holder: HashMap<String, (String, String)>,

}

common::define_entity_map_systems!(
    BodyPart,
    With<EntityZero>,
    BodyPartSeri, "seri.being.body.part", "bodypart.ron",
);
