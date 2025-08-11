#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};


//crear una entidad por cada instancia de clase existente
//esto no va en los beings
#[derive(Component, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[require(Replicated)]
pub struct Class;


#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default, Reflect)]
//esto va en los beings, permite tener multiples clases
pub struct ClassesRefs(#[entities] Vec<Entity>);

impl ClassesRefs {
    pub fn new<I>(classes: I) -> Self 
    where 
        I: IntoIterator, 
        I::Item: Into<Entity>, 
    {
        let iter = classes.into_iter();
        let mut vec = Vec::with_capacity(3);
        vec.extend(iter.map(Into::into));
        Self(vec)
    }
    pub fn classes(&self) -> &[Entity] { &self.0 }
    pub fn classes_mut(&mut self) -> &mut Vec<Entity> { &mut self.0 }
}
