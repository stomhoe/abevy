use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::{common_states::*};

use game_common::HostSystems;
use game_common::game_common::GameplaySystems;
use ::tilemap_shared::*;
use crate::{
    chunking,
    terrain,
    terrain::{TerrainGenSystems, TerrainSystems, },
    tile,
    tile::TilingSystems,
    tilemap_resources::*,
    tilemap_nav_systems::*,
    tilemap_systems::*,
    tilemap_despawn_systems::*,
    tilemap_structs::*,
    tilemap_terrbl_systems::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        bevy_ecs_tilemap::TilemapPlugin,
        terrain::plugin,
        tile::plugin,
        chunking::plugin,
    ))

    .add_systems(Update, (
        (
            process_tiles_pre
            .in_set(PreChunkDespawnSystems)//if this is removed everything breaks
            .before(tile::tile_despawn_systems::despawn_other_tiles_in_same_pos_if_not_excepted),//if this is removed you can get a glimpse of the tilemap which was there before removal
            refresh_terrbl_tilemaps.after(process_tiles_pre),
        ).in_set(ChunkSystems)
    ))
    .add_systems(Update, track_spawned_tiles_for_ai_nav.in_set(HostSystems))
    .add_observer(on_tilemap_despawn)
    .configure_sets(Update, (
        (TerrainGenSystems, ChunkSystems).in_set(GameplaySystems)
    ))

    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            TilingSystems.before(TerrainSystems),
            DimensionSystems.before(TerrainGenSystems),
            TerrainGenSystems.before(GameplaySystems),
        )
    )

    .init_resource::<MassCollectedTiles>()
    .init_resource::<TmapMap>()
    .init_resource::<ImportantRegisteredPositions>()

    .replicate_once::<CardinalDirection>()
    .replicate_once::<DiagonalCardinalDirection>()
    .replicate::<InteractionZones>()
    .init_resource::<SpriteTilesAtGpos>()
    .init_resource::<AiNavTileBlockedGposCounts>()
    .replicate_once::<PoissonDisk>()


    .init_resource::<ItemsAtGpos>()
    .init_resource::<CardinalDirAtGpos>()


;
}
