pub use ac_input::player_action_requests::{LocalMeleeAttackRequest};
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Message, Debug, Clone, )]
pub struct MakeChunkSnapshotForChaser(pub Entity);
