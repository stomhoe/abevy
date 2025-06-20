use bevy::prelude::*;

#[derive(Resource, )]
pub struct WorldSettings {
    
    average_temp: f32,
    world_size: Option<u32>,
}
impl Default for WorldSettings {
    fn default() -> Self {
        Self { 
            average_temp: 15.0, 
            world_size: None 
        }
    }
}