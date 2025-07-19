#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use superstate::SuperstateInfo;

use crate::game::being::sprite::animation_constants::*;


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

#[derive(Component, Debug,)]
pub struct Bullet();

#[derive(Component, Debug,)]
pub struct Health(pub i32,);//SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

#[derive(Component, Debug,)]
pub struct PhysicallyImmune();

#[derive(Component, Debug,)]
pub struct MagicallyInvulnerable();

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ImgPathHolder(pub String);


#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayName(pub String);
impl DisplayName {pub fn new(name: impl Into<String>) -> Self {DisplayName(name.into())}}

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Description(pub String);
impl Description {
    pub fn new<S: Into<String>>(id: S) -> Self {Self (id.into())}
    pub fn id(&self) -> &String {&self.0}
}

#[derive(Component)]
pub enum Direction{Down, Left, Right, Up,}
impl Direction {
    pub fn as_suffix(&self) -> &str {
        match self {
            Direction::Down => DOWN, Direction::Left => LEFT,
            Direction::Right => RIGHT, Direction::Up => UP,
        }
    }
}