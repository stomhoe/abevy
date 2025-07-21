use bevy::prelude::*;
use superstate::superstate_plugin;

use crate::game::{tilemap::{chunking_resources::*, chunking_systems::*, terrain_gen::TerrainGenPlugin, tile_imgs::*, chunking_components::*, tilemap_systems::*}, IngameSystems, SimRunningSystems};

mod tilemap_systems;
mod chunking_systems;
pub mod chunking_components;
pub mod chunking_resources;
pub mod tile_nids;

pub mod tile_imgs;

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
            .init_resource::<NidImgMap>()
            .add_systems(Startup, (tile_imgs::setup_nid_img_map,))
        ;
    }
}