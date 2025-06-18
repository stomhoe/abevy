use bevy::prelude::*;

// Module being
mod beings_components;
mod beings_systems;
//mod being_events;
mod beings_resources;
pub struct BeingPlugin;
#[allow(unused_parens)]
impl Plugin for BeingPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}