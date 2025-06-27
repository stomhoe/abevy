#[allow(unused_imports)] use bevy::prelude::*;

//use crate::game::beings::classes::classes_systems::*;
use crate::game::beings::classes::classes_components::*;
use crate::game::beings::classes::classes_resources::*;
//use crate::game::beings::classes::classes_events::*;
//mod classes_systems;
mod classes_components;
mod classes_resources;
//mod classes_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClassesSystems;

pub struct ClassesPlugin;
#[allow(unused_parens)]
impl Plugin for ClassesPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem).in_set(ClassesSystems).in_set(IngameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
        ;
    }
}