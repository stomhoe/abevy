
use bevy::{
    prelude::*,
};
use crate::pregame_menus::main_menu::main_menu_layout::*;
use crate::pregame_menus::main_menu::main_menu_sys_comps::*;
use crate::pregame_menus::PreGameState;
use crate::AppState;

mod main_menu_sys_comps;
mod main_menu_layout;


pub struct MainMenuPlugin;
#[allow(unused_parens)]
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(OnEnter(AppState::PreGame), (setup, layout).run_if(in_state(PreGameState::MainMenu)))
            .add_systems(Update, (menu_button_interaction).run_if(in_state(PreGameState::MainMenu)))
        ;
    }
}



