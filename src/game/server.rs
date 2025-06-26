use bevy::prelude::*;

// Module server
mod server_components;
mod server_systems;
//mod server_events;
//mod server_styles;
mod server_resources;
pub struct ServerPlugin;
#[allow(unused_parens, path_statements)]
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}