use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::game::GamePlugin;
use crate::pregame_menus::{MenuPlugin, PreGameState};
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

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum MpStatus {
    #[default]
    Host, 
    Client
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GamePlugin, MenuPlugin, MyUiPlugin))
        .init_state::<AppState>()
        .init_state::<MpStatus>()
        //.add_systems(Startup, first_state)
        .run()
    ;
}


#[derive(Component, Default)]
#[require(Camera2d, StateScoped::<AppState>)]
pub struct StateScopedCamera;




