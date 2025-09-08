use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_common_assets::ron::RonAssetPlugin;
use common::common_states::*;
use bevy_asset_loader::prelude::*;
use bevy_replicon::prelude::*;

use crate::{color_sampler_systems::*, color_sampler_resources::WeightedColorsSeri, game_common_components::*, game_common_components_samplers::{ColorSampler, EntiWeightedSampler, WeightedSamplerRef}, game_common_states::*, game_common_systems::* };

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

    .add_systems(OnEnter(AssetsLoadingState::LocalFinished), (init_color_samplers, add_colorsamplers_to_map).chain().in_set(ColorSamplersInitSystems))

    .add_systems(Update, (
        (z_sort_system, apply_color).in_set(StatefulSessionSystems),
        (toggle_simulation, ).in_set(GameplaySystems),
        (tick_time_based_multipliers).in_set(SimRunningSystems),
        add_colorsamplers_to_map.run_if(not(server_or_singleplayer)),
        apply_color,
    ))
    .configure_sets(Update, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(in_state(LocallyLoadedAssetsSession::KeepAlive))
            .and(
                in_state(AssetsLoadingState::LocalFinished).and(not(server_or_singleplayer))
                .or(in_state(AssetsLoadingState::ReplicatedFinished).and(server_or_singleplayer))
            )
        )
        .in_set(StatefulSessionSystems),
        
        StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
        SimRunningSystems.run_if(in_state(SimulationState::Running)),
        SimPausedSystems.run_if(in_state(SimulationState::Paused)),
    ))
    .configure_sets(FixedUpdate, (
        (ModifierSystems, ).in_set(SimRunningSystems),
        (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
        (GameplaySystems).run_if(
            in_state(GamePhase::ActiveGame)
            .and(in_state(LocallyLoadedAssetsSession::KeepAlive))
            .and(
                in_state(AssetsLoadingState::LocalFinished).and(not(server_or_singleplayer))
                .or(in_state(AssetsLoadingState::ReplicatedFinished).and(server_or_singleplayer))
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
    .register_type::<Category>()
    .register_type::<Categories>()
    .register_type::<EntityZeroRef>()
    
    .replicate::<VisibilityGameState>()    
    .replicate::<FacingDirection>()
    .replicate::<Directionable>()
    .replicate::<EntiWeightedSampler>()
    .replicate::<ColorSampler>()
    .replicate::<MyZ>()
    .replicate::<YSortOrigin>()
    .replicate::<Description>()
    .replicate::<FacingDirection>()
    .replicate::<EntityZeroRef>()
    ;
}