use bevy::ecs::{entity::{Entity, MapEntities}, message::Message};
use modifier_shared::modifier_components::BaseValue;

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct UpdateBeingSpeed {
    #[entities]pub being_ent: Entity, pub value: BaseValue,
}
