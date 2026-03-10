use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Last movement intent received from a remote client.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RemoteMoveInput(pub Vec2);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LastProcessedMoveInputSeq(pub u32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LastProcessedMoveInputTick(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct PendingMoveIntent {
    pub input_seq: u32,
    pub client_tick: u32,
    pub dir: Vec2,
}

#[derive(Component, Debug, Default, Clone)]
pub struct PendingMoveIntents(pub Vec<PendingMoveIntent>);

/// Processed movement state - direction after modifiers and calculated speed
#[derive(Component, Debug, Default, Clone, )]
pub struct MoveVecMag {
    pub norm_move_dir: Vec2,
    pub speed_magnitude: f32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct GridLockedMovement{
    pub queued_move_dir: Vec2,
    pub active_move_dir: Vec2,
}

//PONER WALLCLIMBER? PUEDE TRASPASAR MURALLAS SI NO HAY TECHO DEL OTRO LADO
//UTIL PARA RAZAS DE IGUANAS O ARAÑAS
