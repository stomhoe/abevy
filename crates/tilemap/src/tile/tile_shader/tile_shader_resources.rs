#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::common_types::HashIdToEntityMap;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Reflect )]
#[reflect(Resource, Default)]
pub struct TileShaderEntityMap(pub HashIdToEntityMap);

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
pub struct ShaderVoronoiSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/voro" , collection(typed))] 
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
    pub scale: f32,
    pub speed: [f32; 2],
    pub amplitude: f32,
    pub wave_color: [f32; 4],
    pub cell_scale: f32,
    pub seam_strength: f32,
    pub highlight_strength: f32,
    pub warp_strength: f32,
    pub flow_speed: f32,
}  