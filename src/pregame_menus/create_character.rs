use bevy::prelude::*;

// Module create_character
mod create_character_components;
mod create_character_systems;
//mod create_character_events;
mod create_character_styles;
//mod create_character_resources;
pub struct create_characterPlugin;
#[allow(unused_parens)]
impl Plugin for create_characterPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}