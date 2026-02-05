
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use tilemap_shared::GlobalTilePos;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
#[require(ProcessedInputMoveVector, FinalMoveVector, OutputSpeedMagnitude)]
pub struct InputMoveVector(pub Vec2);//USADO TMB POR BOTS
//no se incluye la coordenada z de agacharse o saltar porq esto se debe mandar reliably ya q no se spammea tanto

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Message)]
pub struct InputJump;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Message)]
pub struct InputDuck;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct ProcessedInputMoveVector(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct FinalMoveVector(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct OutputSpeedMagnitude(pub f32);


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(QueuedGridMoveDir)]
pub struct GridLockedMovement;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct QueuedGridMoveDir(pub Vec2);




//PONER WALLCLIMBER? PUEDE TRASPASAR MURALLAS SI NO HAY TECHO DEL OTRO LADO
//UTIL PARA RAZAS DE IGUANAS O ARAÑAS


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct WallPhaser;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct InnateMovementCapability;//NO SACARSELO SOLO PORQ ESTÉ ULTRAHERIDO


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(InnateMovementCapability)] 
pub struct LandWalker;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(InnateMovementCapability)] 
pub struct Swimmer;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(InnateMovementCapability)] 
pub struct Flier;

