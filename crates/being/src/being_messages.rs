use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Message, Clone, Copy)]
pub struct LocalMeleeAttackRequested {
    pub being_ent: Entity,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClientMeleeAttack {
    #[entities]
    pub being_ent: Entity,
}
