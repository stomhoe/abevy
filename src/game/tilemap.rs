use bevy::prelude::*;

use crate::game::{tilemap::{formation_generation::FormationGenerationPlugin, tilemap_resources::*, tilemap_systems::*}, SimRunningSystems};

mod tilemap_systems;
pub mod tilemap_components;
pub mod tilemap_resources;

mod formation_generation;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilemapSystems;

pub struct MyTileMapPlugin;
#[allow(unused_parens)]
impl Plugin for MyTileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((bevy_ecs_tilemap::TilemapPlugin, FormationGenerationPlugin))
            .add_systems(Update, (visit_chunks_around_activators, rem_outofrange_chunks_from_activators, despawn_unreferenced_chunks, show_chunks_around_camera, hide_outofrange_chunks, ).in_set(SimRunningSystems).in_set(TilemapSystems))
            .init_resource::<LoadedChunks>()
            .init_resource::<ChunkRangeSettings>()
            
        ;
    }
}