use bevy::prelude::*;


use bevy_replicon::prelude::AppRuleExt;
use common::states::{LocalAssetsLoadingState, ReplicatedAssetsLoadingState};
use superstate::superstate_plugin;

use crate::{chunking_components::*, chunking_resources::{ChunkRangeSettings, LoadedChunks}, chunking_systems::*, terrain_gen::{terrgen_resources::GlobalGenSettings, *}, tile::*, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
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

    ;
}