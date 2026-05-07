use crate::game_common_components::*;
use crate::GameCommonStringComponentsBundle;
use ::common::*;
use ::sprite_shared::*;
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use ::tilemap_shared::*;

#[derive(Bundle)]
pub struct EntityZeroCloneDeny(
    Templ,
    InteractionZones,
    AcZ,
    YSortOrigin,
    SizeInTiles,
    TagSet,
    HashedTagsVec,
    AddHashIdFromStrId,
    HashId,
    GameCommonStringComponentsBundle,
    CloneTemplChildren,
    SpriteDists,
);

#[derive(Bundle)]
pub struct DenyForTemplClonedChildren(
    Templ,
    BaseHolderRef,
    Disabled,
    PathHolder,
    AcZ,
    YSortOrigin,
    AllScalesAndOffsetsAndRotation,
    StrId,
    Children,
    SpriteDists,
);
