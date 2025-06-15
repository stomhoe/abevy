use bevy::{color::palettes::tailwind::SLATE_50, prelude::*, text::cosmic_text::ttf_parser::Style};
use bevy_ui_text_input::*;

use crate::ui::{ui_components::ButtonBackgroundStyle, ui_styles::BUTTON_BG_NORMAL};

//TODO LA FUENTE
pub fn text_button<T: Into<String>>(base: impl Bundle, text: T, other_children: impl Bundle, style: Option<ButtonBackgroundStyle>,) -> impl Bundle 
{
    button(base, (Text::new(text), other_children), style)
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


