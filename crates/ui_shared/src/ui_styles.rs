use bevy::prelude::*;

pub const RETRO_FONT: &str = "fonts/character_creation/W95FA.ttf";
pub const RETRO_FONT_BOLD: &str = "fonts/character_creation/W95FA.ttf";
pub const TITLE_FONT: &str = "fonts/character_creation/LiberationSerif-Regular.ttf";

pub fn panel_radius() -> BorderRadius { BorderRadius::all(Val::Px(3.)) }
pub const FORM: Color = Color::srgba(0.10, 0.095, 0.085, 0.4);
pub const PANEL: Color = Color::srgba(0.075, 0.07, 0.06, 0.28);
pub const INSET: Color = Color::srgba(0.025, 0.022, 0.018, 0.6);
pub const FIELD: Color = Color::srgb(0.012, 0.011, 0.009);
pub const TEXT: Color = Color::srgb(0.94, 0.93, 0.87);
pub const DROPDOWN_FIELD: Color = Color::srgb(0.92, 0.89, 0.82);
pub const DROPDOWN_TEXT: Color = Color::srgb(0.075, 0.055, 0.035);
pub const BUTTON_BG_NORMAL: Color = Color::srgba(0.18, 0.17, 0.145, 0.24);
pub const BUTTON_BG_HOVERED: Color = Color::srgba(0.45, 0.43, 0.37, 0.32);
pub const BUTTON_BG_PRESSED: Color = Color::srgba(0.075, 0.065, 0.05, 0.6);

pub fn bevel_border() -> BorderColor {
    BorderColor { top: Color::srgba(0.75, 0.72, 0.65, 0.7), left: Color::srgba(0.65, 0.61, 0.54, 0.65), bottom: Color::srgba(0.025, 0.02, 0.015, 0.75), right: Color::srgba(0.035, 0.03, 0.02, 0.65) }
}

pub fn inset_border() -> BorderColor {
    let border = bevel_border();
    BorderColor { top: border.bottom, left: border.right, bottom: border.top, right: border.left }
}

pub fn control_sheen() -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::new(LinearGradient::TO_BOTTOM, vec![
        ColorStop::new(Color::srgba(0.9, 0.86, 0.77, 0.18), Val::Percent(0.)),
        ColorStop::new(Color::NONE, Val::Percent(45.)),
        ColorStop::new(Color::srgba(0., 0., 0., 0.4), Val::Percent(100.)),
    ]))
}


