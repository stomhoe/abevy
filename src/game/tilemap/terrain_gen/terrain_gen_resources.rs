use bevy::{math::U16Vec2, platform::collections::HashMap, prelude::*};
use bevy_ecs_tilemap::map::TilemapTileSize;
use fastnoise_lite::FastNoiseLite;


#[derive(Resource, )]
pub struct WorldGenSettings {
    
    pub seed: u32,
    pub average_temp: f32,
    pub world_size: Option<u32>,
}
impl Default for WorldGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            average_temp: 15.0, //esto debería usarse para reducir o incrementar threshold
            world_size: None 
        }
    }
}

