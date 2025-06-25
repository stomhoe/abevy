#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{tilemap::{terrain_gen::{terrain_gen_components::FnlComp, terrain_gen_resources::*, terrain_gen_systems::*, terrain_gen_utils::*, terrain_materials::TextureOverlayMaterial}, tile_imgs::{NidImgMap, NidRepeatImgMap, IMG_WHITE}, tilemap_resources::*}, GamePhase, IngameSystems};

mod terrain_gen_systems;
pub mod terrain_materials;
pub mod terrain_gen_components;
pub mod terrain_gen_resources;
pub mod terrain_gen_utils;
//mod terrain_generation_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

pub struct TerrainGenPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TerrainGenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (add_tiles2spawn_within_chunk, ).in_set(TerrainGenSystems).in_set(IngameSystems))
            .add_systems(OnEnter(GamePhase::InGame), (setup, ))
            .init_resource::<WorldGenSettings>()

            .add_plugins(MaterialTilemapPlugin::<TextureOverlayMaterial>::default())
        ;
    }
}





