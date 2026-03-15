use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use tilemap_shared::{CardinalDirection, GlobalTilePos};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendStepRequest {
    #[entities]
    pub being_ent: Entity,
    pub dir: CardinalDirection,
    pub steps: u16,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncGpos {
    #[entities]
    pub being_ent: Entity,
    pub gpos: GlobalTilePos,
    pub dir: CardinalDirection,
    pub force_resync: bool,
}
