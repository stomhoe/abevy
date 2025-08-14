#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_types::HashIdToEntityMap;
use std::mem::take;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::{
    dimension_components::*,
//    dimension_constants::*,
//    dimension_events::*,
};

#[derive(Resource, Debug, Default )]
pub struct DimensionEntityMap(pub HashIdToEntityMap);

#[derive(AssetCollection, Resource)]
pub struct DimensionSerisHandles {
    #[asset(path = "ron/dimension", collection(typed))]
    pub handles: Vec<Handle<DimensionSeri>>,
    #[asset(path = "mods", collection(typed))]
    pub mod_handles: Vec<Handle<DimensionSeri>>,
}
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct DimensionSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}
