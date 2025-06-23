use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;
use strum::IntoEnumIterator;

use crate::game::tilemap::formation_generation::formation_generation_utils::BaseZLevels;

#[derive(Resource, )]
pub struct TilemapSettings {
    pub chunk_visib_min_dist: f32,
    pub chunk_active_min_dist: f32,
}
impl Default for TilemapSettings {
    fn default() -> Self {
        Self {
            chunk_visib_min_dist: 1700.0,
            chunk_active_min_dist: 8000.0, 
        }
    }
}

#[derive(Default, Debug, Resource)]
pub struct LoadedChunks (pub HashMap<IVec2, Entity>,);

pub fn contpos_to_chunkpos(contpos: Vec2) -> IVec2 {
    let tile_size_in_pxs: IVec2 = IVec2::new(BASE_TILE_SIZE.x as i32, BASE_TILE_SIZE.y as i32);
    contpos.as_ivec2() / ( tile_size_in_pxs)
}

pub fn chunkpos_to_contpos(chunk_pos: IVec2) -> Vec2 {
    let tile_size: IVec2 = IVec2::new(BASE_TILE_SIZE.x as i32, BASE_TILE_SIZE.y as i32);
    chunk_pos.as_vec2() * tile_size.as_vec2()
}


pub const BASE_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 64.0, y: 64.0 };

//TODO HACER DEL TAMAÑO DE LO QUE ES VISIBLE EN PANTALLA
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };




pub const CNT: i32 = 3;