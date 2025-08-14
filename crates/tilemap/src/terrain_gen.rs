#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetsLoadingState;
use fastnoise_lite::FastNoiseLite;

use crate::{chunking_components::{PendingOperations, ProducedTiles}, terrain_gen::{terrgen_components::*, terrgen_init_systems::*, terrgen_resources::*, terrgen_systems::*}, };

pub mod terrgen_systems;
mod terrgen_init_systems;
pub mod terrgen_components;
pub mod terrgen_resources;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            (spawn_terrain_operations, produce_tiles).in_set(TerrainGenSystems),
            (add_noises_to_map, add_oplists_to_map, ).run_if(not(server_or_singleplayer))
        ))
    
        .add_systems(
            OnEnter(AssetsLoadingState::ReplicatedFinished), (
            (
                init_noises,
                add_noises_to_map,
                init_oplists_from_assets,
                add_oplists_to_map,
                init_oplists_bifurcations,
            )
            .chain(),).in_set(TerrainGenSystems)
        )

        .init_resource::<GlobalGenSettings>()
        .register_type::<GlobalGenSettings>()
 

        .add_plugins((
            RonAssetPlugin::<NoiseSerialization>::new(&["noise.ron"]),
            RonAssetPlugin::<OpListSerialization>::new(&["oplist.ron"]),

        ))
        
        .register_type::<NoiseSerisHandles>()
        .register_type::<NoiseSerialization>()
        .register_type::<OpListSerisHandles>()
        .register_type::<OpListSerialization>()
        .register_type::<FnlNoise>()
        .register_type::<FastNoiseLite>()
        .register_type::<OperationList>()
        .register_type::<Operand>()
        .register_type::<Operation>()
        .register_type::<TerrGenEntityMap>()
        .register_type::<OpListEntityMap>()
        .register_type::<OplistSize>()
        .register_type::<PendingOperations>()
        .replicate::<ProducedTiles>()
        
        .replicate::<OplistSize>()
        .replicate::<FnlNoise>()
        .replicate_with((
            RuleFns::<ProducedTiles>::default(),
            (RuleFns::<OperationList>::default(), SendRate::Once),
        ))
      
    ;
}





