#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetsLoadingState;
use dimension_shared::RootInDimensions;
use fnl::FastNoiseLite;
use ::tilemap_shared::*;
use crate::{chunking_components::OperationsLaunched, terrain_gen::{terrgen_components::*, terrgen_events::*, terrgen_noise_init_systems::*, terrgen_oplist_components::*, terrgen_oplist_init_systems::*, terrgen_resources::*, terrgen_systems::*}, tile::tile_components::TileSamplerHolder, tilemap_systems::process_tiles_pre,};

pub mod terrgen_systems;
mod terrgen_oplist_init_systems;
mod terrgen_noise_init_systems;
pub mod terrgen_components;
pub mod terrgen_oplist_components;
pub mod terrgen_resources;
pub mod terrgen_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            (spawn_terrain_operations, (produce_tiles,).before(process_tiles_pre)).in_set(TerrainGenSystems),
            search_suitable_position.run_if(server_or_singleplayer),
            (add_noises_to_map, add_oplists_to_map, client_remap_operation_entities, ).run_if(not(server_or_singleplayer)),
            oplist_init_dim_refs,
        ))
        
        .add_systems(
            OnEnter(AssetsLoadingState::ReplicatedFinished), (
                (
                    init_noises,
                    add_noises_to_map,
                    init_oplists_from_assets,
                    add_oplists_to_map,
                    init_oplists_bifurcations,
                    cycle_detection,
                
            ).chain(),
        
        ).in_set(TerrainGenSystems)
        )

        .init_resource::<AaGlobalGenSettings>().register_type::<AaGlobalGenSettings>()
        .init_resource::<RegisteredPositions>()
        

        .add_plugins((
            RonAssetPlugin::<NoiseSerialization>::new(&["fnl.ron"]),

            RonAssetPlugin::<OpListSerialization>::new(&["oplist.ron"]),

        ))
        
        .register_type::<NoiseSerisHandles>().register_type::<NoiseSerialization>()
        .register_type::<OpListSerisHandles>().register_type::<OpListSerialization>()
        .register_type::<FnlNoise>().register_type::<FastNoiseLite>()
        .register_type::<OperationList>().register_type::<Operation>().register_type::<Operand>()
        .register_type::<TerrGenEntityMap>()
        .register_type::<OpListEntityMap>()
        .register_type::<OplistSize>()
        .register_type::<OperationsLaunched>()
        .register_type::<ChunkRef>()
        .register_type::<RegisteredPositions>()
        .register_type::<RootInDimensions>()
        .register_type::<MassCollectedTiles>()
        .register_type::<TileHelperStruct>()

        .add_server_trigger::<RegisteredPositions>(Channel::Unordered)
        .make_trigger_independent::<RegisteredPositions>()
        
        

        .replicate_bundle::<(FnlNoise, ChildOf)>()
        .replicate_bundle::<(OperationList, OplistSize)>()
        .replicate_once::<(OplistSize)>()//LO USAN LAS TILE INSTANCES DE TILEMAP, NO BORRAR
        .replicate::<OperationList>().replicate_bundle::<(OperationList, ChildOf)>()
        .replicate::<NoiseHolder>()
        .add_message::<PendingOp>()
        //.add_message::<MassCollectedTiles>()
        .init_resource::<MassCollectedTiles>()
        .add_message::<PosSearch>().add_message::<SuitablePosFound>().add_message::<SearchFailed>()
        ;

        
}




