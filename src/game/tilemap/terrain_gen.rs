#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{tilemap::terrain_gen::{terrain_materials::MonoRepeatTextureOverlayMat, terrgen_resources::*, terrgen_systems::*}, AssetLoadingState, SimRunningSystems};

pub mod terrgen_systems;
pub mod terrain_materials;
pub mod terrgen_components;
pub mod terrgen_resources;
pub mod terrgen_utils;
pub mod terrgen_events;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenInitSystems;

pub struct TerrainGenPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TerrainGenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (spawn_terrain_operations, produce_tiles).in_set(TerrainGenSystems))
            .add_systems(Startup, (setup, ))
            .add_systems(OnEnter(AssetLoadingState::Complete), (
                init_noises.before(init_oplists),
                init_oplists,
            ).in_set(TerrainGenInitSystems))

            .init_resource::<WorldGenSettings>()
            .init_resource::<TerrGenEntityMap>()
            .init_resource::<OpListEntityMap>()

            .add_plugins((
                MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default(),
                RonAssetPlugin::<NoiseSerialization>::new(&["noise.ron"]),
                RonAssetPlugin::<OpListSeri>::new(&["oplist.ron"]),

            ))

        ;
    }
}





