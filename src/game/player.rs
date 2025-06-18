use bevy::prelude::*;

// Module player
pub mod player_components;
mod player_systems;
//pub mod player_resources;
//mod player_events;
//mod player_styles;
pub struct PlayerPlugin;
#[allow(unused_parens)]
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
enum PlayerState {
    #[default]
    Controlling,
    Second,
    Third,
}