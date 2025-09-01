#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetsLoadingState;
use dimension::dimension_components::MultipleDimensionRefs;
use fnl::FastNoiseLite;
use ::tilemap_shared::*;
use crate::{chunking_components::{PendingOps,}, terrain_gen::{terrgen_components::*, terrgen_noise_init_systems::*, terrgen_oplist_components::*, terrgen_oplist_init_systems::*, terrgen_resources::*, terrgen_systems::*, terrgen_events::*}, tilemap_systems::produce_tilemaps, };

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
            (spawn_terrain_operations, (produce_tiles, process_tiles)).in_set(TerrainGenSystems),
            (add_noises_to_map, add_oplists_to_map, client_remap_operation_entities).run_if(not(server_or_singleplayer)),
        ))
    
        .add_systems(
            OnEnter(AssetsLoadingState::ReplicatedFinished), (
            (
                init_noises,
                add_noises_to_map,
                init_oplists_from_assets,
                add_oplists_to_map,
                init_oplists_bifurcations,
            ).chain(),
        
        ).in_set(TerrainGenSystems)
        )

        .init_resource::<AaGlobalGenSettings>().register_type::<AaGlobalGenSettings>()
        .init_resource::<RegisteredPositions>()
        

        .add_plugins((
            RonAssetPlugin::<NoiseSerialization>::new(&["fnl.ron"]),

            RonAssetPlugin::<OpListSerialization>::new(&["oplist.ron"]),

        ))
        
        .register_type::<NoiseSerisHandles>()
        .register_type::<NoiseSerialization>()
        .register_type::<OpListSerisHandles>()
        .register_type::<OpListSerialization>()
        .register_type::<FnlNoise>()
        .register_type::<FastNoiseLite>()
        .register_type::<OperationList>().register_type::<Operation>().register_type::<Operand>()
        .register_type::<TerrGenEntityMap>()
        .register_type::<OpListEntityMap>()
        .register_type::<OplistSize>()
        .register_type::<PendingOps>()
        .register_type::<ChunkRef>()
        .register_type::<RegisteredPositions>()

        .add_server_trigger::<RegisteredPositions>(Channel::Unordered)
        .make_trigger_independent::<RegisteredPositions>()
        
        .add_server_trigger::<NewlyRegPos>(Channel::Unordered)
        .make_trigger_independent::<NewlyRegPos>()
        .add_observer(sync_register_new_pos)

        .replicate::<OplistSize>().replicate::<FnlNoise>()
        .replicate::<OperationList>().replicate_bundle::<(OperationList, ChildOf)>()
        .add_event::<PendingOp>()
        .add_event::<InstantiatedTiles>()
        .add_event::<ProcessedTiles>()
        .init_resource::<Events<PendingOp>>()
        ;

        
}




