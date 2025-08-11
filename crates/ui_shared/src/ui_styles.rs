use bevy::prelude::*;

use crate::ui_functions::color_from_triplicate;


const BUTTON_BG_NORMAL_GRAY_FACTOR: f32 = 0.5;
const BUTTON_BG_HOVERED_GRAY_FACTOR: f32 = 0.8;
const BUTTON_BG_PRESSED_GRAY_FACTOR: f32 = 0.3;

pub const BUTTON_BG_NORMAL: Color = color_from_triplicate(BUTTON_BG_NORMAL_GRAY_FACTOR);
pub const BUTTON_BG_HOVERED: Color = color_from_triplicate(BUTTON_BG_HOVERED_GRAY_FACTOR);
pub const BUTTON_BG_PRESSED: Color = color_from_triplicate(BUTTON_BG_PRESSED_GRAY_FACTOR);


