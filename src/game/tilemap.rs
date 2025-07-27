use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use superstate::superstate_plugin;

use crate::game::{tilemap::{chunking_components::*, chunking_resources::*, chunking_systems::*, terrain_gen::TerrainGenPlugin, tile::TilePlugin, tilemap_systems::*}, ActiveGameSystems, SimRunningSystems};

mod tilemap_systems;
mod chunking_systems;
pub mod chunking_components;
pub mod chunking_resources;

pub mod tile;

pub mod terrain_gen;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunksSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilemapsSystems;

pub struct MyTileMapPlugin;
#[allow(unused_parens)]
impl Plugin for MyTileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                bevy_ecs_tilemap::TilemapPlugin, 
                TerrainGenPlugin,
                TilePlugin,
                superstate_plugin::<ChunkInitState, (UninitializedChunk, TilesReady, LayersReady, InitializedChunk)>
            ))

            .add_systems(FixedUpdate, (
                add_initialized_chunks_to_loaded_chunks,
                visit_chunks_around_activators, 
                rem_outofrange_chunks_from_activators, 
                despawn_unreferenced_chunks, 
                show_chunks_around_camera, 
                hide_outofrange_chunks, 

            ).in_set(SimRunningSystems).in_set(ChunksSystems))
            .add_systems(FixedUpdate, (
                produce_tilemaps.before(fill_tilemaps_data),
                fill_tilemaps_data,
                
            ).in_set(SimRunningSystems).in_set(TilemapsSystems))
            .init_resource::<LoadedChunks>()
            .init_resource::<ChunkRangeSettings>()
            .replicate_once::<ActivatesChunks>()
        ;
    }
}