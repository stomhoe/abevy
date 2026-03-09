#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::HostSystems;

use crate::sprite_sampler::sprite_sampler_components::*;
use crate::sprite_sampler::sprite_sampler_init_systems::*;
use crate::sprite_sampler::sprite_sampler_systems::*;
pub use sprite_shared::sprite_sampler::*;

common::define_entity_map_systems!(
    SpriteWeightedSampler,
    SpriteWeightedSamplerSeri, "seri.sprite.weighted_sampler", "ssampler.ron",
);

pub type SpriteWeightedSamplerHandles = SpriteWeightedSamplerSerisHandles;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSamplerSystems;

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
        .replicate::<EguiSpriteSamplerHolder>()
        .replicate::<SpriteWeightedSampler>()
        ;

}

pub mod sprite_sampler_components;
pub mod sprite_sampler_seris;
mod sprite_sampler_init_systems;
pub mod sprite_sampler_systems;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        sprite_sampler_components::*,
        sprite_sampler_init_systems::*,
        sprite_sampler_seris::*,
        sprite_sampler_systems::*,
    };
}
