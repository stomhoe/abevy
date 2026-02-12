use bevy::math::f32;
#[allow(unused_imports)] use bevy::prelude::*;
use common::define_entity_map_systems;

use crate::tile::tile_sampler_components::TileWeightedSampler;

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct TileWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
}

define_entity_map_systems!(
    TileWeightedSampler,
    TileWeightedSamplerSeri, "ron/tilemap/tiling/weighted_sampler", "tsampler.ron",
);
