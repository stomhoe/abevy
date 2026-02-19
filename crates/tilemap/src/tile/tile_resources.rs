use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, math::f32, platform::collections::{HashMap, HashSet}, };
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use common::{common_components::{Tag}, };
use game_common::game_common_components::EntityZero;
use serde::{Deserialize, Serialize};
use tilemap_shared::InteractionZoneSeri;

pub use crate::tilemap_resources::{MassCollectedTiles, ImportantRegisteredPositions, CloneSpawnParamSet};
use crate::tile::tile_components::{DeleteOtherTiles, DeleteOtherTilesSeri, Tile};

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
    #[serde(default)]
    pub tags: HashSet<String>,
    pub y_sort: Option<f32>,
    /// persisted only when state gets altered from starting state
    #[serde(default)]
    pub persisted: bool,
    #[serde(default)]
    pub shader: String,
    #[serde(default)]
    pub is_spritetile: bool,
    pub color: Option<[u8; 4]>,
    #[serde(default)]
    pub color_map: String,
    #[serde(default)]
    pub spawns: Vec<String>,
    #[serde(default)]
    pub spawns_children: Vec<String>,
    #[serde(default)]
    pub randflipx: bool,
    #[serde(default)]
    pub randflipy: bool,
    #[serde(default)]
    pub randflipd: bool,
    #[serde(default)]
    pub min_distances: HashMap<String, u64>,
    #[serde(default)]
    pub portal: PortalSeri,
    #[serde(default)]
    pub offset: (f32, f32),


    #[serde(default)]
    pub interaction_zones: HashMap<String, InteractionZoneSeri>,

    #[serde(default)]
    pub offsets_for_portal_arrivals: Vec<(f32, (i8, i8))>,

    #[serde(default)]
    pub delete_other_tiles: DeleteOtherTilesSeri,
    #[serde(default)]
    pub terrgen_offset: (i8, i8),

    #[serde(default = "default_size_in_tiles")]
    pub size_in_tiles: (u32, u32),
    /// Optional per-cell collision mask, row-major, '1' blocks movement, '0' is passable.
    #[serde(default)]
    pub colmask: Vec<String>,

    pub adj_retex: Option<AdjRetexConfigSeri>,
    #[serde(default = "default_walk_speed")]
    pub walk_speed: f32,
    /// to be used by other systems to factor in their own walkspeed on top if a certain tag is present on this tile
    #[serde(default)]
    pub walk_speed_tags: HashSet<String>,

    /// When true, this tile spawns a projectile-stopping collider.
    #[serde(default)]
    pub blocks_projectiles: bool,
}
fn default_walk_speed() -> f32 { 1. }
fn default_size_in_tiles() -> (u32, u32) { (1, 1) }



#[derive(Component, Deserialize, TypePath, Clone, )]
pub struct PortalSeri{
    pub dest_dimension: String,
    pub oe_tile: String,
    #[serde(default = "default_portal_terrprobe")]
    pub oe_terrprobe: String,
    #[serde(default)]
    pub one_way: bool,
    #[serde(default)]
    pub dungeon: String,
    #[serde(default)]
    //weight, position (to be converted into GlobalTilePOs)
    pub offset_pos_destinations: Vec<(f32, (i8, i8))>,
}
impl PortalSeri {
    pub fn no_field_is_empty(&self) -> bool {
        !self.dest_dimension.is_empty() && !self.oe_terrprobe.is_empty()
    }

}

impl Default for PortalSeri {
    fn default() -> Self {
        Self {
            dest_dimension: "".to_string(),
            oe_tile: "".to_string(),
            oe_terrprobe: default_portal_terrprobe(),
            one_way: false,
            dungeon: "".to_string(),
            offset_pos_destinations: Vec::new(),
        }
    }
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
