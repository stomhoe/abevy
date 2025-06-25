use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapGridSize};


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


pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 64, y: 64 };

//TODO HACER DEL TAMAÑO DE LO QUE ES VISIBLE EN PANTALLA
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };

