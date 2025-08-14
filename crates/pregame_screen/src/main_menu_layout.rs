use bevy::color::palettes::css::LIGHT_GOLDENROD_YELLOW;
use bevy::prelude::*;
use common::common_states::{AppState, PreGameState};
use bevy_ui_text_input::*;
use ui_shared::{ui_components::{ButtonBackgroundStyle, CurrentText, LineEdit}, ui_functions::text_button};

use crate::main_menu_components::*;

pub fn layout(mut commands: Commands){
    let vbox = commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        },
        StateScoped(PreGameState::MainMenu),
        StateScoped(AppState::NoSession),

        children![
            main_menu_button(MainMenuButton::QuickStart, "Quick start", None),
            main_menu_button(MainMenuButton::Host, "Host", None),
            main_menu_button(MainMenuButton::Join, "Join", None),
        ]
        )).id();

    let line_edit = (
        Node {
            width: Val::Px(270.), height: Val::Px(30.),//dejé espacio para poner el puerto
            ..default()
        },
        LineEdit,
        TextInputNode {
            mode: TextInputMode::SingleLine,
            clear_on_submit: false,
            max_chars: Some(36),
            ..Default::default()
        },
        TextInputPrompt::new("127.0.0.1"),
        CurrentText::new("127.0.0.1"),
        MainMenuIpLineEdit,
        Outline {
            color: LIGHT_GOLDENROD_YELLOW.into(),
            width: Val::Px(2.),
            offset: Val::Px(2.),
        },
        TextFont {font_size: 25., ..Default::default()},
    );
        
    commands.spawn(line_edit).insert(ChildOf(vbox));
}

//construir con ButtonBackgroundStyle::default() para usar el estilo por defecto
pub fn main_menu_button<T: Into<String>> (
    menu_button: MainMenuButton, text: T, style: Option<ButtonBackgroundStyle>) -> impl Bundle {

    let base = (Node {
            padding: UiRect::all(Val::Px(5.)),
            width: Val::Px(200.),
            ..default()
        },);
    (
        menu_button,
        text_button(base , text, (), style),
    )
}