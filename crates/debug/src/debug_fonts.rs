use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};

#[derive(Resource, Default)]
pub struct FontsInitialized(pub bool);

/// Setup custom fonts for debug UI to support unicode symbols
pub fn setup_debug_fonts(
    mut contexts: EguiContexts,
    mut initialized: ResMut<FontsInitialized>,
) {
    if initialized.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    
    // Start with default fonts
    let mut fonts = egui::FontDefinitions::default();

    const FONT_NAME: &str = "";

    // Add a font that supports unicode characters well
    // Using DejaVu Sans which has excellent unicode coverage
    if let Some(font_data) = fonts.font_data.get(FONT_NAME) {
        // Font already available, add it to families
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
        // Load DejaVu Sans from assets
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

        // Also use it for monospace
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, FONT_NAME.to_owned());
    }

    // Apply the fonts
    ctx.set_fonts(fonts);
    initialized.0 = true;
}
