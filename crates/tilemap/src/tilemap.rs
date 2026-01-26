use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use dimension_shared::DimensionSystems;
use game_common::game_common::GameplaySystems;
use ::tilemap_shared::*;
use crate::{chunking_components::*, chunking_resources::*, chunking_systems::*, regioning::{self, RegioningSystems}, terrain_gen::{self,  *}, tile::{self, *}, tilemap_components::TmapHashIdtoTextureIndex, tilemap_resources::*, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        bevy_ecs_tilemap::TilemapPlugin, 
        terrain_gen::plugin,
        tile::plugin,
        regioning::plugin,
    ))

    .add_systems(Update, (
        clear_chunks_on_dim_change,
        rem_outofrange_chunks_from_activators, 
        despawn_unreferenced_chunks, 
        tile_assign_child_of.run_if(on_timer(Duration::from_millis(100))),
        (
            activate_chunks_every_second,
            detect_activators_with_changes, 
            visit_chunks_around_activators,
            detect_camera_change_pos, 
            update_chunk_visib,
            recheck_chunk_visibility.run_if(on_timer(Duration::from_millis(1000))),
            process_tiles_pre.before(despawn_unreferenced_chunks)//NO TOCAR
        ).in_set(ChunkSystems)
    ))
    .configure_sets(Update, (
        (TerrainGenSystems, ChunkSystems, RegioningSystems).in_set(GameplaySystems)
    ))

    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            TilingSystems.before(TerrainGenSystems),
            DimensionSystems.before(TerrainGenSystems),
            TerrainGenSystems.before(RegioningSystems),
            TerrainGenSystems.before(GameplaySystems),
        )
    )
    .register_type::<ChunkTmapsMap>()
    .register_type::<LoadedChunks>()
    .register_type::<ActivatingChunks>()
    .register_type::<ChunkPos>()
    .register_type::<AaChunkRangeSettings>()
    .register_type::<TmapHashIdtoTextureIndex>()
    .register_type::<MassCollectedTiles>().register_type::<TileMassSpawnBundle>()
    .register_type::<PoissonDisk>()
    
    .init_resource::<LoadedChunks>()
    .init_resource::<AaChunkRangeSettings>()
    .init_resource::<MassCollectedTiles>()
    .init_resource::<TilemapAsyncTasks>()

    .replicate::<PoissonDisk>()


    .add_message::<CheckChunkDespawn>()
    .add_message::<ReactivateChunksFor>()
    .add_message::<RecheckChunksVisibility>()

    

    
;
}