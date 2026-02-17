#[allow(unused_imports)]
use bevy::prelude::*;

use crate::body::body_tree_components::BodyTree;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BodyNodeSeri {
    pub part_id: String,
    pub label_override: Option<String>,
    pub children: Vec<BodyNodeSeri>,
}

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BodyTreeSeri {
    pub id: String,
    pub name: String,
    pub tags: Option<Vec<String>>,
    pub root: BodyNodeSeri,
}

common::define_entity_map_systems!(
    BodyTree,
    BodyTreeSeri, "seri.being.body.tree", "bodytree.ron",
);
