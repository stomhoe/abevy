use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use camera::camera_components::CameraTarget;
use common::common_components::StrId;
use std::collections::{BTreeSet, HashMap};

use crate::terrain_gen::terrgen_operaton_list_components::OperationList;
use crate::terrain_gen::terrgen_resources::{TerrGenDebugGrid, TerrGenTileDebugInfo};
use ::tilemap_shared::*;

fn pick_value(info: &TerrGenTileDebugInfo, metric: &str) -> Option<f32> {
    if metric == "out" {
        Some(info.output)
    } else {
        info.variables.get(metric).copied()
    }
}

fn div_floor_i32(v: i32, d: i32) -> i32 {
    v.div_euclid(d)
}

pub fn terrgen_debug_window_system(
    mut contexts: EguiContexts,
    mut debug_grid: ResMut<TerrGenDebugGrid>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    oplist_id_query: Query<&StrId, With<OperationList>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    egui::Window::new("Terrgen Tile Debug")
        .default_size([1200.0, 760.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut debug_grid.enabled, "Capture");
                if ui.button("Clear").clicked() {
                    debug_grid.tiles.clear();
                }
            });
            ui.separator();
            ui.label(format!("tracked tiles: {}", debug_grid.tiles.len()));
            ui.label("Only captured samples are shown. Missing samples render as dark cells.");

            let camera_info = camera_query.iter().next();
            let camera_dim = camera_info.map(|(dim_ref, _)| dim_ref.0);

            let mut all_oplists = Vec::<String>::new();
            for id in oplist_id_query.iter() {
                all_oplists.push(id.as_str().to_string());
            }
            all_oplists.sort_unstable();
            all_oplists.dedup();
            const MAX_DROPDOWN_OPLISTS: usize = 512;
            if all_oplists.len() > MAX_DROPDOWN_OPLISTS {
                all_oplists.truncate(MAX_DROPDOWN_OPLISTS);
            }

            ui.horizontal(|ui| {
                ui.label("Filter");
                egui::ComboBox::from_label("Oplist")
                    .selected_text(debug_grid.oplist_filter.as_deref().unwrap_or("All oplists"))
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(debug_grid.oplist_filter.is_none(), "All oplists").clicked() {
                            debug_grid.oplist_filter = None;
                        }
                        for id in all_oplists.iter() {
                            let selected = debug_grid.oplist_filter.as_deref() == Some(id.as_str());
                            if ui.selectable_label(selected, id).clicked() {
                                debug_grid.oplist_filter = Some(id.clone());
                            }
                        }
                    });
            });

            let filter = debug_grid.oplist_filter.clone();
            let mut metrics = BTreeSet::new();
            metrics.insert("out".to_string());
            for (key, info) in debug_grid.tiles.iter() {
                if let Some(dim) = camera_dim && key.dimension != dim {
                    continue;
                }
                if let Some(filter_id) = filter.as_ref() && &info.oplist_id != filter_id {
                    continue;
                }
                for var in info.variables.keys() {
                    metrics.insert(var.clone());
                }
            }
            let metrics = metrics.into_iter().collect::<Vec<_>>();
            if !metrics.is_empty() && !metrics.iter().any(|m| m == &debug_grid.selected_metric) {
                debug_grid.selected_metric = metrics[0].clone();
            }

            let bucket_size = debug_grid.bucket_size_tiles.max(1);
            let bucket_radius = debug_grid.bucket_radius.max(4);

            let mut buckets: HashMap<IVec2, (f32, u32)> = HashMap::new();
            let mut bucket_values: HashMap<IVec2, f32> = HashMap::new();

            let body_h = ui.available_height();
            ui.horizontal(|ui| {
                let left_w = 220.0;
                let panel_h = body_h.max(220.0);

                ui.allocate_ui_with_layout(
                    egui::vec2(left_w, panel_h),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label("Variables");
                        egui::ScrollArea::vertical().max_height(panel_h - 22.0).show(ui, |ui| {
                            for metric in metrics.iter() {
                                let selected = metric == &debug_grid.selected_metric;
                                if ui.selectable_label(selected, metric).clicked() {
                                    debug_grid.selected_metric = metric.clone();
                                }
                            }
                        });
                    },
                );

                ui.separator();

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), panel_h),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(format!("Selected: {}", debug_grid.selected_metric));

                        let Some((dim_ref, transform)) = camera_info else {
                            ui.separator();
                            ui.label("No CameraTarget found.");
                            return;
                        };
                        let dim_ent = dim_ref.0;
                        let anchor = GlobalTilePos::from(transform.translation().xy()).0;
                        let anchor_bucket = IVec2::new(
                            div_floor_i32(anchor.x, bucket_size),
                            div_floor_i32(anchor.y, bucket_size),
                        );
                        let min_bucket = anchor_bucket - IVec2::splat(bucket_radius);
                        let max_bucket = anchor_bucket + IVec2::splat(bucket_radius);

                        buckets.clear();
                        bucket_values.clear();
                        for (key, info) in debug_grid.tiles.iter() {
                            if key.dimension != dim_ent {
                                continue;
                            }
                            if let Some(filter_id) = filter.as_ref() && &info.oplist_id != filter_id {
                                continue;
                            }
                            let Some(v) = pick_value(info, &debug_grid.selected_metric) else { continue; };
                            let bucket = key.gpos;
                            if bucket.x < min_bucket.x || bucket.x > max_bucket.x
                                || bucket.y < min_bucket.y || bucket.y > max_bucket.y
                            {
                                continue;
                            }
                            buckets
                                .entry(bucket)
                                .and_modify(|agg| {
                                    agg.0 += v;
                                    agg.1 += 1;
                                })
                                .or_insert((v, 1));
                        }
                        for (bucket, (sum, count)) in buckets.iter() {
                            if *count > 0 {
                                bucket_values.insert(*bucket, *sum / *count as f32);
                            }
                        }

                        ui.separator();
                        ui.label(format!(
                            "Dim {:?} | camera ({},{}) | bucket:{} | sampled buckets:{}",
                            dim_ent,
                            anchor.x,
                            anchor.y,
                            bucket_size,
                            bucket_values.len(),
                        ));

                        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                            let cell_w = 30.0f32;
                            let cell_h = 22.0f32;
                            let cols = bucket_radius * 2 + 1;
                            let rows = bucket_radius * 2 + 1;
                            let grid_size = egui::vec2(cols as f32 * cell_w, rows as f32 * cell_h);
                            let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
                            let painter = ui.painter_at(rect);

                            for row in 0..rows {
                                for col in 0..cols {
                                    let bx = min_bucket.x + col;
                                    let by = max_bucket.y - row;
                                    let bucket = IVec2::new(bx, by);
                                    let has_value = bucket_values.get(&bucket).copied();

                                    let left = rect.left() + col as f32 * cell_w;
                                    let top = rect.top() + row as f32 * cell_h;
                                    let cell_rect = egui::Rect::from_min_size(
                                        egui::pos2(left, top),
                                        egui::vec2(cell_w, cell_h),
                                    );

                                    let fill = if let Some(v) = has_value {
                                        let t = v.clamp(0.0, 1.0);
                                        egui::Color32::from_rgb(
                                            (40.0 + t * 140.0) as u8,
                                            (20.0 + t * 70.0) as u8,
                                            (40.0 + (1.0 - t) * 120.0) as u8,
                                        )
                                    } else {
                                        egui::Color32::from_rgb(12, 12, 12)
                                    };

                                    painter.rect_filled(cell_rect, 0.0, fill);
                                    if has_value.is_none() {
                                        painter.line_segment(
                                            [cell_rect.left_top(), cell_rect.right_bottom()],
                                            egui::Stroke::new(0.5, egui::Color32::from_gray(28)),
                                        );
                                    }

                                    if let Some(v) = has_value {
                                        painter.text(
                                            cell_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            format!("{v:.2}"),
                                            egui::FontId::proportional(10.0),
                                            egui::Color32::WHITE,
                                        );
                                    }

                                    if bucket == anchor_bucket {
                                        painter.rect_stroke(
                                            cell_rect,
                                            0.0,
                                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                                            egui::StrokeKind::Outside,
                                        );
                                    }
                                }
                            }
                        });
                    },
                );
            });
        });
}
