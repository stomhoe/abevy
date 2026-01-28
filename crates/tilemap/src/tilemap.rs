use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use dimension_shared::DimensionSystems;
use game_common::game_common::GameplaySystems;
use ::tilemap_shared::*;
use crate::{chunking_components::*, chunking_resources::*, chunking_systems::*, regioning::{self, RegioningSystems}, terrain_gen::{self,  *}, tile::{self, tile_systems::despawn_if_not_excepted}, tilemap_components::TmapHashIdtoTextureIndex, tilemap_resources::*, tilemap_systems::*};

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
        rem_outofrange_chunks_from_activators, 
        despawn_chunks, 
        tmaptile_assign_child_of,
        despawn_all_chunks_on_order,
        
        (
            /* --- reparent_orphan_tilemaps.run_if(on_timer(Duration::from_secs(5))), --- */
            activate_chunks_every_second,
            detect_activators_with_pos_changes, 
            spawn_chunks_around_activators.after(despawn_chunks).after(despawn_if_not_excepted),//DON'T TOUCH
            requeue_limbo_tiles.run_if(on_timer(Duration::from_millis(500))),
            detect_camera_change_pos, 
            update_chunk_visib,
            periodically_check_despawn_unreferenced_chunks.run_if(on_timer(Duration::from_millis(500))),
            periodically_recheck_chunk_visibility.run_if(on_timer(Duration::from_millis(500))),
            process_tiles_pre.before(despawn_chunks),//DON'T TOUCH

            // DON'T TOUCH
        ).in_set(ChunkSystems)
    ))
    .configure_sets(Update, (
        (TerrainGenSystems, ChunkSystems, RegioningSystems).in_set(GameplaySystems)
    ))

    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            crate::tile::TilingSystems.before(TerrainGenSystems),
            DimensionSystems.before(TerrainGenSystems),
            TerrainGenSystems.before(RegioningSystems),
            TerrainGenSystems.before(GameplaySystems),
        )
    )
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
    .init_resource::<TilemapLimboTiles>()
    .init_resource::<TilemapAsyncTasks>()

    .replicate::<PoissonDisk>()


    .add_message::<CheckChunkDespawn>()
    .add_message::<ReactivateChunksFor>()
    .add_message::<RecheckChunksVisibility>()
    .add_message::<ForceChunkDespawn>()
    .add_message::<ForceAllChunksDespawn>()
    

    
;
}