#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{ecs::entity::MapEntities, platform::collections::HashSet, prelude::*};
use common::common_types::HashIdToEntityMap;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;



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
    /// this dimension's tags, used for whoever needs it
    pub tags: Option<HashSet<String>>,

    pub whitelisted_structure_gen_tags: Option<Vec<String>>,
    pub blacklisted_structure_gen_tags: Option<Vec<String>>,
}
