use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use ::tilemap_shared::*;
use crate::terrain::terrgen_systems::*;
use crate::terrain::terrgen_cache_systems::init_terrgen_shared_task_data;
use crate::terrain::terrgen_noise_init_systems::*;
use crate::tilemap_systems::process_tiles_pre;

pub mod terrgen_systems;
pub mod terrgen_cache_systems;
mod terrgen_helpers;
mod terrgen_async_fns;
mod terrgen_noise_init_systems;
pub mod biome;
pub mod operation_list;
pub mod terrgen_components;
pub mod terrgen_async_resources;
pub mod terrgen_resources;
pub mod terrgen_seris;
pub mod terrgen_messages;
pub mod terrgen_expression;
pub mod terrprobe;
#[allow(unused_imports)] pub use biome::*;
#[allow(unused_imports)] pub use operation_list::*;
#[allow(unused_imports)] pub use terrgen_async_resources::*;
#[allow(unused_imports)] pub use terrgen_components::*;
#[allow(unused_imports)] pub use terrgen_expression::*;
#[allow(unused_imports)] pub use terrgen_messages::*;
#[allow(unused_imports)] pub use terrgen_resources::*;
#[allow(unused_imports)] pub use terrgen_seris::*;
#[allow(unused_imports)] pub use terrprobe::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, ((
            enqueue_chunk_terrgen_jobs,
            init_terrgen_shared_task_data,
            process_pending_ops_and_collect_tiles.before(process_tiles_pre),//DON'T TOUCH THIS LINE
            ).in_set(TerrainGenSystems),
        ))

        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (
                init_noises,
                map_terrgen_id_to_entity,

            ).chain(),
            ).in_set(TerrainGenSystems)
        )
        .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities),(
            (TerrainGenSystems, TerrainProbeSystems).in_set(TerrainSystems),
            TerrainGenSystems.before(OperationListSystems),
        ))
        .init_resource::<ChunksTerrgenQueue>()
        .init_resource::<TerrAsyncTasks>()
        .init_resource::<TerrGenDebugGrid>()
        .init_resource::<TerrGenDisabledGposByChunk>()

        .add_plugins((
            biome::plugin,
            plugin_terrgen,
            operation_list::plugin,
            terrprobe::opfilter::plugin,
            terrprobe::plugin,
        ))
        .replicate::<FnlNoiseComp>()

        .replicate_filtered::<ChildOf, With<OperationList>>()
        .replicate_filtered::<ChildOf, With<FnlNoiseComp>>()
        .replicate_filtered::<ChildOf, With<FailedSearchOplistFilterHolder>>()
        .replicate_once_filtered::<OplistSize, With<OperationList>>()


        .replicate::<GlobalGenSettings>()

        .add_message::<PendingOp>()
        .add_message::<ChunkTerrainBuilt>()


        ;


}
