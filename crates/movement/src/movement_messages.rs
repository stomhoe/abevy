use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use tilemap_shared::CardinalDirection;

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]
    pub being_ent: Entity,
    pub dir: IVec2,
    pub input_seq: u32,
}

#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct GridMoveStateAck {
    #[entities]
    pub being_ent: Entity,
    pub tile_pos: IVec2,
    pub visual_origin_tile: IVec2,
    pub step_dir: IVec2,
    pub progress_ticks: u16,
    pub step_ticks_total: u16,
    pub facing_dir: CardinalDirection,
    pub last_processed_input_seq: u32,
}
