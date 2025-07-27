use bevy::{math::U16Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};

use crate::game::tilemap::tile::{
    tile_components::*, tile_constants::TILEMAP_TILE_SIZE_64,
    tile_constants::*,
};

#[derive(Debug, Default)]
pub struct TileimgConfig { size: U16Vec2, needs_y_sort: bool }

impl TileimgConfig {
    pub fn new(needs_y_sort: bool) -> Self {
        Self { size: U16Vec2::ZERO, needs_y_sort }
    }
    pub fn set_size(&mut self, size: U16Vec2) {self.size = size;}

    pub fn tile_size(&self) -> TilemapTileSize { TilemapTileSize::new(self.size.x as f32, self.size.y as f32) }
    pub fn tile_size_u16vec2(&self) -> U16Vec2 { self.size }
    pub fn needs_y_sort(&self) -> bool { self.needs_y_sort }
}

#[derive(Resource, Default)]
pub struct HandleConfigMap { pub map: HashMap<Tileimg, TileimgConfig>, size_loaded_count: u32 }

impl HandleConfigMap {

    pub fn get(&self, img: &Tileimg) -> Option<&TileimgConfig> {
       self.map.get(img)
    }
    pub fn get_mut(&mut self, img: &Tileimg) -> Option<&mut TileimgConfig> {
        self.map.get_mut(img)
    }
    pub fn set_size(&mut self, img: &Tileimg, size: U16Vec2) {
        unsafe{
         let cfg = self.map.get_mut(img).unwrap_unchecked();
         cfg.set_size(size);
         self.size_loaded_count += 1;
         info!("Set size for Tileimg {:?} to {:?}, count: {}", img, size, self.size_loaded_count);
        }
    }
    pub fn all_tile_sizes_loaded(&self) -> bool {
        self.size_loaded_count == self.map.len() as u32
    }

    pub fn insert<S: Into<String>>(
        &mut self, asset_server: &AssetServer,
        path: S,
        needs_y_sort: bool,
    ) {
        let img_cfg = TileimgConfig::new(needs_y_sort);
        let full_path = format!("{}{}", TILEIMG_BASE_PATH, path.into());
        let handle = asset_server.load(full_path);
        self.map.insert(Tileimg(handle), img_cfg);
    }

}