#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use dimension_shared::RootInDimensions;
use fnl::FastNoiseLite;
use ::tilemap_shared::*;
use crate::{chunking_components::OperationsLaunched, terrain_gen::{terrgen_components::*, terrgen_messages::*, terrgen_noise_init_systems::*, terrgen_oplist_components::*, terrgen_oplist_init_systems::*, terrgen_resources::*, terrgen_systems::*}, tilemap_systems::process_tiles_pre,};

pub mod terrgen_systems;
mod terrgen_oplist_init_systems;
mod terrgen_noise_init_systems;
pub mod terrgen_components;
pub mod terrgen_oplist_components;
pub mod terrgen_resources;
pub mod terrgen_messages;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            (spawn_terrain_operations, (process_pending_ops_and_collect_tiles,).before(process_tiles_pre)).in_set(TerrainGenSystems),
            search_suitable_positions.run_if(in_state(ClientState::Disconnected)),
            oplist_init_dim_refs,
            set_hashed_tags_for_oplist,
        ))
        
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (
                init_noises,
                init_oplists_from_assets,
                init_oplists_bifurcations,
                cycle_detection,
            
            ).chain(),
            ).in_set(TerrainGenSystems)
        )

        .register_type::<AcGlobalGenSettings>()
        .init_resource::<RegisteredPositions>()
        

        .add_plugins((
            RonAssetPlugin::<NoiseSerialization>::new(&["fnl.ron"]),
            RonAssetPlugin::<OpListSerialization>::new(&["oplist.ron"]),

        ))
        .register_type::<NoiseSerisHandles>().register_type::<NoiseSerialization>()
        .register_type::<OpListSerisHandles>().register_type::<OpListSerialization>()
        .register_type::<FnlNoiseComp>().register_type::<FastNoiseLite>()
        .register_type::<OperationList>().register_type::<Operation>()
        .register_type::<Operand>()
        .register_type::<TerrGenEntityMap>()
        .register_type::<OpListEntityMap>()
        .register_type::<OplistSize>()
        .register_type::<OperationsLaunched>()
        .register_type::<ChunkRef>()
        .register_type::<RegisteredPositions>()
        .register_type::<RootInDimensions>()
        .register_type::<MassCollectedTiles>()
        .register_type::<OpFilter>()
        .register_type::<TileHelperStruct>()

        .add_server_event::<RegisteredPositions>(Channel::Unordered)
        .make_event_independent::<RegisteredPositions>()
        
        .replicate_once::<(OplistSize)>()//LO USAN LAS TILE INSTANCES DE TILEMAP, NO BORRAR
        
        .replicate::<OperationList>()
        .replicate::<FnlNoiseComp>()

        .replicate_filtered::<ChildOf, With<OperationList>>()
        .replicate_filtered::<ChildOf, With<FnlNoiseComp>>()
        .replicate_filtered::<OplistSize, With<OperationList>>()
        
        .replicate::<NoiseHolder>()
        .replicate::<AcGlobalGenSettings>()

        .add_message::<PendingOp>()
        .init_resource::<MassCollectedTiles>()
        .add_message::<TerrainProbe>().add_message::<SuitablePosFound>().add_message::<SearchFailed>()
        ;

        
}




