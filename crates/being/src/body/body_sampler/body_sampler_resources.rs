use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use crate::body::body_sampler::body_sampler_components::BodyWeightedSampler;


common::define_entity_map_systems!(
    BodyWeightedSampler
);

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct BodyWeightedSamplerHandles {
    #[asset(path = "ron/being/body/sampler", collection(typed))]
    pub handles: Vec<Handle<BodyWeightedSamplerSeri>>,
}


#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct BodyWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
    pub extra: Option<HashMap<String, String>>,
}
