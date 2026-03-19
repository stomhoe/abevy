#[allow(unused_imports)] use bevy::prelude::*;

use crate::body::body_sampler::body_sampler_components::BodyWeightedSampler;
pub use crate::body::body_sampler::body_sampler_seris::*;

common::define_entity_map_systems!(
    BodyWeightedSampler,
    BodyWeightedSamplerSeri, "seri.being.body.sampler", "bosampler.ron",
);

pub type BodyWeightedSamplerHandles = BodyWeightedSamplerSerisHandles;
