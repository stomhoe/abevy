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
    main_component: TileWeightedSampler,
    with_filters: (),
    abbreviation: TileWeightedSampler,
    target: "",
    entity_prefix: "tile weighted sampler",
    despawn_trigger: TileWeightedSampler,
    id_type: common::common_components::StrId,
    assets: [(TileWeightedSamplerSeri, "seri.tilemap.tile.weighted_sampler", "tsampler.ron")]
);
