use bevy::color::palettes::tailwind::{SKY_700, SLATE_50};
use bevy::prelude::*;
use crate::pregame_menus::main_menu::main_menu_components::{MainMenuButton, MainMenuLineEdit};
use crate::pregame_menus::pregame_components::{PreGameScoped};
use crate::ui::ui_components::ButtonBackgroundStyle;
use crate::ui::ui_utils::text_button;
use bevy_ui_text_input::*;


//construir con ButtonBackgroundStyle::default() para usar el estilo por defecto
pub fn main_menu_button<T: Into<String>> (menu_button: MainMenuButton, text: T, style: Option<ButtonBackgroundStyle>) -> impl Bundle {
    (
        text_button(text, None, style),
        menu_button,
        PreGameScoped {},
    )
}


pub fn menu_line_edit<T: Into<String>>(
    menu_line_edit: MainMenuLineEdit,
    placeholder_text: Option<T>,
    bg_color: Option<Color>,
    asset_server: Res<AssetServer>
) -> impl Bundle {
    let default_color: Color = SKY_700.into();
    (
        menu_line_edit,
        BackgroundColor(bg_color.unwrap_or(default_color)),
        Node {
            padding: UiRect::all(Val::Px(5.)),
            width: Val::Px(200.),
            ..default()
        },
        TextInputNode {
            mode: TextInputMode::SingleLine,
            max_chars: Some(36),
            ..Default::default()
        },
        TextFont {
            font: asset_server.load("fonts/.ttf"),
            font_size: 25.,
            ..Default::default()
        },
    )
}
