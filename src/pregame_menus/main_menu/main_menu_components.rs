use bevy::prelude::*;

use crate::pregame_menus::PreGameState;

#[derive(Component)]
#[require(StateScoped::<PreGameState>)]
pub enum MainMenuButton {
    QuickStart,
    Host,
    Join,
}


#[derive(Component)]
#[require(StateScoped::<PreGameState>)]
pub enum MainMenuLineEdit {
    Ip
}
