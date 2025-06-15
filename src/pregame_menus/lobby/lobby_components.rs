use bevy::prelude::*;

use crate::pregame_menus::PreGameState;


#[derive(Component)]
pub enum LobbyButtonId {
  Start,
  Leave,
  CreateCharacter,
  LobbyJoinability,
  Ready,
}


#[derive(Component)]
pub enum LobbyLineEdit {Chat, LobbyName}




#[derive(Component)]
pub enum LobbySlider {
    ChatHistory,
    Settings
}