use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]pub being_ent: Entity,
    pub vec: Vec2,
}


#[derive(Message, Deserialize,Serialize, Clone, MapEntities)]
pub struct UnreliableTransform {
    #[entities]
    pub being_ent: Entity,
    pub trans: Transform,
}
impl UnreliableTransform {
    pub fn new(being: Entity, trans: Transform, ) -> Self {
        Self { being_ent: being, trans, }
    }
}
