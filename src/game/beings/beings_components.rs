use bevy::{platform::{collections::HashMap, hash}, prelude::*};
use strum_macros::EnumCount;
use vec_collections::VecSet;
use superstate::{SuperstateInfo};
use crate::{common::common_components::GameZindex, game::{player::player_components::CameraTarget, tilemap::chunking_components::ActivatesChunks}, AppState};



#[derive(Component)]
#[require(StateScoped::<AppState>)]
pub struct Body {}
 

#[derive(Component, Debug, Default)]//USADO TAMBIEN POR BOTS
pub struct InputMoveDirection(pub Vec3);

#[derive(Component, Default)]
#[require(SuperstateInfo<PlayerDirectControllable>, ActivatesChunks)]
pub struct PlayerDirectControllable;

#[derive(Component)]
#[require(PlayerDirectControllable)]
pub struct ControlledBySelf;

#[derive(Component)]
#[require(PlayerDirectControllable)]
pub struct Free;

#[derive(Component)]
#[require(PlayerDirectControllable)]
pub struct ControlledByOtherPlayer {
    pub player: Entity,
}

#[derive(Component, Debug, PartialEq, Eq, Hash)]
#[require(InputMoveDirection, GameZindex(500.))]
pub struct Being(pub u32);


#[derive(Component)]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {
    #[relationship]
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



#[derive(Component, Default)] pub struct CanMove;


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component)] #[require(CanMove)] pub struct LandWalker;

#[derive(Component)] #[require(CanMove)] pub struct Swimmer;

#[derive(Component)] #[require(CanMove)] pub struct Flier;

//NO SÉ SI USAR UN HASHMAP