#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::common_types::HashIdToEntityMap;
use serde::{Deserialize, Serialize};

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderRepeatTexSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/rep1" , collection(typed))] 
    pub handles: Vec<Handle<ShaderRepeatTexSeri>>,
}


#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub mask_color: [f32; 4],
}

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderVoroshuSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/voroshu" , collection(typed))] 
    pub handles: Vec<Handle<ShaderVoronoiShuffleSeri>>,
}


#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderVoronoiShuffleSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub voronoi_scale: f32,
    pub voronoi_scale_random: f32,
    pub voronoi_rotation: f32,
    pub mask_color: [f32; 4],
}


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderWavySerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/wavy" , collection(typed))] 
    pub handles: Vec<Handle<ShaderWavySeri>>,
}

#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderWavySeri {
    pub id: String,
    // Path to overlay texture (use "placeholder" if unknown)
    pub img_path: String,
    pub mask_color: [f32; 4],
    pub scale: f32,
    pub time: f32,
    pub speed: f32,
    pub debug_mode: f32,
}

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderRockyTerrainSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/rocky" , collection(typed))] 
    pub handles: Vec<Handle<ShaderRockyTerrainSeri>>,
}

#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderRockyTerrainSeri {
    pub id: String,
    pub roughness: f32,
    pub scale: f32,
    pub height_scale: f32,
    pub color_base: [f32; 4],
    pub color_shadow: [f32; 4],
}