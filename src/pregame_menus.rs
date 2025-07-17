use crate::{pregame_menus::{pregame_systems::*}, AppState};
use bevy::prelude::*;
use crate::pregame_menus::main_menu::MainMenuPlugin;


pub mod main_menu;
mod pregame_systems;
//mod pregame_components;


#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::PreGame)]
#[states(scoped_entities)]
pub enum PreGameState {
    #[default]
    MainMenu,
    Settings
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<PreGameState>()
            .add_systems(Startup, setup)
            .add_plugins(MainMenuPlugin)
        ;
    }
}

