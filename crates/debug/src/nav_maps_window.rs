use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use camera::camera_components::CameraTarget;
use being::being_nav::AiNavGrids;
use tilemap_shared::{AiNavTileBlockedGposCounts, DimensionRef, GlobalTilePos};

use debug_shared::DubugWindowsVisibility;

pub struct NavMapsUiState {
    pub radius: i32,
    pub cell_px: f32,
    pub follow_camera_target: bool,
    pub selected_dim: Option<DimensionRef>,
    pub center_x: i32,
    pub center_y: i32,
    pub show_occupied: bool,
}

impl Default for NavMapsUiState {
    fn default() -> Self {
        Self {
            radius: 16,
            cell_px: 10.0,
            follow_camera_target: true,
            selected_dim: None,
            center_x: 0,
            center_y: 0,
            show_occupied: true,
        }
    }
}

fn paint_nav_viewport(
    ui: &mut egui::Ui,
    cache: &being::being_nav::AiNavGridCache,
    dim_ref: DimensionRef,
    center: GlobalTilePos,
    radius: i32,
    cell_px: f32,
    show_occupied: bool,
    tile_blocked_gpos_counts: &AiNavTileBlockedGposCounts,
    camera_gpos: Option<GlobalTilePos>,
) {
    let side = (radius * 2 + 1).max(1) as usize;
    let grid_size = egui::vec2(side as f32 * cell_px, side as f32 * cell_px);
    let (rect, response) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    for row in 0..side {
        for col in 0..side {
            let dx = col as i32 - radius;
            let dy = radius - row as i32;
            let gpos = GlobalTilePos(center.0 + IVec2::new(dx, dy));
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(
                    rect.left() + col as f32 * cell_px,
                    rect.top() + row as f32 * cell_px,
                ),
                egui::vec2(cell_px, cell_px),
            );
            let id = ui.make_persistent_id(("nav_maps_grid_cell", dim_ref.0, row, col));
            let _ = ui.interact(cell_rect, id, egui::Sense::hover());
            let Some(local) = cache.local_from_gpos(gpos) else {
                painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgb(20, 20, 20));
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_gray(35)),
                    egui::StrokeKind::Inside,
                );
                continue;
            };
            let passable = cache.grid.is_passable(local);
            let blocked_by_tile = tile_blocked_gpos_counts.is_blocked(dim_ref, gpos);
            let occupied_ent = cache.occupied.get(&local).copied();
            let fill = if blocked_by_tile {
                egui::Color32::from_rgb(104, 28, 28)
            } else if passable {
                egui::Color32::from_rgb(34, 52, 34)
            } else {
                egui::Color32::from_rgb(92, 38, 38)
            };
            painter.rect_filled(cell_rect, 0.0, fill);
            if show_occupied && occupied_ent.is_some() {
                painter.rect_filled(
                    cell_rect.shrink((cell_px * 0.26).max(1.0)),
                    0.0,
                    egui::Color32::from_rgb(60, 140, 210),
                );
            }
            painter.rect_stroke(
                cell_rect,
                0.0,
                egui::Stroke::new(0.5, egui::Color32::from_gray(42)),
                egui::StrokeKind::Inside,
            );
            if !passable {
                painter.rect_stroke(
                    cell_rect.shrink(0.5),
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 70, 70)),
                    egui::StrokeKind::Inside,
                );
            }
            if camera_gpos == Some(gpos) {
                painter.rect_stroke(
                    cell_rect.shrink(0.5),
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Inside,
                );
            }
            if cell_px >= 11.0 {
                let label = if show_occupied && occupied_ent.is_some() {
                    "B"
                } else if blocked_by_tile {
                    "T"
                } else if !passable {
                    "X"
                } else {
                    ""
                };
                if !label.is_empty() {
                    painter.text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional((cell_px * 0.55).clamp(8.0, 12.0)),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
    }

    if response.hovered() {
        let pointer = ui.input(|input| input.pointer.hover_pos());
        if let Some(pointer) = pointer {
            let local_px = pointer - rect.min;
            let col = (local_px.x / cell_px).floor() as i32;
            let row = (local_px.y / cell_px).floor() as i32;
            if row >= 0 && col >= 0 && row < side as i32 && col < side as i32 {
                let dx = col - radius;
                let dy = radius - row;
                let gpos = GlobalTilePos(center.0 + IVec2::new(dx, dy));
                let Some(local) = cache.local_from_gpos(gpos) else {
                    return;
                };
                let blocked_by_tile = tile_blocked_gpos_counts.is_blocked(dim_ref, gpos);
                response.clone().on_hover_text(format!(
                    "Dim {:?}\nWorld: {}\nLocal: {:?}\nPassable: {}\nTileBlocked: {}\nOccupied: {}",
                    dim_ref,
                    gpos,
                    local,
                    cache.grid.is_passable(local),
                    blocked_by_tile,
                    cache
                        .occupied
                        .get(&local)
                        .map(|ent| format!("{:?}", ent))
                        .unwrap_or_else(|| "none".to_string()),
                ));
            }
        }
    }
}

