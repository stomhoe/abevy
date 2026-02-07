use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::HostSystems;

use crate::body::body_sampler::body_sampler_components::*;
use crate::body::body_sampler::body_sampler_init_systems::*;
use crate::body::body_sampler::body_sampler_resources::*;
use crate::body::body_sampler::body_sampler_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodySamplerSystems;




pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            RonAssetPlugin::<BodyWeightedSamplerSeri>::new(&["sampler.ron"]),
            plugin_body_weighted_sampler,
        ))
        .add_systems(Update, (
            (replace_body_sampler_string_id_by_entity, sample_from_body_entities).in_set(HostSystems),
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (init_body_weighted_samplers, map_body_weighted_sampler_id_to_entity, init_body_weighted_samplers_refs,)
        ).chain().in_set(BodySamplerSystems))
        .register_type::<BodyWeightedSamplerHandles>()
        .register_type::<BodyWeightedSamplerSeri>()
        .replicate::<EguiBodySamplerHolder>()
        .replicate::<BodyWeightedSampler>()
        ;

}

pub mod body_sampler_components;
pub mod body_sampler_resources;
mod body_sampler_init_systems;
pub mod body_sampler_systems;
