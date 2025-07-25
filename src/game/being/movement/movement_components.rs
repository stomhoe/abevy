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
pub struct InputMoveVector(pub Vec3);//USADO TMB POR BOTS

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct FinalMoveVector(pub Vec3);
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