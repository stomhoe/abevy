use bevy::{math::U16Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;


// NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO

const BASE_PATH: &str = "textures/world/";

#[allow(unused_parens)]
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut nid_img_map: ResMut<NidImgMap>, ) {
    nid_img_map.insert_size64(IMG_WHITE, asset_server.load(format!("{}/white.png", BASE_PATH)));
    
}

#[derive(Resource, Default, )]
pub struct NidImgMap (
    pub HashMap<TileImgNid, (Handle<Image>, U16Vec2)>,//NO METERLE OTRAS COSAS, ESTO ES SOLO PARA MAPEAR AL HANDLE. 
);

impl NidImgMap {
    pub fn get(&self, nid: TileImgNid) -> Option<(Handle<Image>, U16Vec2)> {
        self.0.get(&nid).map(|(handle, image_size)| (handle.clone(), *image_size))
    }

    pub fn get_handle(&self, nid: TileImgNid) -> Option<Handle<Image>> {
        self.0.get(&nid).map(|(handle, _)| handle.clone())
    }

    pub fn get_image_size(&self, nid: TileImgNid) -> Option<U16Vec2> {
        self.0.get(&nid).map(|(_, image_size)| *image_size)
    }

    

    pub fn insert(&mut self, nid: TileImgNid, handle: Handle<Image>, image_size: U16Vec2) {
        self.0.insert(nid, (handle, image_size));
    }

    pub fn insert_size64(&mut self, nid: TileImgNid, handle: Handle<Image>) {
        let default_image_size = TILEMAP_TILE_SIZE_64;
        self.insert(nid, handle, default_image_size);
    }
}


pub const TILEMAP_TILE_SIZE_64: U16Vec2 = U16Vec2::new(64, 64);
pub const TILEMAP_TILE_SIZE_128: U16Vec2 = U16Vec2::new(128, 128);





#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, )]
pub struct TileImgNid(pub u32);

pub const IMG_WHITE: TileImgNid = TileImgNid(0);
