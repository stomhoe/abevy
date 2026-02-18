#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use ::tilemap_shared::*;
use crate::terrain::{
        operation_list::{
            operation_list_components::*,
            operation_list_init_systems::*,
            operation_list_resources::*,
        },
    terrgen_components::*,
    terrgen_messages::PendingOp,
    terrgen_noise_init_systems::*,
    terrgen_resources::*,
    terrgen_systems::*,
};

pub mod terrgen_systems;
mod terrgen_noise_init_systems;
pub mod operation_list;
pub mod terrgen_components;
pub mod terrgen_resources;
pub mod terrgen_messages;
pub mod terrgen_expression;
pub mod terrgen_search;
pub mod opfilter;
pub mod terrprobe;
pub use operation_list::operation_list_components;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, launch_terrain_operations.in_set(TerrainGenSystems))
        .add_systems(Update, process_pending_ops_and_collect_tiles.in_set(TerrainGenSystems))
        .add_systems(Update, search_suitable_positions.run_if(in_state(ClientState::Disconnected)))

        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (
                init_noises,
                map_terrgen_id_to_entity,
                cache_tg_oplists,
                init_oplists_from_assets,
                map_operation_list_id_to_entity,
                init_oplists_bifurcations,
                cycle_detection,
                assign_rootoplist_to_dimensions,

            ).chain(),
            ).in_set(TerrainGenSystems)
        )

        .init_resource::<TerrGenLaunchQueue>()
        .init_resource::<TerrGenAsyncTasks>()
        .init_resource::<TerrGenDebugGrid>()

        .add_plugins((
            plugin_terrgen,
            operation_list::plugin,
            opfilter::plugin,
            terrprobe::plugin,
        ))
        .replicate::<FnlNoiseComp>()

        .replicate_filtered::<ChildOf, With<OperationList>>()
        .replicate_filtered::<ChildOf, With<FnlNoiseComp>>()
        .replicate_filtered::<ChildOf, With<FailedSearchOplistFilterHolder>>()
        .replicate_filtered::<OplistSize, With<OperationList>>()


        .replicate::<GlobalGenSettings>()
        .replicate::<terrgen_search::AwaitingStartSearch>()

        .add_message::<PendingOp>()


        ;


}
