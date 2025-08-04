use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use superstate::superstate_plugin;

use crate::game::{tilemap::{chunking_components::*, chunking_resources::*, chunking_systems::*, terrain_gen::*, tile::*, tilemap_systems::*}, ActiveGameSystems, AssetLoadingState, SimRunningSystems};

mod tilemap_systems;
mod chunking_systems;
pub mod chunking_components;
pub mod chunking_resources;


pub mod tile;

pub mod terrain_gen;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;

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

            .add_systems(Update, (
                (
                    visit_chunks_around_activators, 
                    rem_outofrange_chunks_from_activators, 
                    despawn_unreferenced_chunks, 
                    show_chunks_around_camera, 
                    hide_outofrange_chunks, 
                    produce_tilemaps.before(fill_tilemaps_data),
                    fill_tilemaps_data,
                ).in_set(ChunkSystems)
            ))
            .configure_sets(Update, (
                TerrainGenSystems.in_set(ChunkSystems),
                TilingInitSystems.before(TerrainGenInitSystems),
            ))
            .configure_sets(
                OnEnter(AssetLoadingState::Complete), (
                    TilingInitSystems.before(TerrainGenInitSystems),
                )
            )
           
            .init_resource::<LoadedChunks>()
            .init_resource::<ChunkRangeSettings>()
            .replicate_once::<ActivatesChunks>()
        ;
    }
}