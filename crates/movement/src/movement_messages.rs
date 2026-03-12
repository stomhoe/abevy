use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use tilemap_shared::GlobalTilePos;

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendStepRequest {
    #[entities]
    pub being_ent: Entity,
    pub gpos: GlobalTilePos,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncGpos {
    #[entities]
    pub being_ent: Entity,
    pub gpos: GlobalTilePos,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncTransform {
    #[entities]
    pub being_ent: Entity,
    pub transform: Transform,
}
