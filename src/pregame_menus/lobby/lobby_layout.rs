use bevy::prelude::*;
use bevy::{color::palettes::css::LIGHT_GOLDENROD_YELLOW};
use bevy_simple_scroll_view::ScrollableContent;
use bevy_ui_text_input::{TextInputMode, TextInputNode, TextInputPrompt};

use crate::pregame_menus::lobby::lobby_components::LobbyLineEdit;
use crate::pregame_menus::{lobby::{lobby_components::LobbyButtonId, lobby_styles::*}, PreGameState};
use crate::ui::ui_components::LineEdit;
use crate::AppState;

pub struct SharedLayout{
    pub vbox_container: Entity,
    pub top_hbox: Entity,
    pub middlesplitter_hbox: Entity,
    pub rightsplit_vbox: Entity,
    pub rightsplit_top_hbox: Entity,
    pub rightsplit_bottom_hbox: Entity,
    pub lobby_visibility: Entity,
    pub lobby_name: Entity,
    pub leave_button: Entity,
}
pub fn do_shared_layout(commands: &mut Commands) -> SharedLayout {
    let vbox_container = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            //row_gap: Val::Px(10.),
            ..default()
        },
        StateScoped(PreGameState::Lobby),
        StateScoped(AppState::PreGame),
    );

    let top_hbox_container = (
        Node {
            width: Val::Percent(100.),
            min_height: Val::Px(50.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );

    let lobby_visibility = (
        lobby_button(LobbyButtonId::LobbyJoinability, "Lobby visibility",),
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
        lobby_button(LobbyButtonId::Leave, "Leave",),
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

    let lobby_visibility = commands.spawn(lobby_visibility).insert(ChildOf(top_hbox)).id();
    let lobby_name = commands.spawn(lobby_name).insert(ChildOf(top_hbox)).id();
    let leave_button = commands.spawn(leave_button).insert(ChildOf(top_hbox)).id();


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
            //row_gap: Val::Px(10.),
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
            min_height: Val::Px(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );

    let rightsplit_bottom_hbox = commands.spawn(rightsplit_bottom_hbox).insert(ChildOf(rightsplit_vbox)).id();

    let create_character_button = (
        lobby_button(LobbyButtonId::CreateCharacter, "Create character",),
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
    
    SharedLayout {
        vbox_container,
        top_hbox,
        middlesplitter_hbox,
        rightsplit_vbox,
        rightsplit_top_hbox,
        rightsplit_bottom_hbox,
        lobby_visibility,
        lobby_name,
        leave_button,
    }
}


pub fn layout_for_host(commands: &mut Commands, shared_layout: &SharedLayout) {
    
    let vbox_container = shared_layout.vbox_container;
    let top_hbox = shared_layout.top_hbox;
    let middlesplitter_hbox = shared_layout.middlesplitter_hbox;
    let rightsplit_vbox = shared_layout.rightsplit_vbox;
    let rightsplit_top_hbox = shared_layout.rightsplit_top_hbox;
    let rightsplit_bottom_hbox = shared_layout.rightsplit_bottom_hbox;
    let lobby_visibility = shared_layout.lobby_visibility;
    let lobby_name = shared_layout.lobby_name;
    let leave_button = shared_layout.leave_button;

   
    let start_game_button = (

        lobby_button(LobbyButtonId::Start, "Start",),
    );

    commands.spawn(start_game_button).insert(ChildOf(rightsplit_bottom_hbox));

}

pub fn layout_for_client(commands: &mut Commands, shared_layout: &SharedLayout) {

    let vbox_container = shared_layout.vbox_container;
    let top_hbox = shared_layout.top_hbox;
    let middlesplitter_hbox = shared_layout.middlesplitter_hbox;
    let rightsplit_vbox = shared_layout.rightsplit_vbox;
    let rightsplit_top_hbox = shared_layout.rightsplit_top_hbox;
    let rightsplit_bottom_hbox = shared_layout.rightsplit_bottom_hbox;
    let lobby_visibility = shared_layout.lobby_visibility;
    let lobby_name = shared_layout.lobby_name;
    let leave_button = shared_layout.leave_button;

    let ready_button = (
        lobby_button(LobbyButtonId::Ready, "Ready",),
    );

    commands.spawn(ready_button).insert(ChildOf(rightsplit_bottom_hbox));

    commands.entity(lobby_name).remove::<(TextInputNode, TextInputPrompt)>();

    commands.entity(lobby_visibility).remove::<(Button, Interaction)>();

}