use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use dimension_shared::DimensionSystems;
use game_common::game_common::GameplaySystems;
use ::tilemap_shared::*;
use crate::{chunking::{self, chunking_components::*, chunking_despawn_systems::{CheckChunkDespawn, ForceChunkDespawn, on_message_signal_despawn_all_chunks, rem_outofrange_chunks_from_activators, periodically_check_despawn_unreferenced_chunks, despawn_chunks}, chunking_resources::*, chunking_spawn_systems::{activate_chunks_every_second, detect_activators_with_pos_changes, spawn_chunks_around_activators}, chunking_visibility_systems::{detect_camera_change_pos, update_chunk_visib, periodically_recheck_chunk_visibility, RecheckChunksVisibility}, chunking_spawn_systems::ReactivateChunksFor}, regioning::{self, RegioningSystems}, terrain_gen::{self,  *}, tile::{self, tile_systems::despawn_if_not_excepted}, tilemap_components::HashIdToTexIndex, tilemap_resources::*, tilemap_systems::*};

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
        chunking::plugin,
    ))

    .add_systems(Update, (
        
        tmaptile_assign_child_of,
        (
            reparent_orphan_tilemaps.run_if(on_timer(Duration::from_secs(1))),//dejar en 1
            requeue_limbo_tiles.run_if(on_timer(Duration::from_millis(500))),
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

    .register_type::<HashIdToTexIndex>()
    .register_type::<MassCollectedTiles>().register_type::<TileMassSpawnBundle>()
    .register_type::<PoissonDisk>()
    
    .init_resource::<MassCollectedTiles>()
    .init_resource::<TilemapLimboTiles>()

    .replicate::<PoissonDisk>()
    

    

    
;
}