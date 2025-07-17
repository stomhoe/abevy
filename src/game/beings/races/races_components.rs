#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, PartialEq, Eq, Hash, Clone)]
#[require(Replicated)]
pub struct Race (u32);

impl Race {
    pub fn new(nid: u32) -> Self {
        Self(nid)
    }
    pub fn nid(&self) -> u32 {self.0}
}


#[derive(Component, Debug)]
pub struct RaceRef(pub Entity);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct AvailableHeads(Vec<String>);



