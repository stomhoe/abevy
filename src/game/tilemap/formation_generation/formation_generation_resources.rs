use bevy::{platform::collections::HashMap, prelude::*};
use fastnoise_lite::FastNoiseLite;

#[derive(Resource, )]
pub struct WorldGenSettings {
    
    seed: u64,
    average_temp: f32,
    world_size: Option<u32>,
}
impl Default for WorldGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            average_temp: 15.0, 
            world_size: None 
        }
    }
}


#[derive(Resource, Default, )]
pub struct Textures (//mejorar eto
    pub HashMap<u32, Handle<Image>>,
);
