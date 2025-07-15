use bevy::prelude::*;
use bevy::{color::palettes::css::LIGHT_GOLDENROD_YELLOW};
use bevy_simple_scroll_view::ScrollableContent;
use bevy_ui_text_input::{TextInputMode, TextInputNode, TextInputPrompt};

use crate::game::setup_menus::lobby::lobby_systems::*;
use crate::game::{GamePhase};
use crate::ui::ui_components::LineEdit;
use crate::ui::ui_utils::text_button;
use crate::AppState;

struct LobbyBaseLayout{
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
fn do_base_layout(commands: &mut Commands) -> LobbyBaseLayout {
    let vbox_container = commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            //row_gap: Val::Px(10.),
            ..default()
        },
        StateScoped(GamePhase::Setup),
        StateScoped(AppState::GameSession),
    )).id();

    let top_hbox_container = commands.spawn((
        ChildOf(vbox_container),
        Node {
            width: Val::Percent(100.),
            min_height: Val::Px(50.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).id();

    let middlesplitter_hbox = commands.spawn((
        ChildOf(vbox_container),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).id();

    let lobby_visibility = commands
        .spawn((
            ChildOf(top_hbox_container),
            lobby_button(LobbyButtonId::LobbyJoinability, "Lobby visibility"),
        ))
        .id();


    let lobby_name = commands.spawn((
        ChildOf(top_hbox_container),
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
    )).id();

    let leave_button = commands.spawn((
        ChildOf(top_hbox_container),
        lobby_button(LobbyButtonId::Leave, "Leave",),
    )).id();


    let _leftsplit_settings_slider = commands.spawn((
        ChildOf(middlesplitter_hbox),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ScrollableContent {
            ..default()
        },
    )).id();

    let rightsplit_vbox = commands.spawn((
        ChildOf(middlesplitter_hbox),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            //row_gap: Val::Px(10.),
            ..default()
        },
    )).id();

    let rightsplit_top_hbox = commands.spawn((
        ChildOf(rightsplit_vbox),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).id();

    let _chat_history_slider = commands.spawn((
        ChildOf(rightsplit_top_hbox),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ScrollableContent {
            ..default()
        },
    )).id();

    let rightsplit_bottom_hbox = commands.spawn((
        ChildOf(rightsplit_vbox),
        Node {
            width: Val::Percent(100.),
            min_height: Val::Px(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).id();

    let _create_character_button = commands.spawn((
        ChildOf(rightsplit_bottom_hbox),
        lobby_button(LobbyButtonId::CreateCharacter, "Create character"),
    )).id();

    let _chat_input = commands.spawn((
        ChildOf(rightsplit_bottom_hbox),
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
    )).id();
    
    LobbyBaseLayout {
        vbox_container,
        top_hbox: top_hbox_container,
        middlesplitter_hbox,
        rightsplit_vbox,
        rightsplit_top_hbox,
        rightsplit_bottom_hbox,
        lobby_visibility,
        lobby_name,
        leave_button,
    }
}


pub fn layout_for_host(mut commands: Commands) {
    let shared_layout = do_base_layout(&mut commands);
    
    let _vbox_container = shared_layout.vbox_container;
    let _top_hbox = shared_layout.top_hbox;
    let _middlesplitter_hbox = shared_layout.middlesplitter_hbox;
    let _rightsplit_vbox = shared_layout.rightsplit_vbox;
    let _rightsplit_top_hbox = shared_layout.rightsplit_top_hbox;
    let rightsplit_bottom_hbox = shared_layout.rightsplit_bottom_hbox;
    let _lobby_visibility = shared_layout.lobby_visibility;
    let _lobby_name = shared_layout.lobby_name;
    let _leave_button = shared_layout.leave_button;

   
    let start_game_button = (

        lobby_button(LobbyButtonId::Start, "Start",),
    );

    commands.spawn(start_game_button).insert(ChildOf(rightsplit_bottom_hbox));

}

pub fn layout_for_client(mut commands: Commands) {

    let shared_layout = do_base_layout(&mut commands);

    let _vbox_container = shared_layout.vbox_container;
    let _top_hbox = shared_layout.top_hbox;
    let _middlesplitter_hbox = shared_layout.middlesplitter_hbox;
    let _rightsplit_vbox = shared_layout.rightsplit_vbox;
    let _rightsplit_top_hbox = shared_layout.rightsplit_top_hbox;
    let rightsplit_bottom_hbox = shared_layout.rightsplit_bottom_hbox;
    let lobby_visibility = shared_layout.lobby_visibility;
    let lobby_name = shared_layout.lobby_name;
    let _leave_button = shared_layout.leave_button;

    let ready_button = (
        lobby_button(LobbyButtonId::Ready, "Ready",),
    );

    commands.spawn(ready_button).insert(ChildOf(rightsplit_bottom_hbox));

    commands.entity(lobby_name).remove::<(TextInputNode, TextInputPrompt)>();

    commands.entity(lobby_visibility).remove::<(Button, Interaction)>();

}


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


