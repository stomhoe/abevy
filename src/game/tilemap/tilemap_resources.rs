use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapTileSize};
use strum::IntoEnumIterator;

use crate::game::tilemap::formation_generation::formation_generation_utils::BaseZLevels;

#[derive(Resource, )]
pub struct ChunkRangeSettings {
    pub chunk_visib_max_dist: f32,
    pub chunk_active_max_dist: f32,
    pub chunk_show_range: u8,
}
impl Default for ChunkRangeSettings {
    fn default() -> Self {
        Self {
            chunk_visib_max_dist: 1700.0,
            chunk_active_max_dist: 8000.0, 
            chunk_show_range: 2,
        }
    }
}

#[derive(Default, Debug, Resource)]
pub struct LoadedChunks (pub HashMap<IVec2, Entity>,);

pub fn contpos_to_chunkpos(contpos: Vec2) -> IVec2 {
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size_in_pxs: IVec2 = IVec2::new(TILE_SIZE_PXS.x as i32, TILE_SIZE_PXS.y as i32);
    contpos.as_ivec2().div_euclid(tile_size_in_pxs * chunk_size)
}

pub fn chunkpos_to_contpos(chunk_pos: IVec2) -> Vec2 {
    let tile_size_in_pxs: IVec2 = IVec2::new(TILE_SIZE_PXS.x as i32, TILE_SIZE_PXS.y as i32);
    chunk_pos.as_vec2() * tile_size_in_pxs.as_vec2() * CHUNK_SIZE.as_vec2() 
}
pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 64, y: 64 };

//TODO HACER DEL TAMAÑO DE LO QUE ES VISIBLE EN PANTALLA
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };

