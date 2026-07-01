use bevy::ecs::{
    entity::{Entity, MapEntities},
    event::Event,
    message::Message,
};
use tilemap_shared::{DimensionRef, GlobalTilePos};
use serde::{Deserialize, Serialize};

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct BeingDebugSpeedApplied {
    #[entities]
    pub being_ent: Entity,
    pub applied: bool,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugSetSpeedRequest {
    #[entities]
    pub being_ent: Entity,
    pub speed: f32,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugSetBeingDimensionRequest {
    #[entities]
    pub being_ent: Entity,
    pub dim_ref: DimensionRef,
    #[entities]
    pub dimension_ent: Entity,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugTeleportBeingRequest {
    #[entities]
    pub being_ent: Entity,
    pub gpos: GlobalTilePos,
}

#[derive(Event, Clone)]
pub struct LocalDebugTeleportBeingRequest {
    pub being_ent: Entity,
    pub gpos: GlobalTilePos,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugSetBeingCurrentHpRequest {
    #[entities]
    pub being_ent: Entity,
    pub current_hp: f32,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugSetBeingCurrentBloodRequest {
    #[entities]
    pub being_ent: Entity,
    pub blood: f32,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugKillBeingRequest {
    #[entities]
    pub being_ent: Entity,
}

#[derive(Event, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClientDebugReviveBeingRequest {
    #[entities]
    pub being_ent: Entity,
}
