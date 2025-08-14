use bevy::{math::f32, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::common_types::HashIdToEntityMap;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Reflect )]
#[reflect(Resource, Default)]
pub struct TileShaderEntityMap(pub HashIdToEntityMap);


#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event, Reflect)]
#[reflect(Resource, Default)]
pub struct TilingEntityMap(pub HashIdToEntityMap);



#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileSerisHandles {
    #[asset(path = "ron/tilemap/tiling/tile" , collection(typed))] 
    pub handles: Vec<Handle<TileSeri>>,
}
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct TileSeri {
    pub id: String,
    pub name: String,
    pub img_paths: HashMap<String, String>,
    pub shader: String,
    pub sprite: bool,
    pub offset: [f32; 2],
    pub z: i32,
    pub color: Option<[u8; 4]>,
    pub color_map: String,
    pub spawns: Vec<String>,
    pub spawns_children: Vec<String>,
    pub somecomp_present: Option<bool>,
    pub child_of_chunk: bool,
}


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileWeightedSamplerSerisHandles {
    #[asset(path = "ron/tilemap/tiling/weighted_sampler" , collection(typed))] 
    pub handles: Vec<Handle<TileWeightedSamplerSeri>>,
}
//SE PUEDE USAR TMB PARA SAMPLEAR COLORES PARA TILES
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct TileWeightedSamplerSeri {
    pub id: String,
    pub weights: HashMap<String, f32>,
}

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderRepeatTexSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader" , collection(typed))] 
    pub handles: Vec<Handle<ShaderRepeatTexSeri>>,
}


#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub mask_color: [f32; 4],
}


