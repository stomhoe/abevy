use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::game::GamePlugin;
use crate::pregame_menus::MenuPlugin;
use crate::ui::MyUiPlugin;
mod game;
mod pregame_menus;
pub mod ui;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    PreGame, Game, GameOver,
}
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GamePlugin, MenuPlugin, MyUiPlugin))
        .init_state::<AppState>()
        .add_systems(Update, transition_to_game_state)
        .run()
    ;
}

pub fn transition_to_game_state(keyboard_input: Res<ButtonInput<KeyCode>>, current_state: Res<State<AppState>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        match current_state.get() {
            AppState::Game => {
                println!("Transitioning to Menu State");
                next_state.set(AppState::PreGame)
            },
            AppState::PreGame => {
                println!("Transitioning to Game State");
                next_state.set(AppState::Game)
            },
            _ => { panic!("invalid state") }
        }
    }
}


#[derive(Component, Default)]
#[require(Camera2d, StateScoped::<AppState>)]
pub struct StateScopedCamera;




