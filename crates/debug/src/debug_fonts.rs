use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};

#[derive(Resource, Default)]
pub struct DebugFontsInitialized(pub bool);

pub fn setup_debug_fonts(
    mut contexts: EguiContexts,
    mut fonts_already_initialized: ResMut<DebugFontsInitialized>,
) {
    if fonts_already_initialized.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    
    let mut fonts = egui::FontDefinitions::default();

    const FONT_NAME: &str = "";

    if let Some(font_data) = fonts.font_data.get(FONT_NAME) {
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, FONT_NAME.to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, FONT_NAME.to_owned());
    } else {
        fonts.font_data.insert(
            FONT_NAME.to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/DejaVuSans.ttf"
            ))),
        );

        // Put DejaVu Sans first for proportional text (higher priority than default)
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, FONT_NAME.to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, FONT_NAME.to_owned());
    }


    ctx.set_fonts(fonts);
    fonts_already_initialized.0 = true;
}
