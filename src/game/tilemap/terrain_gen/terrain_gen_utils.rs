use std::hash::Hasher;

use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};
use strum_macros::{Display, EnumCount, EnumIter, VariantNames};

use crate::game::tilemap::{terrain_gen::{terrain_gen_components::FnlComp, terrain_gen_resources::WorldGenSettings}, tilemap_resources::{chunkpos_to_pixelpos, chunkpos_to_tilepos, pixelpos_to_tilepos, CHUNK_SIZE, TILE_SIZE_PXS}, };

pub const Z_DIVISOR: i32 = 10000;

pub const GRASS_Z_LEVEL: i32 = 10 * Z_DIVISOR;


pub struct TileDto{
    pub layer_z: i32,//posteriormente se divide entre 10000
    pub pos_within_chunk: UVec2,
    pub used_handle: Handle<Image>,//NO SÉ SI USAR ESTO O UNA ID O ALGO ASÍ EN VEZ DE ESTE SHARED POINTER
    pub flip: TileFlip,
    pub color: TileColor,
    pub needs_y_sort: bool,
    pub visible: TileVisible,
    pub tile_size: TilemapTileSize,
    pub entity: Option<Entity>, 
}
impl Default for TileDto {
    fn default() -> Self {
        Self { 
            layer_z: 0,
            pos_within_chunk: UVec2::default(),
            used_handle: Handle::<Image>::default(),
            flip: TileFlip::default(),
            color: TileColor::default(),
            needs_y_sort: false,
            visible: TileVisible::default(),
            tile_size: TILEMAP_TILE_SIZE_64,
            entity: None,
        }
    }
}

pub const TILEMAP_TILE_SIZE_64: TilemapTileSize = tsize(64., 64.);
pub const TILEMAP_TILE_SIZE_128: TilemapTileSize = tsize(128., 128.);
const fn tsize(x: f32, y: f32) -> TilemapTileSize {TilemapTileSize::new(x, y)}
