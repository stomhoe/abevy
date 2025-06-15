use bevy::{color::palettes::css::LIGHT_GOLDENROD_YELLOW, prelude::*};
use bevy_simple_scroll_view::ScrollableContent;
use bevy_ui_text_input::{TextInputMode, TextInputNode, TextInputPrompt};

use crate::{pregame_menus::{lobby::{lobby_components::{LobbyButtonId, LobbyLineEdit}, lobby_styles::lobby_button}, PreGameState}, ui::ui_components::LineEdit, AppState};



pub fn setup_for_host(mut commands: Commands){
    
    println!("Setting up lobby for host");

    let vbox_container = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        },
        StateScoped(PreGameState::LobbyAsHost),
    );

    let top_hbox_container = (
        Node {
            width: Val::Percent(100.),
            height: Val::Px(50.),
            min_height: Val::Px(50.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );

    let lobby_visibility = (
        
        lobby_button(LobbyButtonId::LobbyJoinability, "Lobby visibility", None),
    );

    let lobby_name = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        LineEdit{},
        TextInputNode {
            mode: TextInputMode::SingleLine,
            max_chars: Some(50),
            clear_on_submit: false,
            unfocus_on_submit: true,
            justification: JustifyText::Center,
            ..Default::default()
        },
        TextInputPrompt::new("Lobby name"),
        LobbyLineEdit::LobbyName,
        Outline {
            color: LIGHT_GOLDENROD_YELLOW.into(),
            width: Val::Px(2.),
            offset: Val::Px(0.),
        },
        TextFont {
            font_size: 25.,
            ..Default::default()
        },
    );

    let leave_button = (
        lobby_button(LobbyButtonId::Leave, "Leave", None),
    );

    let middlesplitter_hbox = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );

    let vbox_container = commands.spawn(vbox_container).id();

    let top_hbox = commands.spawn(top_hbox_container).insert(ChildOf(vbox_container)).id();
    let middlesplitter_hbox = commands.spawn(middlesplitter_hbox).insert(ChildOf(vbox_container)).id();

    commands.spawn(lobby_visibility).insert(ChildOf(top_hbox));
    commands.spawn(lobby_name).insert(ChildOf(top_hbox));
    commands.spawn(leave_button).insert(ChildOf(top_hbox));


    let leftsplit_settings_slider = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ScrollableContent {
            ..default()
        },
    );
    commands.spawn(leftsplit_settings_slider).insert(ChildOf(middlesplitter_hbox));

    let rightsplit_vbox = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        },
    );

    let rightsplit_vbox = commands.spawn(rightsplit_vbox).insert(ChildOf(middlesplitter_hbox)).id();

    let rightsplit_top_hbox = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );

    let rightsplit_top_hbox = commands.spawn(rightsplit_top_hbox).insert(ChildOf(rightsplit_vbox)).id();

    let chat_history_slider = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ScrollableContent {
            ..default()
        },
    );
    let chat_history_slider = commands.spawn(chat_history_slider).insert(ChildOf(rightsplit_top_hbox)).id();

    

    let rightsplit_bottom_hbox = (
        Node {
            width: Val::Percent(100.),
            height: Val::Px(100.),
            min_height: Val::Px(50.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );

    let rightsplit_bottom_hbox = commands.spawn(rightsplit_bottom_hbox).insert(ChildOf(rightsplit_vbox)).id();


    let create_character_button = (
        lobby_button(LobbyButtonId::CreateCharacter, "Create character", None),
    );

    let create_character_button = commands.spawn(create_character_button).insert(ChildOf(rightsplit_bottom_hbox)).id();

    let chat_input = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        LineEdit{},
        TextInputNode {
            mode: TextInputMode::SingleLine,
            //max_chars: Some(36),
            ..Default::default()
        },
        TextInputPrompt::new("Type your message here..."),
        LobbyLineEdit::Chat,
        Outline {
            color: LIGHT_GOLDENROD_YELLOW.into(),
            width: Val::Px(2.),
            offset: Val::Px(0.),
        },
        TextFont {
            font_size: 25.,
            ..Default::default()
        },
    );

    let chat_input = commands.spawn(chat_input).insert(ChildOf(rightsplit_bottom_hbox)).id();
    
    let start_game_button = (

        lobby_button(LobbyButtonId::Start, "Start", None),
    );

    let start_game = commands.spawn(start_game_button).insert(ChildOf(rightsplit_bottom_hbox)).id();



}


    

pub fn setup_for_client(mut commands: Commands){
    
    println!("Setting up lobby for client");
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
            lobby_button(LobbyButtonId::Start, "Ready", None),
            lobby_button(LobbyButtonId::Leave, "Leave", None),
            lobby_button(LobbyButtonId::CreateCharacter, "Create character", None),
        ],
    ));
}

pub fn lobby_button_interaction(
    interaction_query: Query<
    (&Interaction, &LobbyButtonId),
    Changed<Interaction>,
    >,
    //mut app_exit_events: EventWriter<AppExit>,
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut app_state: ResMut<NextState<AppState>>,
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    pregame_state.set(PreGameState::MainMenu)
                }
                LobbyButtonId::Start => todo!(),
                LobbyButtonId::CreateCharacter => todo!(),
                LobbyButtonId::Ready => todo!(),
                LobbyButtonId::LobbyJoinability => {},
            }
        }
    }
}