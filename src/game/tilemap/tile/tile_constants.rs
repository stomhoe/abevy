use bevy::math::U16Vec2;
#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::tilemap::tile::tile_components::*;


pub const TILEIMG_BASE_PATH: &str = "texture/world/";



pub const TILEMAP_TILE_SIZE_64: U16Vec2 = U16Vec2::new(64, 64);
pub const TILEMAP_TILE_SIZE_128: U16Vec2 = U16Vec2::new(128, 128);
