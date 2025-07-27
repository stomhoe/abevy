use bevy::{math::U8Vec2, platform::collections::{HashMap, HashSet}};
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Resource, )]
pub struct ChunkRangeSettings {
    pub chunk_visib_max_dist: f32,
    pub chunk_active_max_dist: f32,
    pub chunk_show_range: u8,
}
impl Default for ChunkRangeSettings {
    fn default() -> Self {
        Self {
            chunk_visib_max_dist: 2000.0,
            chunk_active_max_dist: 2000.0,//POSIBLE BUG, SI ESTO ES MÁS BAJO Q EL SHOW RANGE, SE DESPAWNEAN Y RESPAWNEAN CONSTANTEMENTE LOS CHUNKS 
            chunk_show_range: 1,//no subir mucho o afecta visiualización sprites en movimiento
        }
    }
}

#[derive(Default, Debug, Resource)]
pub struct LoadedChunks (pub HashMap<IVec2, Entity>,);


pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 64, y: 64 };

pub const CHUNK_SIZE: U8Vec2 = U8Vec2 { x: 5, y: 5 };

