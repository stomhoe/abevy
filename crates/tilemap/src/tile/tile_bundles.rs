use bevy::prelude::*;
use bevy::render::sync_world::SyncToRenderWorld;
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
    SparedFromHotReloading,
    GameCommonStringComponentsBundle,
    WalkSpeedMultIfOnTop,
    SizeInTiles,
    TileChildSprite,
    BlocksProjectiles,
    AdjRetexConfig,
    InteractionZones,
    GlobalTilePosWeightedSampler,
    TileCollisionMask,
);

#[derive(Bundle)] #[allow(unused, )]
struct ToDenyOnReleaseBuild(Name);

use serde::{Deserialize, Serialize};
#[derive(Bundle, Default, Clone, Copy, Debug, Reflect, Serialize, Deserialize)]
pub struct TileBundleNoTileFlip {
    pub position: TilePos,
    pub texture_index: TileTextureIndex,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
    pub color: TileColor,
    pub old_position: TilePosOld,
}
