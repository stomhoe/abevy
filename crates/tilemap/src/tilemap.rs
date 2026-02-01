use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use dimension_shared::DimensionSystems;
use game_common::game_common::GameplaySystems;
use ::tilemap_shared::*;
use crate::{chunking::{self, chunking_despawn_systems::despawn_chunks}, regioning::{self, RegioningSystems}, terrain_gen::{self,  *}, tile::{self, tile_systems::despawn_if_not_excepted}, tilemap_components::HashIdToTexIndex, tilemap_resources::*, tilemap_systems::*};

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
            requeue_limbo_tiles.run_if(on_timer(Duration::from_secs_f32(RECHECK_LIMBO_TILES_FREQ))),
            process_tiles_pre.before(despawn_chunks).before(despawn_if_not_excepted),//DON'T TOUCH
        ).in_set(ChunkSystems)
    ))
    .add_observer(on_tilemap_despawn)
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
    .init_resource::<TmapMap>()


    .replicate::<PoissonDisk>()
    

    

    
;
}