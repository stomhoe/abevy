use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_components::{StrId};
use serde::{Deserialize, Serialize};

use crate::movement_components::*;

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]pub being_ent: Entity, pub vec: InputMoveVector,
}

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
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
