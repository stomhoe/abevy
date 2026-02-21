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
            plugin_body_weighted_sampler,
        ))
        .add_systems(Update, (
            (sample_nested_body_samplers_until_body_tree_is_found, ).chain().in_set(HostSystems),
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (init_body_weighted_samplers, map_body_weighted_sampler_id_to_entity, init_body_weighted_samplers_strid_refs,)
        ).chain().in_set(BodySamplerSystems))
        ;

}

pub mod body_sampler_components;
pub mod body_sampler_resources;
mod body_sampler_init_systems;
pub mod body_sampler_systems;
