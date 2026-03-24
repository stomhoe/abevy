use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct SpriteTile;

#[derive(Component, Debug, Clone, Default)]
/// maps handle's ids to texture index to use within tilemap as a tile belonging to it
pub struct HashIdToTexIndex(HashIdMap<TileTextureIndex>);
impl HashIdToTexIndex {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashIdMap::with_capacity(capacity))
    }
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    pub fn insert(&mut self, tile_hid: HashId, handle_hid: HashId, tex_index: TileTextureIndex) {
        let _ = self.0.insert(tile_hid.merge(handle_hid), tex_index);
    }
    pub fn get(&self, tile_hid: HashId, handle_hid: HashId) -> Result<TileTextureIndex, ()> {
        let merged = tile_hid.merge(handle_hid);
        self.0.get(merged).cloned()
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct WalkSpeedMultIfOnTop(pub f32);
impl WalkSpeedMultIfOnTop {
    pub fn is_extremely_low(&self) -> bool {
        self.0 <= 0.01
    }
}
impl Default for WalkSpeedMultIfOnTop {
    fn default() -> Self {
        Self(1.0)
    }
}
