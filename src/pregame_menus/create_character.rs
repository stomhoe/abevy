use bevy::prelude::*;

// Module create_character
mod create_character_components;
mod create_character_systems;
//mod create_character_events;
mod create_character_layout;
//mod create_character_resources;
pub struct CreateCharacterPlugin;
#[allow(unused_parens, path_statements)]
impl Plugin for CreateCharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}