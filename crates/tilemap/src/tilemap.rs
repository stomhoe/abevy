use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::{AppRegisterAndReplicateExt, common_states::*};

use game_common::game_common::GameplaySystems;
use ::tilemap_shared::*;
use crate::{chunking::{self, chunking_despawn_systems::despawn_chunks}, regioning::{self, RegioningSystems}, terrain::{self,  *}, tile::{self, tile_systems::despawn_if_not_excepted}, tilemap_resources::*, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        bevy_ecs_tilemap::TilemapPlugin,
        terrain::plugin,
        tile::plugin,
        regioning::plugin,
        chunking::plugin,
    ))

    .add_systems(Update, (
        (
            process_tiles_pre
            .in_set(PreChunkDespawnSystems)//if this is removed everything breaks
            .before(despawn_if_not_excepted),//if this is removed you can get a glimpse of the tilemap which was there before removal
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

    .init_resource::<MassCollectedTiles>()
    .init_resource::<TmapMap>()
    .init_resource::<ImportantRegisteredPositions>()

    .replicate_once::<CardinalDirection>()
    .replicate_once::<DiagonalCardinalDirection>()
    .init_resource::<SpriteTilesAtGpos>()
    .replicate::<PoissonDisk>()





;
}
