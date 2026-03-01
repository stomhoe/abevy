#[allow(unused_imports)]
use bevy::prelude::*;

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
    pub root: BodyNodeSeri,
}
