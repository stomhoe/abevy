use bevy::prelude::*;

use crate::{game_common_components::*, game_common_systems::*, };

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct StatefulSessionSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameplaySystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimRunningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimPausedSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
    ))
    .add_systems(Update, (
        (toggle_simulation, update_transform_z).in_set(GameplaySystems),
        (tick_time_based_multipliers).in_set(SimRunningSystems)
    ))
    .configure_sets(Update, (
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).in_set(StatefulSessionSystems)
    ))
    .configure_sets(FixedUpdate, (
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).in_set(StatefulSessionSystems)
    ))
    .register_type::<MyZ>()

    ;
}