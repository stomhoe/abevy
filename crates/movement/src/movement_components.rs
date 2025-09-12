
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
#[require(ProcessedInputVector, )]
pub struct InputMoveVector(pub Vec2);//USADO TMB POR BOTS
//no se incluye la coordenada z de agacharse o saltar porq esto se debe mandar reliably ya q no se spammea tanto

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Event)]
pub struct InputJump;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Event)]
pub struct InputDuck;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct ProcessedInputVector(pub Vec2);





//PONER WALLCLIMBER? PUEDE TRASPASAR MURALLAS SI NO HAY TECHO DEL OTRO LADO
//UTIL PARA RAZAS DE IGUANAS O ARAÑAS

#[derive(Component)]
pub struct WallPhaser;//HACER ATACABLE POR MODIFIERS PARA DEFENDERSE/ESCAPAR DE FANTASMAS

#[derive(Component, Default)] pub struct InnateMovementCapability;//NO SACARSELO SOLO PORQ ESTÉ ULTRAHERIDO


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component)] #[require(InnateMovementCapability)] pub struct LandWalker;

#[derive(Component)] #[require(InnateMovementCapability)] pub struct Swimmer;

#[derive(Component)] #[require(InnateMovementCapability)] pub struct Flier;