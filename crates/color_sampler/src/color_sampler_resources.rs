use bevy::{math::f32, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct ColorWeightedSamplerHandles {
    #[asset(path = "ron/color_sampler", collection(typed))] 
    pub handles: Vec<Handle<WeightedColorsSeri>>,
}


#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct WeightedColorsSeri {
    pub id: String,
    pub weights: Vec<([u8; 4], f32)>,
}
impl WeightedColorsSeri {
    pub const MIN_ID_LENGTH: u8 = 3;
}



