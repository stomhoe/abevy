use bevy::{platform::collections::HashMap, prelude::*};

use crate::chunking_components::ChunkPos;

#[derive(Default, Debug, Resource)]
//pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);
pub struct LoadedChunks (pub HashMap<ChunkPos, Entity>,);

#[derive(Resource, )]
pub struct ChunkRangeSettings {
    pub chunk_visib_max_dist: f32,
    pub chunk_active_max_dist: f32,
    pub chunk_show_range: u8,
}
impl Default for ChunkRangeSettings {
    fn default() -> Self {
        Self {
            chunk_visib_max_dist: 1000.0,
            chunk_active_max_dist: 1000.0,//POSIBLE BUG, SI ESTO ES MÁS BAJO Q EL SHOW RANGE, SE DESPAWNEAN Y RESPAWNEAN CONSTANTEMENTE LOS CHUNKS 
            chunk_show_range: 1,//no subir mucho o afecta visiualización sprites en movimiento
        }
    }
}
