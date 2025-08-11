use bevy::prelude::*;


use bevy_asset_loader::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::{LocalAssetsLoadingState, ReplicatedAssetsLoadingState};
use game_common::game_common::GameplaySystems;
use superstate::superstate_plugin;

use crate::{chunking_components::*, chunking_resources::{ChunkRangeSettings, LoadedChunks}, chunking_systems::*, terrain_gen::{self, terrgen_resources::*, *}, tile::{tile_resources::*, *}, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        bevy_ecs_tilemap::TilemapPlugin, 
        terrain_gen::plugin,
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
            despawn_all_chunks,
        ).in_set(ChunkSystems)
    ))
    .configure_sets(Update, (
        TerrainGenSystems.in_set(ChunkSystems),
        ChunkSystems.in_set(GameplaySystems)
    ))
    .configure_sets(
        OnEnter(LocalAssetsLoadingState::Finished), (
            TilingSystems.before(TerrainGenSystems),
        )
    )
    .configure_sets(
        OnEnter(ReplicatedAssetsLoadingState::Finished), (
            TilingSystems.before(TerrainGenSystems),
        )
    )
    .register_type::<ActivatingChunks>()
    .register_type::<ProducedTiles>()
    .register_type::<ChunkPos>()
    .register_type::<GlobalGenSettings>()
    .init_resource::<LoadedChunks>()
    .init_resource::<ChunkRangeSettings>()
    

    .configure_loading_state(
        LoadingStateConfig::new(ReplicatedAssetsLoadingState::InProcess)
        .load_collection::<NoiseSerisHandles>()
        .load_collection::<OpListSerisHandles>()
        .load_collection::<TileWeightedSamplerSerisHandles>()
        .finally_init_resource::<TerrGenEntityMap>()
        .finally_init_resource::<OpListEntityMap>()
    )

    .configure_loading_state(
        LoadingStateConfig::new(LocalAssetsLoadingState::InProcess)
        .load_collection::<ShaderRepeatTexSerisHandles>()
        .load_collection::<TileSerisHandles>()
        .finally_init_resource::<TilingEntityMap>()
        .finally_init_resource::<TileShaderEntityMap>()
    )

    ;
}