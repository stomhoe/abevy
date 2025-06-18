use bevy::{platform::collections::HashMap, prelude::*};
use strum_macros::EnumCount;

use crate::AppState;



#[derive(Component)]
#[require(StateScoped::<AppState>)]
pub struct Body {}


#[derive(Component)]
pub struct Controllable {}


#[derive(Component)]
#[require()]
pub struct Being {}

impl Default for Being {
    fn default() -> Self {
        Self {  }
    }
}


#[derive(Component)]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {
    #[relationship] pub master: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);


//crear una entidad por cada instancia de clase existente
#[derive(Component, Debug)]
//esto no va en los beings
pub struct Class;


#[derive(Component, Debug,)]
//esto va en los beings, permite tener multiples clases
pub struct ClassRefs(Vec<Entity>);


#[derive(Component, Debug,)]
pub struct LearningMultiplier(
    HashMap<Entity, f32>
);


#[derive(Component)] pub enum Sex {Male, Female}

#[derive(PartialEq)]
pub struct ClassNid(u32);

#[derive(PartialEq)]
pub struct RaceNid(u32);

#[derive(PartialEq)]
pub struct BeingGenTemplNid(u32);

#[derive(PartialEq)]
pub struct LearningClassNid(u32);


#[derive(EnumCount)]
pub enum BaseClassesNids {
    Warrior = 0,
    Mage,
    Rogue,
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