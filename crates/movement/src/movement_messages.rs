use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]pub being_ent: Entity,
    pub vec: Vec2,
}

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct TransformFromServer {
    #[entities]
    pub being: Entity,
    pub trans: Transform,
    pub interpolate: bool,
}
impl TransformFromServer {
    pub fn new(being: Entity, trans: Transform, interpolate: bool) -> Self {
        TransformFromServer { being, trans, interpolate }
    }
}
