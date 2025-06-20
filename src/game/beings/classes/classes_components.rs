#[allow(unused_imports)] use bevy::prelude::*;

//esto no va en los beings
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClassNid(pub u32);

//crear una entidad por cada instancia de clase existente
//esto no va en los beings
#[derive(Component, Debug)]
pub struct Class(pub ClassNid);
