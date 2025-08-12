use bevy::prelude::*;


use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use game_common::game_common::GameplaySystems;
use superstate::superstate_plugin;

use crate::{chunking_components::*, chunking_resources::{ChunkRangeSettings, LoadedChunks}, chunking_systems::*, terrain_gen::{self, terrgen_resources::*, *}, tile::{self, tile_resources::*, *}, tilemap_systems::*};

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
            produce_tilemaps.before(fill_tilemaps_data),
            fill_tilemaps_data,
        ).in_set(ChunkSystems).run_if(in_state(TerrainGenHotLoading::KeepAlive))
    ))
    .configure_sets(Update, (
        (TerrainGenSystems,
        ChunkSystems).in_set(GameplaySystems).run_if(in_state(LoadedAssetsSession::KeepAlive))
    ))
    .configure_sets(
        OnEnter(AssetsLoadingState::LocalFinished), (
            TilingSystems.before(TerrainGenSystems),
        )
    )
    .configure_sets(
        OnEnter(AssetsLoadingState::ReplicatedFinished), (
            TilingSystems.before(TerrainGenSystems),
        )
    )
    .register_type::<ActivatingChunks>()
    .register_type::<ProducedTiles>()
    .register_type::<ChunkPos>()
    .register_type::<GlobalGenSettings>()
    .init_resource::<LoadedChunks>()
    .init_resource::<ChunkRangeSettings>()
    
    ;
}