use bevy::prelude::*;

// Module formation_generation
//mod formation_generation_components;
//mod formation_generation_systems;
//mod formation_generation_events;
//mod formation_generation_styles;
//mod formation_generation_resources;
pub struct FormationGenerationPlugin;
#[allow(unused_parens)]
impl Plugin for FormationGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}