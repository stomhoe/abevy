use bevy::color::palettes::tailwind::{SKY_700, SLATE_50};
use bevy::prelude::*;
use crate::pregame_menus::main_menu::main_menu_components::{MainMenuButton, MainMenuLineEdit};
use crate::pregame_menus::pregame_components::{PreGameScoped};
use crate::ui::ui_components::ButtonBackgroundStyle;
use crate::ui::ui_utils::{button, text_button};
use bevy_ui_text_input::*;


//construir con ButtonBackgroundStyle::default() para usar el estilo por defecto
pub fn main_menu_button<T: Into<String>> (
    menu_button: MainMenuButton, text: T, style: Option<ButtonBackgroundStyle>) -> impl Bundle {

    let base = Node {
            padding: UiRect::all(Val::Px(5.)),
            width: Val::Px(200.),
            ..default()
    };
    (
        menu_button,
        text_button(base , text, (), style),
    )
}


