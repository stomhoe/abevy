#[allow(unused_imports)] use bevy::prelude::*;

#[derive(Component, Debug, Default, )]
pub struct LobbyPlayerListing;


#[derive(Component, Debug, )]
pub struct LobbyPlayerUiNode(pub Entity);