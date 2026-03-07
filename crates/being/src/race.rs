use bevy::prelude::*;
#[allow(unused_imports, )]
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use crate::race::{race_components::*, race_init_systems::*, race_resources::*};




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
pub mod race_components;
pub mod race_resources;
pub mod race_seris;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        race_components::*,
        race_resources::*,
        race_seris::*,
    };
}
