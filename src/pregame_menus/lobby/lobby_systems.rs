use bevy::prelude::*;

use crate::pregame_menus::lobby::{lobby_components::LobbyButton, lobby_styles::lobby_button};



pub fn setup_for_host(mut commands: Commands){
    
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        },
        children![
            lobby_button(LobbyButton::Start, "Start", None),
            lobby_button(LobbyButton::Leave, "Leave", None),
            lobby_button(LobbyButton::CreateCharacter, "Create character", None),
        ],
    ));
}

pub fn setup_for_client(mut commands: Commands){
    
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        },
        children![
            lobby_button(LobbyButton::Start, "Ready", None),
            lobby_button(LobbyButton::Leave, "Leave", None),
            lobby_button(LobbyButton::CreateCharacter, "Create character", None),
        ],
    ));
}