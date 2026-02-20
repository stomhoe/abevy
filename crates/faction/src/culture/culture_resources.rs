use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::HashMap,
    prelude::*,
};
use common::common_components::{Prefix, StrId, Tag};
use serde::{Deserialize, Serialize};

use crate::faction_components::Culture;

common::define_entity_map_systems!(
    Culture,
    CultureSeri, "seri.faction.culture", "culture.ron",
);

#[derive(Component, Debug, Default, Clone)]
pub struct CultureBitWeightMap(pub HashMap<StrId, f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct CultureRacesOpinionStrIds(pub HashMap<StrId, f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct CultureRacesOpinion(pub EntityHashMap<f32>);

#[derive(Deserialize, Serialize, Asset, TypePath, Default, Debug)]
pub struct CultureSeri {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub bit_weightmap: HashMap<String, f32>,
    #[serde(default)]
    pub races_opinion: HashMap<String, f32>,
    #[serde(default)]
    pub technology_level: f32,
    #[serde(default)]
    pub militarism: f32,
    #[serde(default)]
    pub spirituality: f32,
    #[serde(default)]
    pub trade_focus: f32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct CultureTags(pub Vec<Tag>);

#[derive(Component, Debug, Clone)]
pub struct RacePrefix(pub Prefix);
impl Default for RacePrefix {
    fn default() -> Self {
        Self(Prefix::trunc("Race"))
    }
}
