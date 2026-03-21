#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use crate::race::race_seris::RaceSexEntrySeri;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BodypartNodeSeri {
    pub part_id: String,
    #[serde(default)]
    pub label_override: String,
    pub children: Vec<BodypartNodeSeri>,
}

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BodyTreeSeri {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub mass_kg: f32,
    #[serde(default)]
    pub distributed_totals: HashMap<String, f32>,
    #[serde(default)]
    pub sexes: HashMap<String, RaceSexEntrySeri>,
    #[serde(default)]
    pub caloric_burn_rate_multiplier: f32,
    pub root: BodypartNodeSeri,
}
