use bevy::prelude::*;
use common::{common_states::*};
use bevy_replicon::prelude::*;

use crate::{game_common_components::*, game_common_components_samplers::*, game_common_states::*, game_common_string_components::Description, game_common_systems::* };

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
pub struct HostSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {

    app
    
    .add_systems(Update, (
        (toggle_simulation, ).in_set(GameplaySystems),
        (tick_time_based_multipliers).in_set(SimRunningSystems),
        clone_ezero_children_ents,
        set_entity_name,
        tick_despawn_timers.in_set(HostSystems),
        tick_sim_despawn_timers.in_set(HostSystems).in_set(SimRunningSystems),
        //disable_ezeros,
        despawn_sprites_without_childof,
    ))
    .configure_sets(Update, (
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
        (ModifierSystems, ).in_set(SimRunningSystems),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(
                (in_state(ClientState::Connected))
                .or(in_state(AssetLoading::SpawnReplicatedEntities).and(in_state(ClientState::Disconnected)))
            )
        ).in_set(StatefulSessionSystems),
        
        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        HostSystems.run_if(in_state(ClientState::Disconnected)),
        ClientSystems.run_if(not(in_state(ClientState::Disconnected))),
    ))
    .configure_sets(FixedUpdate, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(
                (in_state(ClientState::Connected))
                .or(in_state(AssetLoading::SpawnReplicatedEntities).and(in_state(ClientState::Disconnected)))
            )
        )
        .in_set(StatefulSessionSystems),

        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
        HostSystems.run_if(in_state(ClientState::Disconnected)),
        ClientSystems.run_if(not(in_state(ClientState::Disconnected))),

    ))
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (GameplaySystems).run_if(in_state(AssetHotReloadState::Stopped))
    )




    .init_state::<GameSetupScreen>()
    .init_state::<SimulationState>()
    .register_type::<Description>()
    .register_type::<Direction>()
    .register_type::<WeightedSamplerRef>()
    .register_type::<EntityZeroRef>()
    .register_type::<EntityWeightedSampler>()
    
    .replicate::<VisibilityGameState>()    
    .replicate::<Persisted>()

    .replicate_once::<Direction>()
    .replicate::<Directionable>()
    .replicate::<EntityWeightedSampler>()
    .replicate::<Description>()
    .replicate_once::<GlobalTransform>()
    .replicate::<EntityZero>()
    .replicate_filtered_as::<Visibility, VisibilityGameState, (With<EntityZero>, )>()
    .replicate_once_as::<Visibility, VisibilityGameState>()

    .replicate::<EntityZeroRef>()
    ;
}