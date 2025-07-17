#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, )]
pub struct SourceDest{
    pub source: Entity,
    pub destination: Entity,
}



#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Nid(u64);

impl Nid {
    pub fn new(nid: u64) -> Self {Self(nid)}
    pub fn nid(&self) -> u64 { self.0 }
}




//es una entity
#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct SpriteData;


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffSetLookingDown;


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffSetLookingUp();



#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffsetLookingSideways();





#[derive(Component, Debug,)]
pub struct Bullet();

#[derive(Component, Debug,)]
pub struct Health(pub i32,);//SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

#[derive(Component, Debug,)]
pub struct PhysicallyImmune();

#[derive(Component, Debug,)]
pub struct MagicallyInvulnerable();



