use bevy::{math::f32, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::common_types::HashIdToEntityMap;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event, Reflect)]
#[reflect(Resource, Default)]
pub struct ColorWeightedSamplersMap(pub HashIdToEntityMap);



#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ColorWeightedSamplerHandles {
    #[asset(path = "ron/color_sampler", collection(typed))] 
    pub handles: Vec<Handle<WeightedColorsSeri>>,
}
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct WeightedColorsSeri {
    pub id: String,
    pub weights: HashMap<[u8; 4], f32>,
}
impl WeightedColorsSeri {
    pub const MIN_ID_LENGTH: u8 = 3;
}



