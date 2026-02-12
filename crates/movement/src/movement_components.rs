use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Raw input movement vector - updated by keyboard/AI and synced to server
#[derive(Component, Debug, Default, Clone, )]
pub struct InputDirection(pub Vec2);

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

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct WallPhaser;

//borrar estos
#[derive(Component, Debug, Default, Copy, Clone)]
pub struct InnateMovementCapability;//NO SACARSELO SOLO PORQ ESTÉ ULTRAHERIDO

// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(InnateMovementCapability)]
pub struct LandWalker;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(InnateMovementCapability)]
pub struct Swimmer;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(InnateMovementCapability)]
pub struct Flier;
