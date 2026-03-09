use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendMeleeAttack {
    #[entities]
    pub being_ent: Entity,
}
