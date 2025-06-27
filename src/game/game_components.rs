#[allow(unused_imports)] use bevy::prelude::*;


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
