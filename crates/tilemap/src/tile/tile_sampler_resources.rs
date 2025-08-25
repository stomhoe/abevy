use bevy::{math::f32, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::common_types::HashIdToEntityMap;
use serde::{Deserialize, Serialize};


#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event, Reflect)]
#[reflect(Resource, Default)]
pub struct TileWeightedSamplersMap(pub HashIdToEntityMap);



#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileWeightedSamplerHandles {
    #[asset(path = "ron/tilemap/tiling/weighted_sampler", collection(typed))] 
    pub handles: Vec<Handle<TileWeightedSamplerSeri>>,
}
//SE PUEDE USAR TMB PARA SAMPLEAR COLORES PARA TILES
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct TileWeightedSamplerSeri {
    pub id: String,
    pub weights: HashMap<String, f32>,
}



