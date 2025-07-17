use bevy::{ecs::bundle, prelude::*};
use bevy_ui_gradients::{BorderGradient, LinearGradient};

use crate::ui::{ui_components::ButtonBackgroundStyle};

//TODO LA FUENTE
pub fn text_button<T: Into<String>>(base: impl Bundle, text: T, other_children: impl Bundle, style: Option<ButtonBackgroundStyle>,) -> impl Bundle 
{

    let implicit_children = (
        TextLayout::new_with_justify(JustifyText::Center),
        BackgroundColor(Color::srgb(0.99, 0.1, 0.1)),
    );

    button(base, (Text::new(text), other_children, implicit_children), style)
}

pub fn button(
    base: impl Bundle,
    children: impl Bundle,
    style: Option<ButtonBackgroundStyle>,
) -> impl Bundle {
    (
        base,
        Button,
        BackgroundColor(style.unwrap_or_default().normal()),
        children![children],
    )
}


pub const fn color_from_triplicate(value: f32) -> Color {
    Color::srgb(value, value, value)
}

#[derive(Bundle, Clone, Debug)]
pub struct BorderBundle{
    pub node: Node,
    pub border_gradient: BorderGradient,
}
impl BorderBundle {
    pub fn new(
        node: Node,
        gradient: LinearGradient,
    ) -> Self {
        BorderBundle {
            node,
            border_gradient: BorderGradient::from(gradient),
        }
    }
}
pub fn produce_gradient_border(
    thickness: f32, 
    stops_tl: Vec<bevy_ui_gradients::ColorStop>,
    stops_br: Vec<bevy_ui_gradients::ColorStop>,
) -> [BorderBundle; 4]
{
    let left = BorderBundle::new(
        Node {
            width: Val::Px(thickness),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            top: Val::Px(0.),
            border: UiRect {
                left: Val::Px(thickness),
                ..default()
            },
            ..default()
        },
        LinearGradient {
            angle: LinearGradient::TO_RIGHT,
            stops: stops_tl.clone(),
        },
    );

    let top = BorderBundle::new(
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(thickness),
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            top: Val::Px(0.),
            border: UiRect {
                top: Val::Px(thickness),
                ..default()
            },
            ..default()
        },
        LinearGradient {
            angle: LinearGradient::TO_BOTTOM,
            stops: stops_tl.clone(),
        },
    );

    let right = BorderBundle::new(
        Node {
            width: Val::Px(thickness),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            right: Val::Px(0.),
            top: Val::Px(0.),
            border: UiRect {
                right: Val::Px(thickness),
                ..default()
            },
            ..default()
        },
        LinearGradient {
            angle: LinearGradient::TO_LEFT,
            stops: stops_br.clone(),
        },
    );

    let bottom = BorderBundle::new(
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(thickness),
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            bottom: Val::Px(0.),
            border: UiRect {
                bottom: Val::Px(thickness),
                ..default()
            },
            ..default()
        },
        LinearGradient {
            angle: LinearGradient::TO_TOP,
            stops: stops_br.clone(),
        },
    );

    [left, top, right, bottom]
}
