use being_shared::Grounding;
use bevy::{ecs::entity::MapEntities, prelude::*};

use serde::{Deserialize, Serialize};
use tilemap_shared::CardinalDirection;


#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncMoveState {
    #[entities]pub being_ent: Entity,
    pub moving: bool,
    pub grounding: Option<Grounding>,
    pub direction: Option<CardinalDirection>
}
