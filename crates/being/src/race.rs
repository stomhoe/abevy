use ::being_shared::*;
use bevy::prelude::*;
#[allow(unused_imports, )]
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use crate::race::{race_init_systems::*, };




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

    ;
}

mod race_init_systems;
