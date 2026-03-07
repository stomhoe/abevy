use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::{HashedTagsVec, TagSet};

use game_common::game_common_components::*;
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use game_common::game_common_string_components::*;

use sprite_shared::{AcZ, YSortOrigin};
use ::tilemap_shared::*;

use crate::tile::tile_components::*;
use crate::tile::tile_resources::{PortalSeri, TileImagePaths};
use crate::tile::tile_shader::tile_shader_components::*;


#[derive(Bundle)]
pub struct ToDenyOnTileClone(
    MinDistancesMap,
    KeepDistanceFrom,
    TileHashIdsHandles,
    Replicated,
    TileShaderRef,
    AcZ,
    YSortOrigin,
    ChildOf,
    TileColor,
    ImagePathHolder,
    DeleteOtherTiles,
    PortalRecipe,
    PortalSeri,
    TagSet,
    HashedTagsVec,
    //children entities don't get cloned
    Children,
    EntityZero,
    AddHashIdFromStrId,
    HashId,
    TileImagePaths,
    AssetScoped,
    HotReload,
    GameCommonStringComponentsBundle,
    WalkSpeedMultIfOnTop,
    SizeInTiles,
    TileChildSprite,
    BlocksProjectiles,
    AdjRetexConfig,
    InteractionZones,
    GlobalTilePosWeightedSampler,
    TileCollisionMask,
    FlipDiagonallyBasedOnHash,
    FlipVerticallyBasedOnHash,
    FlipHorizontallyBasedOnHash,
    OffsetForTerrgenPlacement,
    RotateCardinallyBasedOnHash,
    TransformBasedCardRotation,
    TerrBlendParams
);

#[derive(Bundle)] #[allow(unused, )]
struct ToDenyOnReleaseBuild(Name);

#[derive(Bundle, Debug, Clone)]
pub struct TileMassSpawnBundle {
    pub ezero_ref: EntityZeroRef,
    pub gpos: GlobalTilePos,
    pub dim_ref: DimensionRef,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
}

#[derive(Bundle, Clone, Copy, Debug, )]
pub struct TileBundleNoTileFlip(pub TilePos, pub TileTextureIndex, pub TilemapId, pub TileVisible, pub TileColor, pub TilePosOld);
