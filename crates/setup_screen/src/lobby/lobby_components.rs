#[allow(unused_imports)] use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Clone)]
pub struct LobbyPlayerListing;

#[derive(Component, Debug, Clone)]
pub struct LobbyPlayerUiNode(pub Entity);

#[derive(Component, Clone)]
pub enum LobbyButtonId {
  Start,
  Leave,
  CreateCharacter,
  LobbyJoinability,
  Ready,
}

#[derive(Component, Clone)]
pub enum LobbyLineEdit {Chat, LobbyName}

#[derive(Component, Clone)]
pub enum LobbySlider {ChatHistory, Settings}
