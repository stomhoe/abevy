#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use fnl::FastNoiseLite;
use ::tilemap_shared::*;
use crate::{chunking::chunking_components::TerrGenOpsLaunched, terrain_gen::{terrgen_components::*, terrgen_messages::*, terrgen_noise_init_systems::*, terrgen_operaton_list_components::*, terrgen_operation_list_init_systems::*, terrgen_resources::*, terrgen_systems::*},};

pub mod terrgen_systems;
mod terrgen_operation_list_init_systems;
mod terrgen_noise_init_systems;
pub mod terrgen_components;
pub mod terrgen_operaton_list_components;
pub mod terrgen_resources;
pub mod terrgen_messages;



common::define_entity_map_systems!(
    OperationList,
    (),
    OperationList,
    "operation_list",
    "",
    OperationList,
    common::common_components::StrId,
    OpListSeri, "ron/tilemap/terrgen/oplist", "oplist.ron"
);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            (launch_terrain_gen_operations, (process_pending_ops_and_collect_tiles,)).in_set(TerrainGenSystems),
            search_suitable_positions.run_if(in_state(ClientState::Disconnected)),
        ))

        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (
                init_noises,
                map_terrgen_id_to_entity,
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

        .add_plugins((
            plugin_terrgen,
            plugin_operation_list,
        ))
        .replicate::<FnlNoiseComp>()

        .replicate_filtered::<ChildOf, With<OperationList>>()
        .replicate_filtered::<ChildOf, With<FnlNoiseComp>>()
        .replicate_filtered::<ChildOf, With<FailedSearchOplistFilterHolder>>()
        .replicate_filtered::<OplistSize, With<OperationList>>()


        .replicate::<GlobalGenSettings>()

        .add_message::<PendingOp>()
        .add_message::<TerrainProbe>().add_message::<SuitablePosFound>().add_message::<SearchFailed>()

        .register_type::<GlobalGenSettings>()
        .register_type::<FnlNoiseComp>().register_type::<FastNoiseLite>()
        .register_type::<Operand>()
        .register_type::<OplistSize>()
        .register_type::<TerrGenOpsLaunched>()
        .register_type::<ChunkRef>()
        .register_type::<RootInDimensions>()
        .register_type::<OpFilter>()

        ;


}
