use bevy::prelude::*;

use bevy::{
    color::palettes::css::{LIGHT_GOLDENROD_YELLOW},
};
use bevy_ui_text_input::{TextInputMode, TextInputNode, TextInputPrompt, TextSubmitEvent,
};

use crate::pregame_menus::main_menu::main_menu_styles::main_menu_button;
use crate::ui::ui_components::{LineEdit};
use crate::{AppState};
use crate::pregame_menus::main_menu::*;
use crate::pregame_menus::main_menu::main_menu_components::*;


pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let line_edit = (
        Node {
            width: Val::Px(250.),
            height: Val::Px(30.),
            ..default()
        },
        LineEdit{},
        TextInputNode {
            mode: TextInputMode::SingleLine,
            max_chars: Some(36),
            ..Default::default()
        },
        TextInputPrompt::new("Enter IP address"),
        MainMenuLineEdit::Ip,
        Outline {
            color: LIGHT_GOLDENROD_YELLOW.into(),
            width: Val::Px(2.),
            offset: Val::Px(2.),
        },
        TextFont {
            //font: asset_server.load("fonts/.ttf"),
            font_size: 25.,
            ..Default::default()
        },
    );

    let parent = commands.spawn((
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
            main_menu_button(MainMenuButton::QuickStart, "Quick start", None),
            main_menu_button(MainMenuButton::Host, "Host", None),
            main_menu_button(MainMenuButton::Join, "Join", None),
            ]
        )).id();
        
    commands.spawn(line_edit).insert(ChildOf(parent));
        
}

pub fn menu_button_interaction(
    interaction_query: Query<
    (&Interaction, &MainMenuButton),
    Changed<Interaction>,
    >,
    //mut app_exit_events: EventWriter<AppExit>,
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MainMenuButton::QuickStart => {
                    app_state.set(AppState::Game)
                }
                MainMenuButton::Host => {
                    pregame_state.set(PreGameState::LobbyAsHost);
                }
                MainMenuButton::Join => {
                    pregame_state.set(PreGameState::LobbyAsClient)
                }
            } 
        }
    }
}

pub fn handle_line_edits_interaction(
    mut events: EventReader<TextSubmitEvent>,
    line_edit_query: Query<&MainMenuLineEdit>,
) {
    for event in events.read() {
        let entity = event.entity;
        if let Ok(line_edit_type) = line_edit_query.get(entity) {
            match line_edit_type {
                MainMenuLineEdit::Ip => {
                    if event.text.len() > 15 {
                        continue;
                    }
                }
            }
        }
    }
}