use crate::{pregame_menus::pregame_systems::*, AppState};
use bevy::prelude::*;
use crate::pregame_menus::main_menu::MainMenuPlugin;


mod main_menu;
mod pregame_systems;
mod lobby;
mod pregame_components;

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::PreGame)]
#[states(scoped_entities)]
enum PreGameState {
    #[default]
    MainMenu,
    LobbyAsHost,
    LobbyAsClient,
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_plugins(MainMenuPlugin)
        ;
    }
}




