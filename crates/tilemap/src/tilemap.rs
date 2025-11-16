use bevy::prelude::*;


use common::common_states::*;
use dimension_shared::DimensionSystems;
use game_common::game_common::GameplaySystems;
use tilemap_shared::ChunkPos;

use crate::{chunking_components::*, chunking_resources::*, chunking_systems::*, terrain_gen::{self,  *}, tile::{self, *}, tilemap_components::TmapHashIdtoTextureIndex, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        bevy_ecs_tilemap::TilemapPlugin, 
        terrain_gen::plugin,
        tile::plugin,
    ))

    .add_systems(Update, (
        clear_chunks_on_dim_change,
        (rem_outofrange_chunks_from_activators, despawn_unreferenced_chunks), 
        tile_assign_child_of,
        (
            visit_chunks_around_activators, 
            show_or_hide_chunks, 
            process_tiles_pre.before(despawn_unreferenced_chunks)//NO TOCAR
        ).in_set(ChunkSystems).run_if(in_state(TerrainGenHotLoading::KeepAlive))
    ))

    .configure_sets(Update, (
        (TerrainGenSystems,
        ChunkSystems).in_set(GameplaySystems).run_if(in_state(LocallyLoadedAssetsSession::KeepAlive))
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
    .register_type::<LayersMap>()
    .register_type::<LoadedChunks>()
    .register_type::<ActivatingChunks>()
    .register_type::<ChunkPos>()
    .register_type::<AaChunkRangeSettings>()
    .register_type::<TmapHashIdtoTextureIndex>()
    .init_resource::<LoadedChunks>()
    .init_resource::<AaChunkRangeSettings>()
    .add_message::<CheckChunkDespawn>()
    

;}