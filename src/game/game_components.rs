use std::default;

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use superstate::SuperstateInfo;
use rand::Rng;

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


#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, )]
pub struct DisplayName(pub String);
impl DisplayName {pub fn new(name: impl Into<String>) -> Self {DisplayName(name.into())}}

#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Description(pub String);
impl Description {
    pub fn new<S: Into<String>>(id: S) -> Self {Self (id.into())}
    pub fn id(&self) -> &String {&self.0}
}
#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Deserialize, Serialize, )]
pub enum FacingDirection { Down, Left, Right, Up, }

impl FacingDirection {
    pub fn as_suffix(&self) -> &str {
        match self {
            FacingDirection::Down => DOWN,
            FacingDirection::Left => LEFT,
            FacingDirection::Right => RIGHT,
            FacingDirection::Up => UP,
        }
    }
}

impl std::fmt::Display for FacingDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FacingDirection::Down => "down",
            FacingDirection::Left => "left",
            FacingDirection::Right => "right",
            FacingDirection::Up => "up",
        };
        write!(f, "{}", s)
    }
}
impl FacingDirection {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..4) {
            0 => FacingDirection::Down,
            1 => FacingDirection::Left,
            2 => FacingDirection::Right,
            _ => FacingDirection::Up,
        }
    }
}

impl Default for FacingDirection {
    fn default() -> Self {
        FacingDirection::random()
    }
}