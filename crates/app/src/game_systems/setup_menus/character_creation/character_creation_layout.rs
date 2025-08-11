#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_ui_gradients::ColorStop;
use bevy_ui_text_input::{TextInputMode, TextInputNode, TextInputPrompt};

use crate::{game::GameSetupScreen, ui::{ui_components::{CurrentText, LineEdit}, ui_utils::{produce_gradient_border, BorderBundle}}};

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn do_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    let base_node = commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ImageNode::new(asset_server.load("apocalypse.jpg")),
        StateScoped(GameSetupScreen::CharacterCreation),
    )).id();
    
    let borders = produce_grayish_border(18.0);
    // Use absolute positioning for overlapping borders
    let center_rect_node = commands.spawn((
        ChildOf(base_node),
        Node {
            width: Val::Percent(74.0),
            height: Val::Percent(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_width: Val::Px(1024.),
            min_height: Val::Px(768.),
            position_type: PositionType::Relative,
            ..default()
        },
        children![
            borders[0].clone(),
            borders[1].clone(),
            borders[2].clone(),
            borders[3].clone(),
        ],
        BackgroundColor(Color::srgba(0.28, 0.25, 0.17, 0.29)),
    )).id();


    // NOTE: If right margin is visually smaller, it's likely due to child content stretching or alignment.
    // To ensure margins are respected, set align_items and justify_content to FlexStart.
    // Also, ensure no child node is overflowing or using 100% width with its own margins.
    let vbox_container = commands.spawn((
        ChildOf(center_rect_node),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            min_width: Val::Px(500.),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(9.),
            column_gap: Val::Px(9.),
            margin: UiRect {
                left: Val::Px(20.),
                top: Val::Px(30.),
                right: Val::Px(20.),
                bottom: Val::Px(30.),
            },
            ..default()
        },
    )).id();

    //add text node
    let screen_title = commands.spawn((
        ChildOf(vbox_container),
        Node {
            width: Val::Percent(100.),
            height: Val::Px(50.),
            //align_self: AlignSelf::FlexStart,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Text::new("Character Creation"),
        TextFont {
            font_size: 49.,
            font: asset_server.load("fonts/Exocet2.ttf"),
            ..default()
        },
        TextColor(Color::srgb(0.13, 0.13, 0.13)),
        TextShadow{
            color: Color::srgb(0.99, 0.1, 0.1).into(),
            offset: Vec2::new(0.0, 0.0),
        },
        TextLayout::new_with_justify(JustifyText::Center)
    )).id();

    let lineedit_control = commands.spawn((
        ChildOf(vbox_container),
        Node {
            width: Val::Percent(100.),
            min_width: Val::Px(420.),
            min_height: Val::Px(42.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            //align_self: AlignSelf::FlexStart,
            ..default()
        },
    )).id();


    let charname_box = commands.spawn((
        ChildOf(lineedit_control),
        Node {
            width: Val::Percent(40.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0., 0., 0., 0.68)),
    )).id();

    let charname_font_size = 25.0; 

    let colorstops = vec![
        ColorStop::new(Color::srgba(0.46, 0.0, 0.0, 0.9), Val::Percent(0.)),
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Px(7.)),
    ];

    let borders = produce_gradient_border(7., 
        colorstops.clone(), colorstops
    );

    let _charname_lineedit = commands.spawn((
        ChildOf(charname_box),
        Node {
            width: Val::Percent(100.),
            height: Val::Px(charname_font_size + 6.0),
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        TextInputPrompt::new("Name"),
        TextInputNode {
            mode: TextInputMode::SingleLine,
            clear_on_submit: false,
            unfocus_on_submit: true,
            max_chars: Some(28),
            justification: JustifyText::Center,
            ..Default::default()
        },
        LineEdit,
        TextFont {
            font_size: charname_font_size,
            font: asset_server.load("fonts/Exocet2.ttf"),//cambiarla despues proq no se distinguen mayus de minus
            ..default()
        },
        TextColor(Color::srgba(0.72, 0.71, 0.43, 1.)),
        TextLayout::new_with_justify(JustifyText::Center),
        CurrentText::new(""),
    )).id();

    

    let hbox_container = commands.spawn((
        ChildOf(vbox_container),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Row,
            row_gap: Val::Px(20.),
            column_gap: Val::Px(20.),
            // margin: UiRect {
            //     left: Val::Px(0.),
            //     right: Val::Px(0.),
            //     top: Val::Px(5.),
            //     bottom: Val::Px(0.),
            // },
            ..default()
        },
    )).id();

    // Helper to spawn a vbox under hbox_container
    let spawn_vbox = |commands: &mut Commands| {
        commands.spawn((
            ChildOf(hbox_container),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::FlexStart,
                column_gap: Val::Px(20.),
                row_gap: Val::Px(20.),
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )).id()
    };

    let left_vbox = spawn_vbox(&mut commands);
    let center_vbox = spawn_vbox(&mut commands);
    let right_vbox = spawn_vbox(&mut commands);

    let squares_vbox_margin = UiRect {
        left: Val::Px(15.), top: Val::Px(15.),
        right: Val::Px(15.), bottom: Val::Px(10.),
    };
    
    let borders = produce_grayish_border(8.0,);

    let make_squares_node = || (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            borders[0].clone(),
            borders[1].clone(),
            borders[2].clone(),
            borders[3].clone(),
        ]
    );

    // Helper closure to spawn a square node under a given parent
    let spawn_square = |commands: &mut Commands, parent: Entity| {
        commands.spawn((
            ChildOf(parent),
            make_squares_node()
        )).id()
    };

    let top_square_left = spawn_square(&mut commands, left_vbox);
    let bottom_square_left = spawn_square(&mut commands, left_vbox);

    let top_square_center = spawn_square(&mut commands, center_vbox);
    let bottom_square_center = spawn_square(&mut commands, center_vbox);

    let top_square_right = spawn_square(&mut commands, right_vbox);
    let bottom_square_right = spawn_square(&mut commands, right_vbox);


    let _borders = produce_grayish_border(8.0);
}

pub fn produce_grayish_border(
    thickness: f32,
) -> [BorderBundle; 4] {
    produce_gradient_border(
        thickness, 
        vec![
            ColorStop::new(Color::srgba(0.8, 0.8, 0.8, 0.10), Val::Percent(0.)),
            ColorStop::new(Color::srgba(0.36, 0.37, 0.3, 0.0), Val::Px(thickness)),
        ],
        vec![
            ColorStop::new(Color::srgba(0., 0., 0., 0.6), Val::Percent(0.)),
            ColorStop::new(Color::NONE, Val::Px(thickness)),
        ]
    )
}