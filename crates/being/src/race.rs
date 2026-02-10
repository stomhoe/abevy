use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::HostSystems;

use crate::race::{race_build_being_systems::{build_beings_from_race_ref, sample_sprite_variations}, race_components::*, race_init_systems::*, race_resources::*};




#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystems;
#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_race,
        ))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            (
                (init_races, map_race_id_to_entity).chain()
            ).in_set(RaceSystems)
        )
        .add_systems(
            Update,
            (
                build_beings_from_race_ref
            ).in_set(RaceSystems)
        )
        .add_systems(
            Update,
            (
                sample_sprite_variations
            ).in_set(HostSystems)
        )

    ;
}

mod race_init_systems;
mod race_build_being_systems;
pub mod race_components;
pub mod race_resources;
