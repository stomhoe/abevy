use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;

use crate::terrain::biome::{
    biome_init_systems::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BiomeSystems;

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_biome,
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (
                init_biomes,
                map_biome_id_to_entity,
            ).chain()
        ).in_set(BiomeSystems))
        ;
}

pub mod biome_components;
pub mod biome_resources;
pub mod biome_seris;
mod biome_init_systems;

pub use biome_components::*;
pub use biome_resources::*;
