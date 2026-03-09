use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Last movement intent received from a remote client.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RemoteMoveInput(pub Vec2);

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

#[derive(Component, Debug, Clone)]
pub struct PendingServerTransform(pub Transform);

//PONER WALLCLIMBER? PUEDE TRASPASAR MURALLAS SI NO HAY TECHO DEL OTRO LADO
//UTIL PARA RAZAS DE IGUANAS O ARAÑAS
