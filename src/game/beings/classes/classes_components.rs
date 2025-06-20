#[allow(unused_imports)] use bevy::prelude::*;
use vec_collections::VecSet;

//esto no va en los beings
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClassNid(pub u32);

//crear una entidad por cada instancia de clase existente
//esto no va en los beings
#[derive(Component, Debug)]
pub struct Class(pub ClassNid);

#[derive(Component, Debug)]
//esto va en los beings, permite tener multiples clases
pub struct ClassesRefs(pub VecSet<[Entity; 3]>);