use bevy::prelude::*;

use crate::game::{tilemap::{terrain_gen::TerrainGenPlugin, tilemap_resources::*, tilemap_systems::*}, SimRunningSystems};

mod tilemap_systems;
pub mod tilemap_components;
pub mod tilemap_resources;

mod terrain_gen;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilemapSystems;

pub struct MyTileMapPlugin;
#[allow(unused_parens)]
impl Plugin for MyTileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((bevy_ecs_tilemap::TilemapPlugin, TerrainGenPlugin))
            .add_systems(Update, (visit_chunks_around_activators, rem_outofrange_chunks_from_activators, despawn_unreferenced_chunks, show_chunks_around_camera, hide_outofrange_chunks, add_tilemaps_to_chunk, ).in_set(SimRunningSystems).in_set(TilemapSystems))
            .init_resource::<LoadedChunks>()
            .init_resource::<ChunkRangeSettings>()
            
        ;
    }
}