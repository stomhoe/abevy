#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::platform::collections::HashMap;

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
    pub distributed_totals: HashMap<String, f32>,
    pub root: BodyNodeSeri,
}

common::define_entity_map_systems!(
    BodyTree,
    BodyTreeSeri, "seri.being.body.tree", "bodytree.ron",
);
