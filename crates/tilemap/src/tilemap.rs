use bevy::prelude::*;


use common::common_states::*;
use dimension::DimensionSystems;
use game_common::game_common::GameplaySystems;
use superstate::superstate_plugin;
use tilemap_shared::ChunkPos;

use crate::{chunking_components::*, chunking_resources::*, chunking_systems::*, terrain_gen::{self, *}, tile::{self, *}, tilemap_components::TmapHashIdtoTextureIndex, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        bevy_ecs_tilemap::TilemapPlugin, 
        terrain_gen::plugin,
        tile::plugin,
        superstate_plugin::<ChunkInitState, (UninitializedChunk, TilesReady, LayersReady, InitializedChunk)>
    ))

    .add_systems(Update, (
        despawn_unreferenced_chunks, 
        rem_outofrange_chunks_from_activators, 
        (
            visit_chunks_around_activators, 
            show_chunks_around_camera, 
            hide_outofrange_chunks, 
            (produce_tilemaps, fill_tilemaps_data,).chain()
        ).in_set(ChunkSystems).run_if(in_state(TerrainGenHotLoading::KeepAlive))
    ))
    .configure_sets(Update, (
        (TerrainGenSystems,
        ChunkSystems).in_set(GameplaySystems).run_if(in_state(LoadedAssetsSession::KeepAlive))
    ))
    .configure_sets(
        OnEnter(AssetsLoadingState::LocalFinished), (
            TilingSystems.before(TerrainGenSystems),
            DimensionSystems.before(TerrainGenSystems),
        )
    )
    .configure_sets(
        OnEnter(AssetsLoadingState::ReplicatedFinished), (
            TilingSystems.before(TerrainGenSystems),
            DimensionSystems.before(TerrainGenSystems),
        )
    )
    .register_type::<ActivatingChunks>()
    .register_type::<ProducedTiles>()
    .register_type::<ChunkPos>()
    .register_type::<AaChunkRangeSettings>()
    .register_type::<TmapHashIdtoTextureIndex>()
    .init_resource::<LoadedChunks>()
    .init_resource::<AaChunkRangeSettings>()
    
    ;
}