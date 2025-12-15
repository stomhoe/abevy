use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use common::{common_components::ImagePathHolder, common_states::*, common_types::*};
use bevy_replicon::prelude::*;

use crate::{color_sampler_systems::*, color_sampler_resources::WeightedColorsSeri, game_common_components::*, game_common_components_samplers::{ColorSampler, EntityWeightedSampler, WeightedSamplerRef}, game_common_states::*, game_common_systems::* };

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
pub struct ColorSamplersInitSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {

    app
    .add_plugins((
        RonAssetPlugin::<WeightedColorsSeri>::new(&["wcolors.ron"]),

    ))
    .add_systems(OnEnter(AppState::NoSession), (reset_states))

    .add_systems(OnEnter(AssetsLoadingState::ReplicatedFinished), (init_color_samplers, ).chain().in_set(ColorSamplersInitSystems))

    .add_systems(Update, (
        (z_sort_system, apply_pos_sampled_color).in_set(StatefulSessionSystems),
        (toggle_simulation, ).in_set(GameplaySystems),
        (tick_time_based_multipliers).in_set(SimRunningSystems),
        add_colorsamplers_to_map,
        apply_pos_sampled_color,
        clone_ezero_children_ents,
        disable_ezeros,
    ))
    .configure_sets(Update, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(in_state(LocallyLoadedAssetsSession::KeepAlive))
            .and(
                in_state(AssetsLoadingState::LocalFinished).and(not(in_state(ClientState::Disconnected)))
                .or(in_state(AssetsLoadingState::ReplicatedFinished).and(in_state(ClientState::Disconnected)))
            )
        ).in_set(StatefulSessionSystems),
        
        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
    ))
    .configure_sets(FixedUpdate, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(in_state(LocallyLoadedAssetsSession::KeepAlive))
            .and(
                in_state(AssetsLoadingState::LocalFinished).and(not(in_state(ClientState::Disconnected)))
                .or(in_state(AssetsLoadingState::ReplicatedFinished).and(in_state(ClientState::Disconnected)))
            )
        )
        .in_set(StatefulSessionSystems),

        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
    ))


    .init_state::<GameSetupScreen>()
    .init_state::<SimulationState>()
    .register_type::<MyZ>()
    .register_type::<YSortOrigin>()
    .register_type::<Description>()
    .register_type::<FacingDirection>()
    .register_type::<WeightedSamplerRef>()
    .register_type::<Categories>()
    .register_type::<EntityZeroRef>()
    .register_type::<EntityWeightedSampler>()
    .register_type::<ColorSampler>()
    
    .replicate::<VisibilityGameState>()    
    .replicate::<Persisted>()

    .replicate_once::<FacingDirection>()
    .replicate::<Directionable>()
    .replicate::<EntityWeightedSampler>()
    .replicate::<ColorSampler>()
    .replicate::<MyZ>()
    .replicate::<YSortOrigin>()
    .replicate::<Description>()
    .replicate_once::<GlobalTransform>()
    .replicate::<Categories>()
    .replicate_filtered::<EntityZero, Or<(With<Disabled>, Without<Disabled>)>>()
    .replicate_filtered_as::<Visibility, VisibilityGameState, (With<EntityZero>, Or<(With<Disabled>, Without<Disabled>)>)>()

    .replicate::<EntityZeroRef>()
    ;
}