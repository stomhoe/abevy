#[allow(unused_imports)]
use bevy::prelude::*;

use crate::body::body_tree_components::BodyTree;
use crate::body::body_part::body_part_resources::BodyPartSeri;

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct BodyTreeNodeSeri {
    pub part_id: String,
    pub label_override: Option<String>,
    pub children: Vec<BodyTreeNodeSeri>,
}

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct BodyTreeSeri {
    pub id: String,
    pub name: String,
    pub tags: Option<Vec<String>>,
    pub root: BodyTreeNodeSeri,
}

common::define_entity_map_systems!(
    BodyTree,
    BodyTreeSeri, "ron/being/body/tree", "bodytree.ron",
);
