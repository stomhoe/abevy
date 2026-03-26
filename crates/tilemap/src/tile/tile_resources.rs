
#[allow(unused_imports, )]
use bevy::{math::f32, };
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use common::{common_components::{Tag}, };
use game_common::game_common_components::TemplEnti;
use serde::{Deserialize, Serialize};
pub use ::tilemap_shared::*;

pub use crate::tilemap_resources::*;
use crate::tile::tile_seris::TileSeri;
use crate::tile::tile_components::*;

common::define_entity_map_systems!(
    Tile,
    (With<TemplEnti>, common::AnyDisabling),
    (Tile, TemplEnti),
    TileSeri, "seri.tilemap.tile", "tile.ron",
);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
//hacer singleton entity pa replicar. esto se guardaría en la save.
pub struct TileIndexing  {
    map: HashMap<TileIndex, common::HashId>,
    #[serde(skip)]
    curr_i: TileIndex }
impl TileIndexing {
    pub fn register_templ_tile(&mut self, templ_hash_id: common::HashId) -> TileIndex {
        let i = self.curr_i;
        self.curr_i.0 += 1;
        self.map.insert(i, templ_hash_id);
        i
    }

    pub fn hash_id_for_index(&self, tile_index: TileIndex) -> Option<common::HashId> {
        self.map.get(&tile_index).copied()
    }
}


#[derive(Resource, Debug, Default)]
pub struct TemplTileEntsWithinTag (pub HashMap<Tag, EntityHashSet>);

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
