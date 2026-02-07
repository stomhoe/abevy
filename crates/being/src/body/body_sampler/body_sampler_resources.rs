use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

use crate::body::body_sampler::body_sampler_components::BodyWeightedSampler;


#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct BodyWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
    pub extra: Option<HashMap<String, String>>,
}

common::define_entity_map_systems!(
    BodyWeightedSampler,
    BodyWeightedSamplerSeri, "ron/being/body/sampler", "sampler.ron",
);

pub type BodyWeightedSamplerHandles = BodyWeightedSamplerSerisHandles;
