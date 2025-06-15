use bevy::prelude::*;

use crate::pregame_menus::PreGameState;


#[derive(Component)]
#[require(StateScoped::<PreGameState>)]
pub enum LobbyButton {
  Start,
  Leave,
  CreateCharacter,
  Ready,
}

