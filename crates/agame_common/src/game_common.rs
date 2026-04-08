use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_replicon::prelude::*;
#[allow(unused_imports, )]
use common::{common_states::*};
use tilemap_shared::tilemap_shared_samplers::*;

use crate::{
    game_common_components::*, game_common_states::*,
    game_common_string_components::Description, game_common_systems::*, game_common_timers::{TimedOut, tick_timers},
};

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
pub struct AcClientSystems;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (tick_time_based_multipliers).in_set(SimRunningSystems),
            clone_templ_children_ents,
            tick_timers,
            despawn_sprites_without_childof,
        ),
    );
    //#[cfg(debug_assertions)]
    {
        app.add_systems(Update, set_entity_name);
    }

    app.configure_sets(
        Update,
        (
            SimPausedSystems.run_if(in_state(SimulationState::Paused)),
            (ModifierSystems,).in_set(SimRunningSystems),
            SimRunningSystems.run_if(in_state(SimulationState::Running)),
            (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
            (GameplaySystems)
                .run_if(
                    in_state(GamePhase::ActiveGame).and(
                        (in_state(ClientState::Connected))
                            .or(in_state(AssetLoading::SpawnReplicatedEntities)
                                .and(in_state(ClientState::Disconnected))),
                    ),
                )
                .in_set(StatefulSessionSystems),
            StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
            HostSystems.run_if(in_state(ClientState::Disconnected)),
            AcClientSystems.run_if(not(in_state(ClientState::Disconnected))),
        ),
    )
    .configure_sets(
        FixedUpdate,
        (
            (ModifierSystems,).in_set(SimRunningSystems),
            (SimRunningSystems, SimPausedSystems).in_set(GameplaySystems),
            (GameplaySystems)
                .run_if(
                    in_state(GamePhase::ActiveGame).and(
                        (in_state(ClientState::Connected))
                            .or(in_state(AssetLoading::SpawnReplicatedEntities)
                                .and(in_state(ClientState::Disconnected))),
                    ),
                )
                .in_set(StatefulSessionSystems),
            StatefulSessionSystems.run_if(in_state(AppState::StatefulGameSession)),
            SimRunningSystems.run_if(in_state(SimulationState::Running)),
            SimPausedSystems.run_if(in_state(SimulationState::Paused)),
            HostSystems.run_if(in_state(ClientState::Disconnected)),
            AcClientSystems.run_if(not(in_state(ClientState::Disconnected))),
        ),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (GameplaySystems).run_if(in_state(AssetHotReloadState::Stopped)),
    )
    .init_state::<GameSetupScreen>()
    .init_state::<SimulationState>()
    .replicate::<Description>()
    .replicate::<Templ>()
    .replicate::<TemplEntiRef>()
    .replicate::<TemplEntiHashIdRef>()

    .replicate::<Directionable>()
    //.replicate::<EntityCountMapWeightedSampler>()
    .replicate::<Persisted>()
    .replicate::<Dead>()
    .replicate::<DespawnOnDeath>()
    .replicate::<ScaleHpAndStrengthWithSampledSize>()
    .replicate_filtered::<ChildOf, Without<TilemapId>>()
    .replicate::<EntityWeightedSampler>()
    .replicate::<CloneTemplChildren>()

    .replicate_once::<GlobalTransform>()
    .replicate_once::<Transform>()
    .replicate_filtered_as::<Visibility, common::common_components::VisibilityGameState, (With<Templ>,)>()
    .replicate_once_as::<Visibility, common::common_components::VisibilityGameState>()
    .add_message::<TimedOut>()

    .add_plugins((
        plugin_sprite_vert_normal_dist,
        plugin_sprite_hori_normal_dist,
        plugin_sprite_global_normal_dist,
    ))
    ;
}
