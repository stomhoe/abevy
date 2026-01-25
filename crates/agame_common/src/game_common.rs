use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use common::{common_components::{AnyDisabling, ImagePathHolder}, common_states::*, common_types::*};
use bevy_replicon::prelude::*;

use crate::{game_common_components::*, game_common_components_samplers::*, game_common_states::*, game_common_systems::* };

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



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {

    app
    
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (reset_states))

    .add_systems(Update, (
        (toggle_simulation, ).in_set(GameplaySystems),
        (tick_time_based_multipliers).in_set(SimRunningSystems),
        clone_ezero_children_ents,
        //disable_ezeros,
        delete_sprites_without_childof,
    ))
    .configure_sets(Update, (
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
        (ModifierSystems, ).in_set(SimRunningSystems),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(
                in_state(AssetLoading::LoadingReplicatedCollections).and(in_state(ClientState::Connected))
                .or(in_state(AssetLoading::SpawnReplicatedEntities).and(in_state(ClientState::Disconnected)))
            )
        ).in_set(StatefulSessionSystems),
        
        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
    ))
    .configure_sets(FixedUpdate, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(
                in_state(AssetLoading::LoadingReplicatedCollections).and(in_state(ClientState::Connected))
                .or(in_state(AssetLoading::SpawnReplicatedEntities).and(in_state(ClientState::Disconnected)))
            )
        )
        .in_set(StatefulSessionSystems),

        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
    ))




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
    .replicate_filtered_as::<Visibility, VisibilityGameState, (With<EntityZero>, AnyDisabling)>()
    .replicate_once_filtered_as::<Visibility, VisibilityGameState, (AnyDisabling)>()

    .replicate::<EntityZeroRef>()
    ;
}