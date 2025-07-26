
use bevy::{math::U16Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;


// NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO

const BASE_PATH: &str = "texture/world/";



#[allow(unused_parens)]
pub fn setup_nid_img_map(mut commands: Commands, asset_server: Res<AssetServer>, mut m: ResMut<NidImgMap>, 
) {

    m.insert_64(&asset_server, IMG_WHITE, "white.png", 0, false);
    m.insert_64(&asset_server, TileImgId::new("ejemplo"), "white.png", 0, false);




}
pub const IMG_WHITE: TileImgId = TileImgId(0);



#[derive(Component, Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct TileImgId(u64);

impl TileImgId {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        let mut hasher = DefaultHasher::new();
        id.as_ref().hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct ImgIngameCfg { handle: Handle<Image>, size: U16Vec2, z_index: i32, needs_y_sort: bool }

impl ImgIngameCfg {
    pub fn new(handle: Handle<Image>, image_size: U16Vec2, z_index: i32, needs_y_sort: bool) -> Self {
        Self { handle, size: image_size, needs_y_sort, z_index }
    }
    pub fn cloned_handle(&self) -> Handle<Image> {self.handle.clone()}
    pub fn tile_size(&self) -> TilemapTileSize { TilemapTileSize::new(self.size.x as f32, self.size.y as f32) }
    pub fn tile_size_u16vec2(&self) -> U16Vec2 { self.size }
    pub fn z_index(&self) -> i32 { self.z_index }
    pub fn needs_y_sort(&self) -> bool { self.needs_y_sort }
}

#[derive(Resource, Default)]
pub struct NidImgMap(pub HashMap<TileImgId, ImgIngameCfg>);

impl NidImgMap {
    pub fn get(&self, id: TileImgId) -> Option<&ImgIngameCfg> { self.0.get(&id) }

    pub fn insert<S: Into<String>>(
        &mut self, asset_server: &AssetServer,
        id: TileImgId,
        path: S,
        image_size: U16Vec2,
        z_index: i32,
        needs_y_sort: bool,
    ) {
        let full_path = format!("{}{}", BASE_PATH, path.into());
        let handle = asset_server.load(full_path);
        let img_cfg = ImgIngameCfg::new(handle, image_size, z_index, needs_y_sort);
        self.0.insert(id, img_cfg);
    }

    //TODO https://stackoverflow.com/questions/70657798/get-width-and-height-from-an-image-in-bevy

    pub fn insert_64<S: Into<String>>(
        &mut self, asset_server: &AssetServer,
        id: TileImgId,
        path: S,
        z_index: i32,
        needs_y_sort: bool,
    ) {
        self.insert(asset_server, id, path, TILEMAP_TILE_SIZE_64, z_index, needs_y_sort);
    }
}

pub const TILEMAP_TILE_SIZE_64: U16Vec2 = U16Vec2::new(64, 64);
pub const TILEMAP_TILE_SIZE_128: U16Vec2 = U16Vec2::new(128, 128);
