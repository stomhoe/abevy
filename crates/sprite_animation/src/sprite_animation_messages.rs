use being_shared::Grounding;
use bevy::{ecs::entity::MapEntities, prelude::*};
use game_common::game_common_components::CardinalDirection;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncMoveState {
    #[entities]pub being_ent: Entity,
    pub moving: bool,
    pub grounding: Option<Grounding>,
    pub direction: Option<CardinalDirection>
}
