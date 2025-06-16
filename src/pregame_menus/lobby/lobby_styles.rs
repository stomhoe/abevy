use bevy::color::palettes::tailwind::{SKY_700, SLATE_50};
use bevy::prelude::*;
use crate::pregame_menus::lobby::lobby_components::LobbyButtonId;
use crate::ui::ui_components::ButtonBackgroundStyle;
use crate::ui::ui_utils::text_button;

pub fn lobby_button<T: Into<String>> (
    lobby_button: LobbyButtonId, text: T) -> impl Bundle {
    
    let base = (

        Node{
            height: Val::Percent(100.),
            min_width: Val::Px(120.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    );
    
    let style = None;
    (
        text_button(base, text, (), style),
        lobby_button,
    )
}


