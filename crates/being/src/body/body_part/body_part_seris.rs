use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use bevy::prelude::*;
use item_shared::item_seris::SlottedItemHolderSeri;
use modifier_shared::modifier_seris::ModifierSynergySeri;

#[derive(Asset, serde::Deserialize, TypePath, Default, Debug, Clone)]
/// TODO hacer que el peso/hitpoints de cada bodypart se le pueda aplicar un multiplier por el body size del animal para reducir o aumentar su respectivo valor. asi no hay que crear tantas bodyparts similares que lo unico que cambia es el peso y hp y la blood capacity
pub struct BodyPartSeri {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub parent: String,
    #[serde(default)]
    pub slots: SlottedItemHolderSeri,
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
    #[serde(default)]
    pub synergies: HashMap<String, ModifierSynergySeri>,
    #[serde(default)]
    pub extra_modifiers_on_body_holder: HashMap<String, (String, String)>,
}
