
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};
use sprite_shared::animation_shared::{DOWN, LEFT, RIGHT, UP};




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
#[require(InputSpeedVector, )]
pub struct InputMoveVector(pub Vec2);//USADO TMB POR BOTS
//no se incluye la coordenada z de agacharse o saltar porq esto se debe mandar reliably ya q no se spammea tanto

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Event)]
pub struct InputJump;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Event)]
pub struct InputDuck;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct InputSpeedVector(pub Vec2);
use strum::{VariantNames};
use strum_macros::{AsRefStr, Display, VariantNames};


#[allow(unused_parens, )]
#[derive(Component, Debug, Deserialize, Serialize, Default, AsRefStr, Display )]
#[strum(serialize_all = "lowercase")]
pub enum FacingDirection { #[default] Down, Left, Right, Up, }//PARA CAMBIAR ALEATORIAMENTE AL SPAWNEAR, HACER UN SISTEMA PARA BEINGS ADDED Q USE BEVY_RAND




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
pub struct WallPhaser;//HACER ATACABLE POR MODIFIERS PARA DEFENDERSE/ESCAPAR DE FANTASMAS

#[derive(Component, Default)] pub struct InnateMovementCapability;//NO SACARSELO SOLO PORQ ESTÉ ULTRAHERIDO


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component)] #[require(InnateMovementCapability)] pub struct LandWalker;

#[derive(Component)] #[require(InnateMovementCapability)] pub struct Swimmer;

#[derive(Component)] #[require(InnateMovementCapability)] pub struct Flier;