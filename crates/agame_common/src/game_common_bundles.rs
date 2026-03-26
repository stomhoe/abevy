use crate::game_common_components::*;
use crate::GameCommonStringComponentsBundle;
use ::common::*;
use ::sprite_shared::*;
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use ::tilemap_shared::*;

#[derive(Bundle)]
pub struct EntityZeroCloneDeny(
    TemplEnti,
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
);

#[derive(Bundle)]
pub struct DenyForEntityZeroClonedChild(
    TemplEnti,
    BaseHolderRef,
    Disabled,
    ImagePathHolder,
    AcZ,
    YSortOrigin,
    AllScalesAndOffsets,
    StrId,
);
