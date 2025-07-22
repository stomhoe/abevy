use bevy::{platform::{collections::HashMap}, prelude::*};
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use superstate::{SuperstateInfo};
use crate::{common::common_components::GameZindex, game::{tilemap::chunking_components::ActivatesChunks}, AppState};



#[derive(Component)]
#[require(StateScoped::<AppState>)]
pub struct Body {}
 

#[derive(Component, Debug, Default)]//USADO TAMBIEN POR BOTS
pub struct InputMoveDirection(pub Vec3);

#[allow(dead_code)] 
#[derive(Component, Debug)]
pub struct RaceRef(pub Entity);

#[derive(Component, Default)]
#[require(SuperstateInfo<PlayerDirectControllable>, ActivatesChunks)]//TODO PONER ActivatesChunks CUANDO SEA ADECUADO
pub struct PlayerDirectControllable;

#[derive(Component)]
#[require(PlayerDirectControllable)]
pub struct AvailableForControl;

#[derive(Component, Debug, Deserialize, Serialize)]
#[require(PlayerDirectControllable, Replicated)]
pub struct ControlledBy ( #[entities] pub Entity);

#[derive(Component, Debug, Default, )]
pub struct ControlledBySelf;

#[derive(Component, Debug, Deserialize, Serialize)]
#[require(InputMoveDirection, GameZindex(500.), Replicated, Altitude, Visibility)]
pub struct Being;

#[derive(Component)]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {
    #[relationship] #[entities]
    pub master: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);


#[derive(Component, Debug)]
pub struct LearningMultiplier(pub HashMap<Entity, f32>);

#[derive(Component, Debug)]
pub struct LearnableSkill {
    pub nid: u32,
}



#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Moving;


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

//NO SÉ SI USAR UN HASHMAP



#[derive(Component, Debug, )]
pub struct CurrentDimension(u32);//TANTO PARA BEINGS COMO PARA OBJETOS Y TILES

//HACER Q AFECTE LA VISIBILIDAD DE LAS COSAS . Q TENGAS
//DESPUES EN EL TERRAIN_GEN_SYSTEMS SE PUEDE HACER UN MATCH SEGÚN LA DIMENSION ACTUAL DEL PLAYER
//Y TENER UN PROC DE GENERACIÓN DE TERRAIN POR DIMENSION ANTES DE ENTRAR AL DOBLE FOR DE GENERACIÓN DE CADA TILE
