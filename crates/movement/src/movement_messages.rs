use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use tilemap_shared::CardinalDirection;

use crate::prelude::PendingMoveIntent;

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]
    pub being_ent: Entity,
    pub intent: PendingMoveIntent
}
