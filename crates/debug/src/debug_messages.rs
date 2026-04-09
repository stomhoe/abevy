use bevy::ecs::{
    entity::{Entity, MapEntities},
    message::Message,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct BeingDebugSpeedApplied {
    #[entities]
    pub being_ent: Entity,
    pub applied: bool,
}
