use crate::game_common_components::*;
use crate::prelude::GameCommonStringComponentsBundle;
use common::prelude::*;
use ::sprite_shared::sprite_scale_offset::AllScalesAndOffsets;
use ::sprite_shared::*;
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use tilemap_shared::*;

#[derive(Bundle)]
pub struct EntityZeroCloneDeny(
    EntityZero,
    InteractionZones,
    AcZ,
    YSortOrigin,
    TiledCollisionMask,
    SizeInTiles,
    TagSet,
    HashedTagsVec,
    AddHashIdFromStrId,
    HashId,
    GameCommonStringComponentsBundle,
);

#[derive(Bundle)]
pub(crate) struct DenyForEntityZeroClonedChild(
    EntityZero,
    BaseHolderRef,
    Disabled,
    ImagePathHolder,
    AcZ,
    YSortOrigin,
    AllScalesAndOffsets,
    StrId,
);
