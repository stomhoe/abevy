use bevy::{platform::{collections::HashMap, hash}, prelude::*};
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use strum_macros::EnumCount;
use vec_collections::VecSet;
use superstate::{SuperstateInfo};
use crate::{common::common_components::GameZindex, game::{game_components::Nid, player::player_components::CameraTarget, tilemap::chunking_components::ActivatesChunks}, AppState};



#[derive(Component)]
#[require(StateScoped::<AppState>)]
pub struct Body {}
 

#[derive(Component, Debug, Default)]//USADO TAMBIEN POR BOTS
pub struct InputMoveDirection(pub Vec3);

#[derive(Component, Default)]
#[require(SuperstateInfo<PlayerDirectControllable>, ActivatesChunks)]//TODO PONER ActivatesChunks CUANDO SEA ADECUADO
pub struct PlayerDirectControllable;

#[derive(Component)]
#[require(PlayerDirectControllable)]
pub struct Free;

#[derive(Component, Debug, Deserialize, Serialize)]
#[require(PlayerDirectControllable, Replicated)]
pub struct ControlledBy ( #[entities] pub Entity);

#[derive(Component, Debug, Default, )]
pub struct ControlledBySelf;

#[derive(Component, Debug, Deserialize, Serialize)]
#[require(InputMoveDirection, GameZindex(500.), Replicated)]
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

#[derive(Component)]
pub enum Sex {
    Male,
    Female,
}


#[derive(EnumCount)]
pub enum BaseGenTemplsNids {
    Raider = 0,
    Miner,
    Soldier,
}



#[derive(Component, Default)]
#[require(SuperstateInfo<AltitudeState>)]
pub struct AltitudeState;

#[derive(Component)] #[require(AltitudeState)]
pub struct Floating;

#[derive(Component, Default)]
#[require(AltitudeState, SuperstateInfo<TouchingGround>)]
pub struct TouchingGround;


#[derive(Component)] #[require(TouchingGround)]
pub struct Swimming;

#[derive(Component)] #[require(TouchingGround)]
pub struct Walking; 


//PONER WALLCLIMBER? PUEDE TRASPASAR MURALLAS SI NO HAY TECHO DEL OTRO LADO
//UTIL PARA RAZAS DE IGUANAS O ARAÑAS

#[derive(Component)]
pub struct WallPhaser;



#[derive(Component, Default)] pub struct MovementCapability;//NO SACARSELO SOLO PORQ ESTÉ ULTRAHERIDO


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component)] #[require(MovementCapability)] pub struct LandWalker;

#[derive(Component)] #[require(MovementCapability)] pub struct Swimmer;

#[derive(Component)] #[require(MovementCapability)] pub struct Flier;

//NO SÉ SI USAR UN HASHMAP



#[derive(Component, Debug, )]
pub struct CurrentDimension(u32);//TANTO PARA BEINGS COMO PARA OBJETOS Y TILES

//HACER Q AFECTE LA VISIBILIDAD DE LAS COSAS . Q TENGAS
//DESPUES EN EL TERRAIN_GEN_SYSTEMS SE PUEDE HACER UN MATCH SEGÚN LA DIMENSION ACTUAL DEL PLAYER
//Y TENER UN PROC DE GENERACIÓN DE TERRAIN POR DIMENSION ANTES DE ENTRAR AL DOBLE FOR DE GENERACIÓN DE CADA TILE