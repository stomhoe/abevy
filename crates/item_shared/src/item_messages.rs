use bevy::prelude::*;
use game_common::game_common_components::TemplEntiRef;
use tilemap_shared::{DimensionRef, GlobalTilePos};



#[derive(Message, Debug, Copy, Clone)]
pub enum ItemOperation {
    FromTempl(TemplEntiRef, KnownItemDest),
    Preexisting(Entity, Option<KnownItemDest>), // if None, walk ancestry to find ground pos of holder entity and drop item there
}

#[derive(Debug, Copy, Clone)]
pub enum KnownItemDest {
    Holder(Entity),
    Ground(DimensionRef, GlobalTilePos),
}

impl ItemOperation {
    pub fn spawn_on_ground(templ_ref: TemplEntiRef, dim_ref: DimensionRef, gpos: GlobalTilePos) -> Self {
        Self::FromTempl(templ_ref, KnownItemDest::Ground(dim_ref, gpos))
    }

    pub fn spawn_into_holder(templ_ref: TemplEntiRef, holder: Entity) -> Self {
        Self::FromTempl(templ_ref, KnownItemDest::Holder(holder))
    }

    pub fn drop_preexisting_on_holder_position(item: Entity) -> Self {
        Self::Preexisting(item, None)
    }

    pub fn teleport_preexisting_to_ground(item: Entity, dim_ref: DimensionRef, gpos: GlobalTilePos) -> Self {
        Self::Preexisting(item, Some(KnownItemDest::Ground(dim_ref, gpos)))
    }

    pub fn place_preexisting_in_holder(item: Entity, holder: Entity) -> Self {
        Self::Preexisting(item, Some(KnownItemDest::Holder(holder)))
    }
}
