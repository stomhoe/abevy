#[allow(unused_imports)] use bevy::prelude::*;

#[derive(Component, Debug, Default, )]
pub struct LobbyPlayerListing;


#[derive(Component, Debug, )]
pub struct LobbyPlayerUiNode(pub Entity);

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
pub enum LobbySlider {ChatHistory, Settings}