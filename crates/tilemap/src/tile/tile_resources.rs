use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, math::f32, platform::collections::{HashMap, HashSet}, };
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use common::{common_components::{Tag}, };
use game_common::game_common_components::EntityZero;
use serde::{Deserialize, Serialize};

pub use crate::tilemap_resources::{MassCollectedTiles, ImportantRegisteredPositions, CloneSpawnParamSet};
use crate::tile::tile_components::Tile;

common::define_entity_map_systems!(
    Tile,
    (With<EntityZero>, common::AnyDisabling),
    (Tile, EntityZero),
    TileSeri, "seri.tilemap.tile", "tile.ron",
);


#[derive(Resource, Debug, Default)]
pub struct TileEntsWithinTag (pub HashMap<Tag, EntityHashSet>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
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

#[derive(Deserialize, Asset, TypePath, Default, )]
/// something similar to godot's autotiling
pub struct AdjRetexConfigSeri (
    //(Vec(direction_str, tile_ezero_id)) -> animation ID or texture ID
    // higher in this, higher priority
    pub Vec<(Vec<(String, String)>, (String, Option<TileFlip>))>,
);

#[derive(Deserialize, Asset, TypePath, Default, )]
pub struct TileSeri {
    pub id: String,
    pub name: String,
    pub z: f32,
    pub img_paths: Vec<(String, String)>,
    pub tags: Option<HashSet<String>>,
    pub y_sort: Option<f32>,
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

    pub size_in_tiles: Option<(u32, u32)>,

    pub adj_retex: Option<AdjRetexConfigSeri>,
    ///if Some, is a ground tile and f32 the is walk speed modifier. if None or Some(0.0) is impassable tile
    pub walk_speed: Option<f32>,
    /// to be used by other systems to factor in their own walkspeed on top if a certain tag is present on this tile
    pub walk_speed_tags: Option<HashSet<String>>,

    /// When true, this tile spawns a projectile-stopping collider.
    pub blocks_projectiles: Option<bool>,

}

#[derive(Component, Deserialize, TypePath, Default, Clone)]
pub struct PortalSeri{
    pub dest_dimension: String,
    pub oe_tile: String,
    #[serde(default = "default_portal_terrprobe")]
    pub oe_terrprobe: String,
    pub one_way: Option<bool>,
    /// NASE
    pub dungeon: String,
}

fn default_portal_terrprobe() -> String {
    "portal_spiral".to_string()
}

#[derive(Deserialize, Asset, TypePath, )]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}
