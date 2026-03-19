#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;

use crate::terrain::biome::{
    biome_components::*,
    biome_init_systems::*,
    biome_resources::*,
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

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        biome_components::*,
        biome_resources::*,
        biome_seris::*,
    };
}
