#[allow(unused_imports)]
use bevy::prelude::*;

use crate::body::body_tree_components::BodyTree;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BodyNodeSeri {
    pub part_id: String,
    #[serde(default)]
    pub label_override: String,
    pub children: Vec<BodyNodeSeri>,
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
    pub hp_capacity: f32,
    #[serde(default)]
    pub hp_regen_rate: f32,
    #[serde(default)]
    pub blood_capacity: f32,
    #[serde(default)]
    pub blood_pumping: f32,
    #[serde(default)]
    pub walk_speed: f32,
    #[serde(default)]
    pub swim_speed: f32,
    #[serde(default)]
    pub fly_speed: f32,
    #[serde(default)]
    pub manipulation: f32,
    #[serde(default)]
    pub vision: f32,
    #[serde(default)]
    pub pain_sensitivity: f32,
    #[serde(default)]
    pub caloric_burn_rate: f32,
    #[serde(default)]
    pub caloric_capacity: f32,
    pub root: BodyNodeSeri,
}

common::define_entity_map_systems!(
    BodyTree,
    BodyTreeSeri, "seri.being.body.tree", "bodytree.ron",
);
