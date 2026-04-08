#[allow(unused_imports)] use bevy::prelude::*;

use crate::body::body_sampler::body_sampler_components::BodyWeightedSampler;
pub use crate::body::body_sampler::body_sampler_seris::*;

common::define_entity_map_systems!(
    main_component: BodyWeightedSampler,
    with_filters: (),
    abbreviation: BodyWeightedSampler,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: BodyWeightedSampler,
    id_type: common::common_components::StrId,
    assets: [(BodyWeightedSamplerSeri, "seri.being.body.sampler", "bosampler.ron")],
);
