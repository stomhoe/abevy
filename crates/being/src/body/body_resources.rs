#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_asset_loader::prelude::*;

use crate::body::body_components::BodyTree;
use crate::body::body_part::body_part_resources::BodyPartSeri;

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct BodyTreeSerisHandles {
    #[asset(path = "ron/being/body/tree", collection(typed))]
    pub handles: Vec<Handle<BodyTreeSeri>>,
}

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
    BodyTreeEntityMap,
    common::common_components::StrId,
    BodyTree
);
