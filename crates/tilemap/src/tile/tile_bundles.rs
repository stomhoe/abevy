use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::{HashedTagsVec, TagSet};

use game_common::game_common_components::*;
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;

use game_common::prelude::*;
use item_shared::ItemsGeneratedOnDeath;
use ::tilemap_shared::*;

use crate::tile::tile_components::*;
use crate::tile::tile_resources::{PortalSeri, TileImagePaths};
use crate::tile::tile_shader::tile_shader_components::*;


#[derive(Bundle)]
pub struct ToDenyOnTileClone(
    EntityZeroCloneDeny,

    MinDistancesMap,
    KeepDistanceFrom,
    TileHashIdsHandles,
    Replicated,
    TileShaderRef,
    ChildOf,
    TileColor,
    ImagePathHolder,
    DeleteOtherTilesInSamePos,
    PortalRecipe,
    PortalSeri,

    //children entities don't get cloned
    Children,

    TileImagePaths,
    AssetScoped,
    HotReload,
    WalkSpeedMultIfOnTop,
    TileChildSprite,
    BlocksProjectiles,
    AdjRetexConfig,

    GlobalTilePosWeightedSampler,
    FlipDiagonallyBasedOnHash,
    FlipVerticallyBasedOnHash,
    FlipHorizontallyBasedOnHash,
    OffsetForTerrgenPlacement,
    RotateCardinallyBasedOnHash,
    TransformBasedCardRotation,
    TerrBlendParams,
    ItemsGeneratedOnDeath,


);

#[derive(Bundle)] #[allow(unused, )]
struct ToDenyOnReleaseBuild(Name);

#[derive(Bundle, Debug, Clone)]
pub struct TileMassSpawnBundle {
    pub ezero_ref: EntityZeroRef,
    pub gpos: GlobalTilePos,
    pub snap_to_gpos: SnapTransformToGpos,
    pub dim_ref: DimensionRef,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
}

#[derive(Bundle, Clone, Copy, Debug, )]
pub struct TileBundleNoTileFlip(pub TilePos, pub TileTextureIndex, pub TilemapId, pub TileVisible, pub TileColor, pub TilePosOld);
