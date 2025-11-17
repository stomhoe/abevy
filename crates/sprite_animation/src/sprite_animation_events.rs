use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Message, Serialize, Clone, Component, MapEntities)]
pub struct MoveStateUpdated {
    #[entities]pub being_ent: Entity, pub moving: bool,
}
