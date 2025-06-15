mod main_menu_systems;
mod main_menu_styles;
mod main_menu_components;

use bevy::{
    prelude::*,
};
use crate::pregame_menus::main_menu::main_menu_systems::*;
use crate::pregame_menus::PreGameState;




pub struct MainMenuPlugin;
#[allow(unused_parens)]
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(OnEnter(PreGameState::MainMenu), setup)
            .add_systems(Update, (menu_button_interaction).run_if(in_state(PreGameState::MainMenu)))
        ;
    }
}



