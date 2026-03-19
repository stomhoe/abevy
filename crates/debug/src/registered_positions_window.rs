use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use tilemap::tilemap_resources::ImportantRegisteredPositions;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn registered_positions_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    registered_positions: Res<ImportantRegisteredPositions>,
) {
    if !window_visible.registered_positions {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 10.0;
    let default_y = screen_rect.top() + 650.0;
    let mut open = window_visible.registered_positions;

    egui::Window::new("Registered Positions")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(600.0)
        .default_height(300.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Registered Positions");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("RegisteredPositions:");
                ui.label(format!("Exempted entities: {}", registered_positions.get_exempted_tile_ents().len()));
                ui.label(format!("Registered ezero entries: {}", registered_positions.get_registered_ezeros().len()));
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Show exempted entities
                    if !registered_positions.get_exempted_tile_ents().is_empty() {
                        ui.label("Exempted Entities:");
                        for entity in registered_positions.get_exempted_tile_ents().iter() {
                            let is_selected = selected_entities.selected_exempted_entity == Some(*entity);
                            let label = format!("  {:?}", entity);
                            if ui.selectable_label(is_selected, label).clicked() {
                                selected_entities.selected_exempted_entity = Some(*entity);
                                selected_entities.selected_tile = None;
                                window_visible.tile_details = true;
                            }
                        }
                        ui.separator();
                    }

                    // Show registered entries
                    if !registered_positions.get_registered_ezeros().is_empty() {
                        ui.label("Registered Positions:");
                        for (entity, positions) in registered_positions.get_registered_ezeros().iter() {
                            ui.label(format!("Entity {:?}: {} positions", entity, positions.len()));
                            for (dim_ref, pos) in positions {
                                ui.label(format!("  Dim: {:?}, Pos: {:?}", dim_ref, pos));
                            }
                        }
                    }
                });
        });
    window_visible.registered_positions = open;
}
