use bevy::prelude::*;
use bevy::ecs::entity::MapEntities;
use common::common_components::Grounding;
use serde::{Deserialize, Serialize};
use tilemap_shared::CardinalDirection;

#[derive(Message, Clone, PartialEq, Eq, Hash)]
pub struct BeingChangedMoveState(pub Entity);

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncMoveState {
    #[entities] pub being_ent: Entity,
    pub moving: bool,
    pub grounding: Option<Grounding>,
    pub direction: Option<CardinalDirection>,
}
