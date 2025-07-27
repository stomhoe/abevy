#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::game::{GameSetupScreen, ActiveGameSystems};
use crate::game::setup_menus::character_creation::{
    character_creation_systems::*,
    character_creation_components::*, 
    character_creation_events::*, 
    character_creation_layout::*,
    //character_creation_resources::*,
};
mod character_creation_systems;
pub mod character_creation_components;
//mod character_creation_resources;
mod character_creation_events;
mod character_creation_layout;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CharacterCreationSystems;

pub struct CharacterCreationPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for CharacterCreationPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(CharacterCreationSystems))
            .add_systems(OnEnter(GameSetupScreen::CharacterCreation), (do_layout, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
            .add_client_trigger::<NameSelected>(Channel::Ordered)
            .add_client_trigger::<RaceSelected>(Channel::Ordered)
            .add_client_trigger::<HeadSelected>(Channel::Ordered)
            .add_client_trigger::<ClassSelected>(Channel::Ordered)
            .add_client_trigger::<FollowerSelected>(Channel::Ordered)
        ;
    }
}