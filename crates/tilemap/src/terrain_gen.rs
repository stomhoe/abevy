#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use common::states::ReplicatedAssetsLoadingState;
use fastnoise_lite::FastNoiseLite;

use crate::{{chunking_components::ProducedTiles, terrain_gen::{terrain_materials::MonoRepeatTextureOverlayMat, terrgen_components::*, terrgen_init_systems::*, terrgen_resources::*, terrgen_systems::*}}, };

pub mod terrgen_systems;
mod terrgen_init_systems;
pub mod terrain_materials;
pub mod terrgen_components;
pub mod terrgen_resources;
pub mod terrgen_utils;



#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

pub struct TerrainGenPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TerrainGenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                (spawn_terrain_operations, produce_tiles).in_set(TerrainGenSystems),
                (add_noises_to_map, add_oplists_to_map, )
            ))
        
            .add_systems(OnEnter(ReplicatedAssetsLoadingState::Finished), (
                init_noises.before(add_noises_to_map),   
                add_noises_to_map.before(init_oplists_from_assets),
                init_oplists_from_assets.before(add_oplists_to_map),
                add_oplists_to_map.before(init_oplists_bifurcations),
                init_oplists_bifurcations,
            ).in_set(TerrainGenSystems))

            .init_resource::<GlobalGenSettings>()
            // .init_resource::<TerrGenEntityMap>()
            // .init_resource::<OpListEntityMap>()

            .add_plugins((
                MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default(),
                RonAssetPlugin::<NoiseSerialization>::new(&["noise.ron"]),
                RonAssetPlugin::<OpListSerialization>::new(&["oplist.ron"]),

            ))
            .register_type::<FnlNoise>()
            .register_type::<FastNoiseLite>()
            .register_type::<OperationList>()
            .register_type::<Operand>()
            .register_type::<Operation>()

            .replicate::<FnlNoise>()
            .replicate::<RootOpList>()
            .replicate_with((
                RuleFns::<ProducedTiles>::default(),
                (RuleFns::<OperationList>::default(), SendRate::Once),
            ))

        ;
    }
}





