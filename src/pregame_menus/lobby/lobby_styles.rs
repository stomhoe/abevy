use bevy::color::palettes::tailwind::{SKY_700, SLATE_50};
use bevy::prelude::*;
use crate::pregame_menus::lobby::lobby_components::LobbyButton;
use crate::pregame_menus::pregame_components::PreGameScoped;
use crate::ui::ui_components::ButtonBackgroundStyle;
use crate::ui::ui_utils::text_button;

pub fn lobby_button<T: Into<String>> (
    lobby_button: LobbyButton, text: T, style: Option<ButtonBackgroundStyle>,) -> impl Bundle {
    
    let base = Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        ..default()
    };
    (
        text_button(base, text, (), style),
        lobby_button,
    )
}


