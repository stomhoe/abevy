use bevy::prelude::*;

use crate::pregame_menus::PreGameState;

#[derive(Component)]
pub enum MainMenuButton {
    QuickStart,
    Host,
    Join,
}


#[derive(Component)]
pub enum MainMenuLineEdit {
    Ip
}
