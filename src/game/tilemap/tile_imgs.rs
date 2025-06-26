use bevy::{math::U16Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};


// NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO

const BASE_PATH: &str = "textures/world/";



#[allow(unused_parens)]
pub fn setup_nid_img_map(mut commands: Commands, asset_server: Res<AssetServer>, mut m: ResMut<NidImgMap>, 
) {

    m.insert_64(&asset_server,IMG_WHITE, "white.png", 0, false);




}
pub const IMG_WHITE: TileImgNid = TileImgNid(0);



#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct TileImgNid(pub u32);

#[derive(Debug, Default)]
pub struct ImgIngameCfg { pub handle: Handle<Image>, pub size: U16Vec2, pub z_index: i32, pub needs_y_sort: bool }

impl ImgIngameCfg {
    pub fn new(handle: Handle<Image>, image_size: U16Vec2, z_index: i32, needs_y_sort: bool) -> Self {
        Self { handle, size: image_size, needs_y_sort, z_index }
    }
    pub fn get_tile_size(&self) -> TilemapTileSize { TilemapTileSize::new(self.size.x as f32, self.size.y as f32) }
    pub fn get_z_index(&self) -> i32 { self.z_index }
}

#[derive(Resource, Default)]
pub struct NidImgMap(pub HashMap<TileImgNid, ImgIngameCfg>);

impl NidImgMap {
    pub fn get(&self, nid: TileImgNid) -> Option<&ImgIngameCfg> { self.0.get(&nid) }
    pub fn insert<S: Into<String>>(&mut self, asset_server: &AssetServer, nid: TileImgNid, path: S, image_size: U16Vec2, z_index: i32, needs_y_sort: bool) {
        let image_ing_data = ImgIngameCfg::new(asset_server.load(format!("{}{}", BASE_PATH, path.into())), image_size, z_index, needs_y_sort);
        self.0.insert(nid, image_ing_data);
    }
    pub fn insert_64<S: Into<String>>(&mut self, asset_server: &AssetServer, nid: TileImgNid, path: S, z_index: i32, needs_y_sort: bool) {
        self.insert(asset_server, nid, path, TILEMAP_TILE_SIZE_64, z_index, needs_y_sort);
    }
}

pub const TILEMAP_TILE_SIZE_64: U16Vec2 = U16Vec2::new(64, 64);
pub const TILEMAP_TILE_SIZE_128: U16Vec2 = U16Vec2::new(128, 128);

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct RepImgNid(pub u32);

#[derive(Debug, Default, Clone)]
pub struct RepeatImgCfg { pub handle: Handle<Image>, pub scale: f32 }

impl RepeatImgCfg {
    pub fn new(handle: Handle<Image>, scale: f32) -> Self { Self { handle, scale } }
}

#[derive(Resource, Default)]
pub struct NidRepeatImgMap(pub HashMap<RepImgNid, RepeatImgCfg>);

impl NidRepeatImgMap {
    pub fn get(&self, nid: RepImgNid) -> Option<&RepeatImgCfg> { self.0.get(&nid) }
    pub fn get_handle(&self, nid: RepImgNid) -> Option<Handle<Image>> { self.0.get(&nid).map(|cfg| cfg.handle.clone()) }
    pub fn get_scale(&self, nid: RepImgNid) -> Option<f32> { self.0.get(&nid).map(|cfg| cfg.scale) }
    pub fn insert<S: Into<String>>(&mut self, asset_server: &AssetServer, nid: RepImgNid, path: S, scale: f32) {
        let handle = asset_server.load(format!("{}{}", BASE_PATH, path.into()));
        self.0.insert(nid, RepeatImgCfg::new(handle, scale));
    }
}





#[allow(unused_parens)]
pub fn setup_rep_img_map(mut commands: Commands, asset_server: Res<AssetServer>,
    mut m: ResMut<NidRepeatImgMap>
) {
    m.insert(
        &asset_server,
        REPIMG_GRASS,
        "/terrain/temperate_grass/grass.png",
        0.001,
    );


pub const REPIMG_GRASS: RepImgNid = RepImgNid(0);
}