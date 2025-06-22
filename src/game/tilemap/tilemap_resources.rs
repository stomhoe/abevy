use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

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
pub struct ChunkManager {
    pub loaded_chunks: HashMap<IVec2, Entity>,
}