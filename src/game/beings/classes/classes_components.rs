#[allow(unused_imports)] use bevy::prelude::*;
use vec_collections::VecSet;


//crear una entidad por cada instancia de clase existente
//esto no va en los beings
#[derive(Component, Debug, PartialEq, Eq, Hash)]
pub struct Class(u32);

impl Class {
    pub fn new(nid: u32) -> Self {
        Self(nid)
    }
    pub fn nid(&self) -> u32 {self.0}
}

#[derive(Component, Debug)]
//esto va en los beings, permite tener multiples clases
pub struct ClassesRefs(pub VecSet<[Entity; 3]>);