#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Race (u32);

impl Race {
    pub fn new(nid: u32) -> Self {
        Self(nid)
    }
    pub fn nid(&self) -> u32 {self.0}
}


#[derive(Component, Debug)]
pub struct RaceRef(pub Entity);