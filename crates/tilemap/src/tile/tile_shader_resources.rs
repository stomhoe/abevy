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
    pub handles: Vec<Handle<ShaderVoronoiSeri>>,
}


#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderVoronoiSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub voronoi_scale: f32,
    pub voronoi_scale_random: f32,
    pub voronoi_rotation: f32,
    pub mask_color: [f32; 4],
}