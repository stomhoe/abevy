use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;


use ::game_common::*;
use item_shared::ItemsGeneratedOnDeath;
use ::tilemap_shared::{*, DeleteOtherTilesInSamePos, PortalSeri};

use crate::tile::tile_components::*;
use crate::tile::tile_resources::TileImagePaths;
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
    PathHolder,
    DeleteOtherTilesInSamePos,
    PortalRecipe,
    PortalSeri,

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
    ChangeFacingDirectionBasedOnHash,
    RotateTransform,
    TerrBlendParams,
    ItemsGeneratedOnDeath,
    TileIndex,

);

#[derive(Bundle)] #[allow(unused, )]
struct ToDenyOnReleaseBuild(Name);

#[derive(Bundle, Debug, Clone)]
pub struct TileMassSpawnBundle {
    pub templ_ref: TemplEntiRef,
    pub gpos: GlobalTilePos,
    pub snap_to_gpos: SnapTransformToGpos,
    pub dim_ref: DimensionRef,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
}

#[derive(Bundle, Clone, Copy, Debug, )]
pub struct TileBundleNoTileFlip(pub TilePos, pub TileTextureIndex, pub TilemapId, pub TileVisible, pub TileColor, pub TilePosOld);
