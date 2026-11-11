use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("PetSet"), AssetScoped, SelectedForHotReload)]
pub struct PetSet;

#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct PetSetData {
    pub max_selected: u32,
    pub budget: u32,
    pub per_race_max: HashMap<HashId, u32>,
    pub pets: Vec<PetSetEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetSetEntry {
    pub id: HashId,
    pub name: String,
    pub race_id: HashId,
    pub cost: u32,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct PetSetSeri {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_selected: u32,
    pub budget: u32,
    pub per_race_max: HashMap<String, u32>,
    pub pets: Vec<PetSeri>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PetSeri {
    pub id: String,
    pub race_id: String,
    pub cost: u32,
}
