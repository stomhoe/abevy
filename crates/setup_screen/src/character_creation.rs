#[allow(unused_imports)] use {bevy::prelude::*,};
use {crate::character_creation::character_creation_layout::do_layout, game_common::game_common_states::GameSetupScreen, multiplayer_shared::multiplayer_events::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;


mod character_creation_systems;
mod character_creation_layout;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CharacterCreationSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        //.add_systems(Update, (somesystem, ).in_set(CharacterCreationSystems))
        .add_systems(OnEnter(GameSetupScreen::CharacterCreation), (do_layout, ))
        //.init_resource::<RESOURCE_NAME>()
        .add_plugins((
        // SomePlugin, 
        ))
        .add_client_message::<NameSelected>(Channel::Ordered)
        .add_client_message::<RaceSelected>(Channel::Ordered)
        .add_client_message::<HeadSelected>(Channel::Ordered)
        .add_client_message::<ClassSelected>(Channel::Ordered)
        .add_client_message::<FollowerSelected>(Channel::Ordered)
    ;
}