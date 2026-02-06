use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use common::common_states::AssetLoading;
use being_shared::BodyTreeStrIdRef;

use crate::race::{race_init_systems::*, race_resources::*, race_components::*};




#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystems;
#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            RonAssetPlugin::<RaceSerialization>::new(&["race.ron"]),
            plugin_race_entity_map,
        ))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            (
                (init_races, map_race_entity_map_id_to_entity).chain()
            ).in_set(RaceSystems)
        )
        .add_systems(Update, (
            map_race_entity_map_id_to_entity,
        ))
        .register_type::<BodyTreeStrIdRef>()
    ;
}

mod race_init_systems;
mod race_build_being_systems;
pub mod race_components;
pub mod race_resources;
