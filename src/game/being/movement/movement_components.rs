use std::time::Duration;

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::being::movement::{
//    movement_resources::*,
//    movement_constants::*,

};




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct InputMoveVector(pub Vec2);//USADO TMB POR BOTS
//no se incluye la coordenada z de agacharse o saltar porq esto se debe mandar reliably ya q no se spammea tanto

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Event)]
pub struct InputJump;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Event)]
pub struct InputDuck;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct FinalMoveVector(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct VoluntarilyMoving;





#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub enum Altitude{
    #[default] 
    OnGround,
    Swimming,
    Floating,
}

//PONER WALLCLIMBER? PUEDE TRASPASAR MURALLAS SI NO HAY TECHO DEL OTRO LADO
//UTIL PARA RAZAS DE IGUANAS O ARAÑAS

#[derive(Component)]
pub struct WallPhaser;

#[derive(Component, Default)] pub struct InnateMovementCapability;//NO SACARSELO SOLO PORQ ESTÉ ULTRAHERIDO


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component)] #[require(InnateMovementCapability)] pub struct LandWalker;

#[derive(Component)] #[require(InnateMovementCapability)] pub struct Swimmer;

#[derive(Component)] #[require(InnateMovementCapability)] pub struct Flier;