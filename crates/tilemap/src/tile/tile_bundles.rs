use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::{HashedTagsVec, TagSet};

use game_common::game_common_components::*;
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
);

#[derive(Bundle)] #[allow(unused, )]
struct ToDenyOnReleaseBuild(Name);
