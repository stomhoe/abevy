use bevy::prelude::*;
use common::common_states::*;
use bevy_asset_loader::prelude::*;

use crate::{game_common_components::*, game_common_states::*, game_common_systems::* };

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct StatefulSessionSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameplaySystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimRunningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimPausedSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ModifierSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSystemsSet;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
    ))
    .add_systems(OnEnter(AppState::NoGameSession), (reset_states))

    .add_systems(Update, (
        (toggle_simulation, update_transform_z).in_set(GameplaySystems),
        (tick_time_based_multipliers).in_set(SimRunningSystems)
    ))
    .configure_sets(Update, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).in_set(StatefulSessionSystems).run_if(in_state(ReplicatedAssetsLoadingState::Finished)
        .and(in_state(GamePhase::ActiveGame))),
        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
    ))
    .configure_sets(FixedUpdate, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).in_set(StatefulSessionSystems).run_if(in_state(ReplicatedAssetsLoadingState::Finished)
        .and(in_state(GamePhase::ActiveGame))),
        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
    ))
     .add_loading_state(
        LoadingState::new(LocalAssetsLoadingState::InProcess).continue_to_state(LocalAssetsLoadingState::Finished)

    )
    .add_loading_state(
        LoadingState::new(ReplicatedAssetsLoadingState::InProcess).continue_to_state(ReplicatedAssetsLoadingState::Finished)
  
        //.load_collection::<DimensionSerisHandles>()
    )

    .init_state::<GameSetupScreen>()
    .init_state::<SimulationState>()
    .register_type::<MyZ>()
    .register_type::<Description>()
    .register_type::<BeingAltitude>()

    ;
}