pub use ac_input::player_action_requests::{LocalMeleeAttackRequest};
#[allow(unused_imports)] use bevy::prelude::*;
use ::being_shared::{GoTo, NavOrderSource};
use ::being_shared::movement_shared_components::*;


#[derive(Message, Debug, Clone, )]
pub struct MakeChunkSnapshotForChaser(pub Entity);

#[derive(Message, Debug, Clone, )]
pub struct NavOrder {
    pub being_ent: Entity,
    pub priority: u8,
    pub source: NavOrderSource,
    pub go_to: Option<GoTo>,
    pub speed_throttle_mult: InputSpeedThrottleMult,
    pub max_speed: InputMaxSpeed,
}
impl NavOrder {
    pub fn new(
        being_ent: Entity,
        priority: u8,
        source: NavOrderSource,
        go_to: Option<GoTo>,
    ) -> Self {
        Self {
            being_ent,
            priority,
            source,
            go_to,
            speed_throttle_mult: InputSpeedThrottleMult::default(),
            max_speed: InputMaxSpeed::default(),
        }
    }

    pub fn with_speed_throttle(
        being_ent: Entity,
        priority: u8,
        source: NavOrderSource,
        go_to: Option<GoTo>,
        speed_throttle_mult: f32,
    ) -> Self {
        Self {
            being_ent,
            priority,
            source,
            go_to,
            speed_throttle_mult: InputSpeedThrottleMult(speed_throttle_mult),
            max_speed: InputMaxSpeed::default(),
        }
    }

    pub fn with_max_speed(
        being_ent: Entity,
        priority: u8,
        source: NavOrderSource,
        go_to: Option<GoTo>,
        max_speed: f32,
    ) -> Self {
        Self {
            being_ent,
            priority,
            source,
            go_to,
            speed_throttle_mult: InputSpeedThrottleMult::default(),
            max_speed: InputMaxSpeed(max_speed),
        }
    }
}
