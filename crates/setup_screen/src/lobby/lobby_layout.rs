use bevy::prelude::*;
use common::common_states::{AppState, GamePhase};
use ui_shared::ui_components::*;
use ui_shared::ui_functions::*;

use crate::lobby::lobby_components::*;
use ui_shared::ui_styles::*;


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

fn do_base_layout(commands: &mut Commands, assets: &AssetServer) -> LobbyBaseLayout {
    let vbox_container = commands.spawn((
        LobbyRoot,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(28.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.),
            ..default()
        },
        ImageNode { color: Color::srgb(0.76, 0.75, 0.72), image_mode: NodeImageMode::Stretch, visual_box: VisualBox::BorderBox, ..ImageNode::new(assets.load("apocalypse.jpg")) }, MenuBackground,
        DespawnOnExit(GamePhase::Setup),
        DespawnOnExit(AppState::StatefulGameSession),
    )).id();

    carved_frame(commands, vbox_container, 20.);
    commands.spawn((
        ChildOf(vbox_container),
        retro_caption(assets, "Lobby", 28.),
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            Node { width: Val::Percent(100.), height: Val::Px(30.), justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center, ..default() },
    ));

    let top_hbox_container = commands.spawn((
        ChildOf(vbox_container),
        Node {
            width: Val::Percent(100.),
            height: Val::Px(52.),
            column_gap: Val::Px(10.),
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
            height: Val::Px(0.),
            flex_grow: 1.,
            min_height: Val::Px(0.),
            column_gap: Val::Px(12.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
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
            flex_grow: 1.,
            min_width: Val::Px(0.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(8.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        LineEdit,
        bevy::input_focus::AutoFocus,
        bevy::input_focus::tab_navigation::TabIndex(0),
        bevy::text::EditableText::default(),
        TextLayout::no_wrap(),
        LobbyLineEdit::LobbyName,
        BackgroundColor(INSET), TextColor(TEXT),
        Outline {
            color: Color::NONE,
            width: Val::Px(1.),
            offset: Val::Px(0.),
        },
        TextFont {
            font_size: FontSize::Px(20.),
            ..Default::default()
        },
    )).id();
    carved_frame(commands, lobby_name, 6.);

    let leave_button = commands.spawn((
        ChildOf(top_hbox_container),
        lobby_button(LobbyButtonId::Leave, "Leave",),
    )).id();


    let settings = commands.spawn((
        ChildOf(middlesplitter_hbox),
        Node {
            width: Val::Px(180.),
            min_width: Val::Px(180.),
            max_width: Val::Px(180.),
            min_height: Val::Px(0.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(16.)),
            ..default()
        },
        BackgroundColor(PANEL),
        // ScrollableContent {
        //     ..default()
        // },
    )).id();
    carved_frame(commands, settings, 10.);

    let rightsplit_vbox = commands.spawn((
        ChildOf(middlesplitter_hbox),
        Node {
            width: Val::Px(0.),
            height: Val::Percent(100.),
            flex_grow: 1.,
            min_width: Val::Px(0.),
            min_height: Val::Px(0.),
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
            height: Val::Px(0.),
            flex_grow: 1.,
            min_width: Val::Px(0.),
            min_height: Val::Px(0.),
            column_gap: Val::Px(8.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).id();

    let chat_history = commands.spawn((
        ChildOf(rightsplit_top_hbox),
        Node {
            flex_grow: 1.,
            min_width: Val::Px(0.),
            min_height: Val::Px(0.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(16.)),
            ..default()
        },
        // ScrollableContent {
        //     ..default()
        // },
        BackgroundColor(INSET),
    )).id();
    carved_frame(commands, chat_history, 10.);

    let player_listing = commands.spawn((
        ChildOf(rightsplit_top_hbox),
        LobbyPlayerListing,
        Node {
            width: Val::Px(180.),
            min_width: Val::Px(180.),
            max_width: Val::Px(180.),
            min_height: Val::Px(0.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(16.)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            //row_gap: Val::Px(10.),
            ..default()
        },
        BackgroundColor(PANEL),
    )).id();
    carved_frame(commands, player_listing, 10.);



    let rightsplit_bottom_hbox = commands.spawn((
        ChildOf(rightsplit_vbox),
        Node {
            width: Val::Percent(100.),
            flex_grow: 0.,
            min_width: Val::Px(0.),
            height: Val::Px(52.),
            column_gap: Val::Px(10.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).id();

    let _create_character_button = commands.spawn((
        ChildOf(rightsplit_bottom_hbox),
        lobby_button(LobbyButtonId::CreateCharacter, "Character creation"),
    )).id();

    let chat_input = commands.spawn((
        ChildOf(rightsplit_bottom_hbox),
        Node {
            flex_grow: 1.,
            min_width: Val::Px(0.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(7.)),
            ..default()
        },
        LineEdit,
        bevy::text::EditableText::default(),
        TextLayout::no_wrap(),
        bevy::input_focus::AutoFocus,
        bevy::input_focus::tab_navigation::TabIndex(1),
        LobbyLineEdit::Chat,
        BackgroundColor(INSET), TextColor(TEXT),
        Outline {
            color: Color::NONE,
            width: Val::Px(1.),
            offset: Val::Px(0.),
        },
        TextFont {
            font_size: FontSize::Px(20.),
            ..Default::default()
        },
    )).id();
    carved_frame(commands, chat_input, 6.);

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


pub fn layout_for_host(mut commands: Commands, assets: Res<AssetServer>) {
    // Spawn the lobby UI for the host
    let shared_layout = do_base_layout(&mut commands, &assets);

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

pub fn layout_for_client(mut commands: Commands, assets: Res<AssetServer>) {

    let shared_layout = do_base_layout(&mut commands, &assets);

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

    commands.entity(lobby_name).remove::<bevy::ui_widgets::TextInput>();

    commands.entity(lobby_visibility).remove::<(
        bevy::ui_widgets::Button,
        bevy::ui::Pressed,
        bevy::picking::hover::Hovered,
    )>();

}


pub fn lobby_button<T: Into<String>> (
    lobby_button: LobbyButtonId, text: T) -> impl Bundle {

    let base = (
        Node{
            height: Val::Percent(100.),
            width: Val::Px(180.),
            min_width: Val::Px(180.),
            max_width: Val::Px(180.),
            border: UiRect::all(Val::Px(2.)),
            border_radius: panel_radius(),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    );

    let style = None;
    (
        text_button(base, text, (TextColor(Color::srgb(0.91, 0.8, 0.58)),), style),
        lobby_button,
    )
}
