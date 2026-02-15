use bevy::prelude::*;
use bevy_ui_text_input::*;

use crate::ui_styles::{BUTTON_BG_HOVERED, BUTTON_BG_NORMAL, BUTTON_BG_PRESSED};
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Clone)]
#[require(ButtonBackgroundStyle)]//meter más subestilos en el futuro
pub struct ButtonStyle {}

#[derive(Component, Clone)]
#[require(Node, CurrentText, BackgroundColor(BUTTON_BG_NORMAL), TextInputNode, TextInputPrompt, TextInputContents)]//Outline
pub struct LineEdit;

#[derive(Component, Debug, Default, Clone)]
pub struct CurrentText(pub String);

impl CurrentText {
    pub fn new<S: Into<String>>(text: S) -> Self {
        CurrentText(text.into())
    }
}

#[derive(Component, Clone)]
pub struct ButtonBackgroundStyle {
    normal: Color,
    hovered: Color,
    pressed: Color,
}
impl ButtonBackgroundStyle {
    pub fn new(
        normal: Option<Color>,
        hovered: Option<Color>,
        pressed: Option<Color>,
    ) -> Self {
        
        let hovered_color = hovered
        .or_else(|| normal.map(|c| c.mix(&Color::WHITE, 0.3)))
        .unwrap_or(BUTTON_BG_HOVERED);
    
        let pressed_color = pressed
        .or_else(|| normal.map(|c| c.mix(&Color::BLACK, 0.3)))
        .unwrap_or(BUTTON_BG_PRESSED);

        Self {
            normal:  normal.unwrap_or(BUTTON_BG_NORMAL),
            hovered: hovered_color,
            pressed: pressed_color,
        }
    }
    pub fn normal(&self) -> Color {self.normal}
    pub fn hovered(&self) -> Color {self.hovered}
    pub fn pressed(&self) -> Color {self.pressed}
}

impl Default for ButtonBackgroundStyle {
    fn default() -> Self {
        Self {
            normal: BUTTON_BG_NORMAL,
            hovered: BUTTON_BG_HOVERED,
            pressed: BUTTON_BG_PRESSED,
        }
    }
}

