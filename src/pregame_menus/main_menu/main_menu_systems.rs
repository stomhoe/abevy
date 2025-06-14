use bevy::prelude::*;

use bevy::text::cosmic_text::ttf_parser::Style;
use bevy::{
    color::palettes::css::{GREY, LIGHT_GOLDENROD_YELLOW},
    input_focus::InputFocus,
    platform::collections::HashMap,
    prelude::*,
};
use bevy_ui_text_input::{
    TextInputContents, TextInputMode, TextInputNode, TextInputPlugin, TextInputPrompt, TextSubmitEvent,
};

use crate::pregame_menus::main_menu::main_menu_styles::main_menu_button;
use crate::pregame_menus::pregame_components::PreGameScoped;
use crate::ui::ui_components::ButtonBackgroundStyle;
use crate::ui::ui_resources::InputOutputMap;
use crate::{AppState};
use crate::pregame_menus::main_menu::*;
use crate::pregame_menus::main_menu::main_menu_components::*;



pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut line_edit_map: ResMut<InputOutputMap>) {
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
            main_menu_button(MainMenuButton::QuickStart, "Quick start", None),
            main_menu_button(MainMenuButton::Host, "Host", None),
            main_menu_button(MainMenuButton::Join, "Join", None),
        
        ],
        PreGameScoped{}
    ));
    
    let input_entity = commands
    .spawn((
        TextInputNode {
            mode: TextInputMode::SingleLine,
            ..Default::default()
        },
        TextFont {
            font: asset_server.load("fonts/name_label/PixAntiqua.ttf"),
            font_size: 25.,
            ..Default::default()
        },
        TextInputPrompt::default(),
        TextInputContents::default(),
        Node {
            width: Val::Px(250.),
            height: Val::Px(30.),
            ..default()
        },
        BackgroundColor(bevy::color::palettes::css::NAVY.into()),
    ))
    .id();

    let output_entity = commands.spawn(

        Text::default()
    ).id();
    
    line_edit_map.insert(input_entity, output_entity);

}





pub fn menu_button_interaction(
    interaction_query: Query<
    (&Interaction, &MainMenuButton),
    (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
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


pub fn update(
    input_focus: Res<InputFocus>,
    mut events: EventReader<TextSubmitEvent>,
    map: Res<InputOutputMap>,
    mut text_query: Query<&mut Text>,
    mut outline_query: Query<(Entity, &mut Outline)>,
) {
    if input_focus.is_changed() {
        for (entity, mut outline) in outline_query.iter_mut() {
            if input_focus.0.is_some_and(|active| active == entity) {
                outline.color = Color::WHITE;
            } else {
                outline.color = GREY.into();
            }
        }
    }
    
    for event in events.read() {
        let out = map[&event.entity];
        text_query.get_mut(out).unwrap().0 = event.text.clone();
    }
}