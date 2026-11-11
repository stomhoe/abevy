#[allow(unused_imports)] use {bevy::prelude::*,};
use {crate::character_creation::character_creation_layout::*, game_common::game_common_states::GameSetupScreen, multiplayer_shared::multiplayer_events::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;


mod character_creation_systems;
mod character_creation_layout;
mod character_creation_components;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CharacterCreationSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .init_resource::<character_creation_components::CharacterCreationSelection>()
        .add_systems(OnExit(common::common_states::AppState::StatefulGameSession), character_creation_systems::reset_character_creation)
        .add_systems(Update, character_creation_systems::fill_character_selection_defaults.after(character_creation_systems::character_creation_button_interaction).run_if(in_state(common::common_states::AssetLoading::Finished)).in_set(game_common::game_common::StatefulSessionSystems))
        .add_systems(Update, character_creation_systems::send_character_selection.after(character_creation_systems::fill_character_selection_defaults).run_if(in_state(common::common_states::AssetLoading::Finished)).run_if(resource_changed::<character_creation_components::CharacterCreationSelection>).in_set(game_common::game_common::StatefulSessionSystems))
        .add_systems(Update, (
            character_creation_systems::character_creation_button_interaction.in_set(CharacterCreationSystems),
            rebuild_layout_after_selection_change.in_set(CharacterCreationSystems),
            prepare_sprite_choice_icons.in_set(CharacterCreationSystems),
            character_creation_systems::update_companion_tooltip.in_set(CharacterCreationSystems),
            character_creation_systems::resize_character_creation.in_set(CharacterCreationSystems),
            character_creation_systems::position_character_creation_popups.in_set(CharacterCreationSystems),
            character_creation_systems::scroll_character_creation.run_if(on_message::<bevy::input::mouse::MouseWheel>).in_set(CharacterCreationSystems),
        ))
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