#[allow(unused_parens, )]
pub fn nav_maps_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    nav_grids: Res<AiNavGrids>,
    tile_blocked_gpos_counts: Res<AiNavTileBlockedGposCounts>,
    camera_target_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    mut ui_state: Local<NavMapsUiState>,
) {
    if !window_visible.nav_maps {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = window_visible.nav_maps;

    let camera_target = camera_target_query
        .iter()
        .next()
        .map(|(dim_ref, gtf)| (*dim_ref, GlobalTilePos::from(gtf.translation().xy())));
    if ui_state.follow_camera_target {
        if let Some((camera_dim, camera_gpos)) = camera_target.as_ref().copied() {
            ui_state.center_x = camera_gpos.0.x;
            ui_state.center_y = camera_gpos.0.y;
            ui_state.selected_dim = Some(camera_dim);
        }
    }

    let available_dims: Vec<DimensionRef> = nav_grids.by_dim.keys().copied().collect();
    let selected_dim = ui_state
        .selected_dim
        .filter(|dim| nav_grids.by_dim.contains_key(dim))
        .or_else(|| available_dims.first().copied());

    let Some(selected_dim) = selected_dim else {
        ui_state.selected_dim = None;
        egui::Window::new("AI Nav Maps")
            .default_size([960.0, 720.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut ui_state.follow_camera_target, "Follow camera");
                    ui.add(egui::Slider::new(&mut ui_state.radius, 4..=64).text("radius"));
                    ui.add(egui::Slider::new(&mut ui_state.cell_px, 4.0..=20.0).text("cell px"));
                    ui.checkbox(&mut ui_state.show_occupied, "Show occupied");
                });
                ui.separator();
                ui.label("No AI nav grids are currently loaded.");
            });
        window_visible.nav_maps = open;
        return;
    };
    ui_state.selected_dim = Some(selected_dim);

    let Some(cache) = nav_grids.by_dim.get(&selected_dim) else {
        window_visible.nav_maps = open;
        return;
    };
    let center = GlobalTilePos::new(ui_state.center_x, ui_state.center_y);
    let camera_gpos = camera_target
        .as_ref()
        .copied()
        .filter(|(camera_dim, _)| *camera_dim == selected_dim)
        .map(|(_, gpos)| gpos);

    egui::Window::new("AI Nav Maps")
        .default_size([960.0, 720.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.follow_camera_target, "Follow camera");
                ui.add(egui::Slider::new(&mut ui_state.radius, 4..=64).text("radius"));
                ui.add(egui::Slider::new(&mut ui_state.cell_px, 4.0..=20.0).text("cell px"));
                ui.checkbox(&mut ui_state.show_occupied, "Show occupied");
            });
            ui.horizontal(|ui| {
                ui.label("Dimension:");
                egui::ComboBox::from_id_salt("nav_maps_dim_select")
                    .selected_text(format!("{:?}", selected_dim))
                    .show_ui(ui, |ui| {
                        for dim in available_dims.iter().copied() {
                            ui.selectable_value(
                                &mut ui_state.selected_dim,
                                Some(dim),
                                format!("{:?}", dim),
                            );
                        }
                    });
                ui.label("Center x:");
                ui.add(egui::DragValue::new(&mut ui_state.center_x).speed(1.0));
                ui.label("Center y:");
                ui.add(egui::DragValue::new(&mut ui_state.center_y).speed(1.0));
            });
            ui.separator();
            ui.label(format!(
                "Grid min={:?} size={}x{} occupied={}",
                cache.min,
                cache.grid.width(),
                cache.grid.height(),
                cache.occupied.len(),
            ));
            paint_nav_viewport(
                ui,
                cache,
                selected_dim,
                center,
                ui_state.radius,
                ui_state.cell_px,
                ui_state.show_occupied,
                &tile_blocked_gpos_counts,
                camera_gpos,
            );
        });

    window_visible.nav_maps = open;
}
