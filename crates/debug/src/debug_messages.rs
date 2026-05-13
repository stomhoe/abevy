use bevy::ecs::{
    entity::{Entity, MapEntities},
    message::Message,
};
use tilemap_shared::{DimensionRef, GlobalTilePos};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct BeingDebugSpeedApplied {
    #[entities]
    pub being_ent: Entity,
    pub applied: bool,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClientDebugIncreaseSpeedRequest {
    #[entities]
    pub being_ent: Entity,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClientDebugDecreaseSpeedRequest {
    #[entities]
    pub being_ent: Entity,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClientDebugSetBeingDimensionRequest {
    #[entities]
    pub being_ent: Entity,
    pub dim_ref: DimensionRef,
    #[entities]
    pub dimension_ent: Entity,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClientDebugTeleportBeingRequest {
    #[entities]
    pub being_ent: Entity,
    pub gpos: GlobalTilePos,
}
