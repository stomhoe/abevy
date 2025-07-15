use bevy::prelude::*;

use crate::game::setup_menus::{character_creation::CharacterCreationPlugin, lobby::LobbyPlugin};

pub mod character_creation;
pub mod lobby;
pub struct SetupMenusPlugin;
#[allow(unused_parens)]
impl Plugin for SetupMenusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((LobbyPlugin, CharacterCreationPlugin))
        ;
    }
}