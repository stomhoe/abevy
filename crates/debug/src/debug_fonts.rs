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

    // Add Symbola font for emoji and symbol support
    fonts.font_data.insert(
        "Symbola".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Symbola.ttf"
        ))),
    );

    // Put Symbola first for emoji/symbol support
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Symbola".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "Symbola".to_owned());

    // Apply the fonts
    ctx.set_fonts(fonts);
    initialized.0 = true;
}
