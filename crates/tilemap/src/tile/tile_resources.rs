use bevy::{ecs::entity::EntityHashSet, math::f32, platform::collections::{HashMap, HashSet}, tasks::Task};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::{common_components::Tag, common_types::HashIdToEntityMap};
use dimension_shared::DimensionRef;
use serde::{Deserialize, Serialize};
use tilemap_shared::GlobalTilePos;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event, Reflect)]
#[reflect(Resource, Default)]
pub struct TileEzerosMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Message, Reflect)]
#[reflect(Resource, Default)]
//NO SE USA
pub struct TileInstancesEntityMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct TileCategories (pub HashMap<Tag, EntityHashSet>);

#[derive(Resource, Debug, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct TilesAtGpos (pub HashMap<(DimensionRef, GlobalTilePos), Vec<Entity>>);

#[derive(Debug, Clone)]
pub struct TileGposAddition {
    pub dimension_ref: DimensionRef,
    pub gpos: GlobalTilePos,
    pub entity: Entity,
    pub is_primary: bool,
}

#[derive(Debug, Default)]
pub struct TileGposTaskResult {
    pub additions: Vec<TileGposAddition>,
}

#[derive(Resource, Debug, Default)]
pub struct TileAsyncTasks {
    pub gpos_tasks: Vec<Task<TileGposTaskResult>>,
    pub despawn_tasks: Vec<Task<Vec<Entity>>>,
}


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileSerisHandles {
    #[asset(path = "ron/tilemap/tiling/tile", collection(typed))] 
    pub handles: Vec<Handle<TileSerialization>>,
}
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct TileImagePaths(
    pub Vec<(String, String)>, //key, path
);

impl TileImagePaths {
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, (String, String)> {
        self.0.iter_mut()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, (String, String)> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl<'a> IntoIterator for &'a TileImagePaths {
    type Item = &'a (String, String);
    type IntoIter = std::slice::Iter<'a, (String, String)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a> IntoIterator for &'a mut TileImagePaths {
    type Item = &'a mut (String, String);
    type IntoIter = std::slice::IterMut<'a, (String, String)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
impl IntoIterator for TileImagePaths {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<(String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}


#[derive(Deserialize, Asset, Reflect, Default, Component)]
pub struct TileSerialization {
    pub id: String,
    pub name: String,
    pub z: f32,
    pub img_paths: Vec<(String, String)>,
    pub tags: Option<Vec<String>>,
    pub y_sort: Option<f32>,
    pub cats: Option<HashSet<String>>,
    /// persisted only when state gets altered from starting state
    pub persisted: Option<bool>,
    pub shader: Option<String>,
    pub sprite: Option<bool>,
    pub color: Option<[u8; 4]>,
    pub color_map: Option<String>,
    pub spawns: Option<Vec<String>>,
    pub spawns_children: Option<Vec<String>>,
    pub randflipx: Option<bool>,
    pub min_distances: Option<HashMap<String, u64>>,
    pub portal: Option<PortalSeri>,
    pub offset: Option<(f32, f32)>,

}
#[derive(Component, Deserialize, Reflect, Default)]
pub struct PortalSeri{
    pub dest_dimension: String,
    pub oe_tile: String,
    pub oe_op_tags: Vec<String>,
    pub op_i: Option<i16>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub one_way: Option<bool>,
    /// NASE
    pub dungeon: String,
}

#[derive(Deserialize, Asset, Reflect, )]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}