pub use ac_input::player_action_requests::{LocalMeleeAttackRequest};
#[allow(unused_imports)] use bevy::prelude::*;
use ::being_shared::{GoTo, NavOrderSource};


#[derive(Message, Debug, Clone, )]
pub struct MakeChunkSnapshotForChaser(pub Entity);

#[derive(Message, Debug, Clone, Copy)]
pub struct PredatorSeenByPrey {
    pub prey: Entity,
    pub predator: Entity,
}

#[derive(Message, Debug, Clone, )]
pub struct NavOrder {
    pub being_ent: Entity,
    pub priority: u8,
    pub source: NavOrderSource,
    pub go_to: Option<GoTo>,
    pub speed_throttle_mult: Option<f32>,
    pub max_speed: Option<f32>,
}
impl NavOrder {
    pub fn new(
        being_ent: Entity,
        priority: u8,
        source: NavOrderSource,
        go_to: Option<GoTo>,
        speed_throttle_mult: Option<f32>,
        max_speed: Option<f32>,
    ) -> Self {
        Self {
            being_ent,
            priority,
            source,
            go_to,
            speed_throttle_mult,
            max_speed,
        }
    }
}
