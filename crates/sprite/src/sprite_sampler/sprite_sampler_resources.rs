use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SpriteWeightedSamplerHandles {
    #[asset(path = "ron/sprite/sampler", collection(typed))]
    pub handles: Vec<Handle<SpriteWeightedSamplerSeri>>,
}

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct SpriteWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>, // sprite_id or sampler_id* with weight
}
