use bevy::prelude::*;

use crate::pregame_menus::lobby::{lobby_components::LobbyButton, lobby_styles::lobby_button};



pub fn setup_for_host(mut commands: Commands){
    
    println!("Setting up lobby for host");
    let vbox_container = commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        },
    )).id();

    let top_hbox_container = commands.spawn(
    Node {
        width: Val::Percent(100.),
        height: Val::Px(50.),
        min_height: Val::Px(50.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        flex_direction: FlexDirection::Row,
        ..default()
    },).id();

    let lobby_name = commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 1.0, 0.0)),
    )).id();

    let leave_button = commands.spawn((
        Node {
            width: Val::Px(150.),
            height: Val::Percent(100.),
            min_width: Val::Px(150.),
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
    )).id();

    commands.entity(top_hbox_container).add_children(&[
        lobby_name, leave_button,
    ]);



    let middle_hbox_container = commands.spawn(
    Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        flex_direction: FlexDirection::Row,
        ..default()
    },).id();

    let bottom_hbox_container = commands.spawn(
    Node {
        width: Val::Percent(100.),
        height: Val::Px(50.),
        min_height: Val::Px(50.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        flex_direction: FlexDirection::Row,
        ..default()
    },).id();

        let chat_input = commands.spawn((
            Node{
                width: Val::Percent(100.),
                height: Val::Percent(100.),

                ..default()
            },
            BackgroundColor(Color::BLACK)
        )).id();
        let create_character = commands.spawn((
            Node{
                width: Val::Px(150.),
                height: Val::Percent(100.),
                min_width: Val::Px(150.),
                ..default()
            },
            BackgroundColor(Color::srgb(0.0, 0.0, 1.0))
        )).id();
        let start_game = commands.spawn((
            Node{
                width: Val::Px(150.),
                height: Val::Percent(100.),
                min_width: Val::Px(150.),
                ..default()
            },
            BackgroundColor(Color::WHITE)
        )).id();
    commands.entity(bottom_hbox_container).add_children(&[
        chat_input, create_character, start_game,
    ]);


    commands.entity(vbox_container).add_children(&[
        top_hbox_container,
        middle_hbox_container,
        bottom_hbox_container,
    ]);
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
            lobby_button(LobbyButton::Start, "Ready", None),
            lobby_button(LobbyButton::Leave, "Leave", None),
            lobby_button(LobbyButton::CreateCharacter, "Create character", None),
        ],
    ));
}