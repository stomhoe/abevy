use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::sex::sex_components::Sex;

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SexSerisHandles {
    #[asset(path = "ron/being/sex", collection(typed))]
    pub handles: Vec<Handle<SexSerialization>>,
}

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct SexSerialization {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}


common::define_entity_map_systems!(
    common::common_components::StrId,
    Sex
);