use bevy::prelude::*;

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


