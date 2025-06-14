use bevy::{color::palettes::tailwind::SLATE_50, prelude::*};

use crate::ui::ui_components::ButtonBackgroundStyle;

pub fn text_button<T: Into<String>>(text: T, node: Option<Node>, style: Option<ButtonBackgroundStyle>,) -> impl Bundle 
{
    button((Text::new(text), TextColor(SLATE_50.into())), node, style)
}

pub fn button(
    children: impl Bundle,
    node: Option<Node>,
    style: Option<ButtonBackgroundStyle>,
) -> impl Bundle {
    (
        Button,
        BackgroundColor(style.unwrap_or_default().normal()),
        node.unwrap_or(Node {
            padding: UiRect::all(Val::Px(5.)),
            width: Val::Px(200.),
            ..default()
        }),
        children![children],
    )
}
pub const fn color_from_triplicate(value: f32) -> Color {
    Color::srgb(value, value, value)
}
