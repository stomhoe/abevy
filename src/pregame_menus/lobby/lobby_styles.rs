use bevy::color::palettes::tailwind::{SKY_700, SLATE_50};
use bevy::prelude::*;
use crate::pregame_menus::lobby::lobby_components::LobbyButton;


pub fn lobby_button<T: Into<String>> (menu_button_action: LobbyButton, text: T, bg_color: Option<Color>,) -> impl Bundle {
    let default_color: Color = SKY_700.into();
    (
        Button,
        BackgroundColor(bg_color.unwrap_or(default_color)),
        Node {
            padding: UiRect::all(Val::Px(5.)),
            width: Val::Px(200.),
            ..default()
        },
        children![(
            Text::new(text),
            TextColor(SLATE_50.into())
        )],
        menu_button_action,
    )
}


