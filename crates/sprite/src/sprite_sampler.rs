#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use common::define_entity_map_systems;
use game_common::HostSystems;

use crate::sprite_sampler::sprite_sampler_components::*;
use crate::sprite_sampler::sprite_sampler_init_systems::*;
use crate::sprite_sampler::sprite_sampler_resources::*;
use crate::sprite_sampler::sprite_sampler_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSamplerSystems;


define_entity_map_systems!(
    SpriteWeightedSampler,
    SpriteWeightedSamplerSeri, "ron/sprite/sampler", "sampler.ron",
);

pub type SpriteWeightedSamplerHandles = SpriteWeightedSamplerSerisHandles;

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_sprite_weighted_sampler,
        ))
        .add_systems(Update, (
            (replace_sampler_string_ids_by_entities, sample_from_sprite_entities).in_set(HostSystems),
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (init_sprite_weighted_samplers, map_sprite_weighted_sampler_id_to_entity, init_sprite_weighted_samplers_refs,)
        ).chain().in_set(SpriteSamplerSystems))
        .register_type::<SpriteWeightedSamplerHandles>()
        .register_type::<SpriteWeightedSamplerSeri>()

        
        .replicate::<EguiSpriteSamplerHolder>()
        .replicate::<SpriteWeightedSampler>()
        ;

}

pub mod sprite_sampler_components;
pub mod sprite_sampler_resources;
mod sprite_sampler_init_systems;
pub mod sprite_sampler_systems;