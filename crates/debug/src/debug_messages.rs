use bevy::ecs::{entity::{Entity, MapEntities}, message::Message};
use modifier::modifier_components::CurrFinalValue;

use serde::{Deserialize, Serialize};


#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct UpdateBeingSpeed {
    #[entities]pub being_ent: Entity, pub value: CurrFinalValue,
}
