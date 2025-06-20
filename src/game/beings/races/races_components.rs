#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug)]
pub struct Race {
    pub nid: u32,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RaceNid(pub u32);