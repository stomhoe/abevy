#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_ui_gradients::{BorderGradient, ColorStop, LinearGradient, Position, RadialGradient, RadialGradientShape};

use crate::game::GameSetupScreen;

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
            //row_gap: Val::Px(10.),
            ..default()
        },
        ImageNode::new(asset_server.load("apocalypse.jpg")),
        StateScoped(GameSetupScreen::CharacterCreation),
    )).id();

    // Use absolute positioning for overlapping borders
    let center_node = commands.spawn((
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
        BackgroundColor(Color::srgba(0.28, 0.25, 0.17, 0.29)),
    )).id();

    let _top_border = commands.spawn((
        ChildOf(center_node),
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(20.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            top: Val::Px(0.),
            border: UiRect {
                top: Val::Px(20.),
                ..default()
            },
            ..default()
        },
        BorderGradient::from(LinearGradient {
            angle: LinearGradient::TO_BOTTOM,
            stops: vec![
                ColorStop::new(Color::srgba(0.8, 0.8, 0.8, 1.0), Val::Percent(0.)),
                ColorStop::new(Color::srgba(0.36, 0.37, 0.3, 0.0), Val::Px(20.)),
            ],
        }),
    )).id();

    let _left_border = commands.spawn((
        ChildOf(center_node),
        Node {
            width: Val::Px(15.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            top: Val::Px(0.),
            border: UiRect {
                left: Val::Px(15.),
                ..default()
            },
            ..default()
        },
        BorderGradient::from(LinearGradient {
            angle: LinearGradient::TO_RIGHT,
            stops: vec![
                ColorStop::new(Color::srgba(0.8, 0.8, 0.8, 0.5), Val::Percent(0.)),
                ColorStop::new(Color::srgba(0.36, 0.37, 0.3, 0.0), Val::Px(15.)),
            ],
        }),
    )).id();
}
