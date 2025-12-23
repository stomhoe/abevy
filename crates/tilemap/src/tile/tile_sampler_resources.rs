use bevy::math::f32;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::common_types::HashIdToEntityMap;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Clone, Reflect)]
#[reflect(Resource, Default)]
pub struct TileWeightedSamplersMap(pub HashIdToEntityMap);

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileWeightedSamplerHandles {
    #[asset(path = "ron/tilemap/tiling/weighted_sampler", collection(typed))] 
    pub handles: Vec<Handle<TileWeightedSamplerSeri>>,
}

#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct TileWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
}




