use bevy::{platform::{collections::HashMap, hash}, prelude::*};
use strum_macros::EnumCount;
use vec_collections::VecSet;

use crate::AppState;



#[derive(Component)]
#[require(StateScoped::<AppState>)]
pub struct Body {}


 
use superstate::{SuperstateInfo};

#[derive(Component, Debug, Default)]
pub struct InputMoveDirection(pub Vec3);

#[derive(Component, Default)]
#[require(SuperstateInfo<PlayerDirectControllable>)]
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

#[derive(Component)]
pub struct Being(pub BeingNid);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BeingNid(pub u32);

#[derive(Component)]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {
    #[relationship]
    pub master: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);

//crear una entidad por cada instancia de clase existente
#[derive(Component, Debug)]
pub struct Class(pub ClassNid);
//esto no va en los beings

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClassNid(pub u32);

#[derive(Component, Debug)]
pub struct Race {
    pub nid: u32,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RaceNid(pub u32);

#[derive(Component, Debug)]
pub struct ClassesRefs(pub VecSet<[Entity; 3]>);
//esto va en los beings, permite tener multiples clases

#[derive(Component, Debug)]
pub struct RaceRef(pub Entity);

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
pub enum BaseRacesNids {
    Human = 0,
    Dwarf,
    Elf,
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



#[derive(Component)]
pub struct WallPhaser;



#[derive(Component, Default)] pub struct CanMove;


// NO SON EXLUSIVOS ASÍ Q NO ES SUPERSTATE
#[derive(Component)] #[require(CanMove)] pub struct LandWalker;

#[derive(Component)] #[require(CanMove)] pub struct Swimmer;

#[derive(Component)] #[require(CanMove)] pub struct Flier;

//NO SÉ SI USAR UN HASHMAP