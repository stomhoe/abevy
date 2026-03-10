use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]pub being_ent: Entity,
    pub vec: Vec2,
    pub input_seq: u32,
    pub client_tick: u32,
}


#[derive(Message, Deserialize,Serialize, Clone, MapEntities)]
pub struct UnreliableTransform {
    #[entities]
    pub being_ent: Entity,
    pub trans: Transform,
    pub last_processed_input_seq: u32,
}
impl UnreliableTransform {
    pub fn new(being: Entity, trans: Transform, last_processed_input_seq: u32) -> Self {
        Self { being_ent: being, trans, last_processed_input_seq }
    }
}

#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct GridMoveStateAck {
    #[entities]
    pub being_ent: Entity,
    pub tile_pos: Vec2,
    pub moving_dir: Vec2,
    pub step_progress: f32,
    pub last_processed_input_seq: u32,
}
