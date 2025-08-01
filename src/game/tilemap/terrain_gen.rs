#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{tilemap::{terrain_gen::{terrgen_resources::*, terrgen_systems::*, terrain_materials::MonoRepeatTextureOverlayMat}, tile::ImageSizeSetState}, SimRunningSystems};

pub mod terrgen_systems;
pub mod terrain_materials;
pub mod terrgen_components;
pub mod terrgen_resources;
pub mod terrgen_utils;
pub mod terrgen_events;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

pub struct TerrainGenPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TerrainGenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (add_tiles2spawn_within_chunk, produce_tiles).in_set(TerrainGenSystems).in_set(SimRunningSystems))
            .add_systems(Startup, (setup, ))
            .init_resource::<WorldGenSettings>()

            .add_plugins(MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default())

            .configure_sets(Update, TerrainGenSystems.run_if(in_state(ImageSizeSetState::Done)))
        ;
    }
}





