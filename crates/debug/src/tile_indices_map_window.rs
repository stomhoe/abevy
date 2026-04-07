use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use camera::camera_components::CameraTarget;
use tilemap::chunking::MacroChunkU16IndexMatrix;
use tilemap_shared::{DimensionRef, GlobalTilePos, LoadedMacroChunks};

use crate::debug_resources::DubugWindowsVisibility;

pub struct TileIndicesMapUiState {
    radius: i32,
    cell_px: f32,
    follow_camera_target: bool,
    center_dim: Entity,
    center_x: i32,
    center_y: i32,
    selected_gpos: Option<GlobalTilePos>,
}
impl Default for TileIndicesMapUiState {
    fn default() -> Self {
        Self {
            radius: 10,
            cell_px: 24.0,
            follow_camera_target: true,
            center_dim: Entity::PLACEHOLDER,
            center_x: 0,
            center_y: 0,
            selected_gpos: None,
        }
    }
}

fn gather_tile_indices_u16_at(
    loaded_macro_chunks: &LoadedMacroChunks,
    macro_chunk_tile_indices_query: &Query<&MacroChunkU16IndexMatrix, >,
    dim_ref: DimensionRef,
    gpos: GlobalTilePos,
    out: &mut Vec<u16>,
) {
    out.clear();
    let macro_chunk_pos = gpos.to_chunkpos().to_macrochunk_pos();
    let Some(&macro_chunk_ent) = loaded_macro_chunks.0.get(&(dim_ref, macro_chunk_pos)) else {
        return;
    };
    let Ok(macro_chunk_tile_indices) = macro_chunk_tile_indices_query.get(macro_chunk_ent) else {
        return;
    };
    let anchor = macro_chunk_pos.to_chunkpos().to_tilepos();
    let Some(indices) = macro_chunk_tile_indices.tile_indices_at_gpos(anchor, gpos) else {
        return;
    };
    out.reserve(indices.len());
    for index in indices.iter() {
        out.push(index.0);
    }
}

#[allow(unused_parens, )]
pub fn tile_indices_map_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    loaded_macro_chunks: Res<LoadedMacroChunks>,
    macro_chunk_tile_indices_query: Query<&MacroChunkU16IndexMatrix, >,
    camera_target_query: Query<(&DimensionRef, &GlobalTransform), (With<CameraTarget>, )>,
    mut ui_state: Local<TileIndicesMapUiState>,
    mut cell_indices: Local<Vec<u16>>,
    mut selected_indices: Local<Vec<u16>>,
) {
    if !window_visible.tile_indices_map {
        return;
    }
    if ui_state.radius <= 0 {
        *ui_state = TileIndicesMapUiState::default();
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if ui_state.follow_camera_target {
        let Ok((&dim_ref, gtf)) = camera_target_query.single() else {
            return;
        };
        ui_state.center_dim = dim_ref.0;
        let center = GlobalTilePos::from(gtf.translation().xy());
        ui_state.center_x = center.0.x;
        ui_state.center_y = center.0.y;
    }

    let dim_ref = DimensionRef(ui_state.center_dim);
    let center = GlobalTilePos::new(ui_state.center_x, ui_state.center_y);
    let mut open = window_visible.tile_indices_map;

    egui::Window::new("Tile Index Map")
        .default_size([900.0, 640.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.follow_camera_target, "Follow camera");
                ui.add(egui::Slider::new(&mut ui_state.radius, 4..=40).text("radius"));
                ui.add(egui::Slider::new(&mut ui_state.cell_px, 12.0..=40.0).text("cell px"));
            });
            ui.horizontal(|ui| {
                ui.label("Dimension:");
                ui.label(format!("{:?}", ui_state.center_dim));
                ui.label("Center x:");
                ui.add(egui::DragValue::new(&mut ui_state.center_x).speed(1.0));
                ui.label("Center y:");
                ui.add(egui::DragValue::new(&mut ui_state.center_y).speed(1.0));
            });
            ui.separator();

            let side = (ui_state.radius * 2 + 1).max(1) as usize;
            let grid_size = egui::vec2(side as f32 * ui_state.cell_px, side as f32 * ui_state.cell_px);
            let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            for row in 0..side {
                for col in 0..side {
                    let dx = col as i32 - ui_state.radius;
                    let dy = ui_state.radius - row as i32;
                    let gpos = GlobalTilePos(center.0 + IVec2::new(dx, dy));
                    gather_tile_indices_u16_at(
                        &loaded_macro_chunks,
                        &macro_chunk_tile_indices_query,
                        dim_ref,
                        gpos,
                        &mut cell_indices,
                    );
                    let x = rect.left() + col as f32 * ui_state.cell_px;
                    let y = rect.top() + row as f32 * ui_state.cell_px;
                    let cell_rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(ui_state.cell_px, ui_state.cell_px),
                    );
                    let id = ui.make_persistent_id(("tile_indices_map", row, col));
                    let response = ui.interact(cell_rect, id, egui::Sense::click());
                    let fill = if cell_indices.is_empty() {
                        egui::Color32::from_rgb(16, 16, 16)
                    } else if cell_indices.len() == 1 {
                        egui::Color32::from_rgb(55, 95, 60)
                    } else {
                        egui::Color32::from_rgb(95, 75, 50)
                    };
                    painter.rect_filled(cell_rect, 0.0, fill);
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_gray(38)),
                        egui::StrokeKind::Inside,
                    );
                    if gpos == center {
                        painter.rect_stroke(
                            cell_rect.shrink(0.5),
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if Some(gpos) == ui_state.selected_gpos {
                        painter.rect_stroke(
                            cell_rect.shrink(1.5),
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if !cell_indices.is_empty() && ui_state.cell_px >= 22.0 {
                        let label = if cell_indices.len() == 1 {
                            cell_indices[0].to_string()
                        } else {
                            format!("{}+", cell_indices.len())
                        };
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(10.0),
                            egui::Color32::WHITE,
                        );
                    }
                    if response.clicked() {
                        ui_state.selected_gpos = Some(gpos);
                        gather_tile_indices_u16_at(
                            &loaded_macro_chunks,
                            &macro_chunk_tile_indices_query,
                            dim_ref,
                            gpos,
                            &mut selected_indices,
                        );
                    }
                }
            }

            ui.separator();
            if let Some(selected_gpos) = ui_state.selected_gpos {
                ui.label(format!("Selected gpos: {}", selected_gpos));
                ui.label(format!("Indices count: {}", selected_indices.len()));
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    if selected_indices.is_empty() {
                        ui.label("No indices at this tile.");
                    } else {
                        for idx in selected_indices.iter() {
                            ui.monospace(format!("{}", idx));
                        }
                    }
                });
            } else {
                ui.label("Click a tile to inspect all tile indices at that position.");
            }
        });

    window_visible.tile_indices_map = open;
}
