use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use game_common::game_common_timers::{DespawnOnTimeout, MessageOnTimeout};
use std::collections::{BTreeMap, HashMap};

use camera::camera_components::CameraTarget;
use common::common_components::*;
use tilemap::regioning::natural::river::{RiverDebugData, RiverRegionDebugInfo};
use tilemap::regioning::regioning_components::*;
use ::tilemap_shared::*;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn regions_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    region_query: Query<(
        Entity,
        &Region,
        &DimensionRef,
        &RegionPos,
        Option<&Name>,
        Option<&GridOfSgcs>,
        Option<&ClaimList>,
        Option<&RegionPlannedTiles>,
        Option<&ChunksActiveInRegion>,
        Option<&CountsOfSgcs>,
        &RegionState,
        Has<MessageOnTimeout>,
        Has<DespawnOnTimeout>,
    ), With<Region>>,
    camera_dimension: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    id_query: Query<&StrId>,
    river_debug: Option<Res<RiverDebugData>>,
) {
    if !window_visible.regions_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 350.0;
    let default_y = screen_rect.top() + 10.0;
    let mut open = window_visible.regions_list;

    // Group regions by dimension and position (keyed by StrId and Entity number)
    let mut regions_by_dimension: BTreeMap<String, (Entity, HashMap<RegionPos, (Entity, Option<&Name>, Option<&GridOfSgcs>, Option<&ClaimList>, Option<&RegionPlannedTiles>, Option<&ChunksActiveInRegion>, Option<&CountsOfSgcs>, RegionState, bool, bool)>)> =
        BTreeMap::new();

    for (entity, _region, dim_ref, region_pos, name, grid, claim_list, planned_tiles, chunks_active, counts, &region_state, timeout_timer, empty_timer) in region_query.iter() {
        let dim_key = if let Ok(str_id) = id_query.get(dim_ref.0) {
            format!("{} ({})", str_id.as_str(), dim_ref.0.index())
        } else {
            format!("{:?} ({})", dim_ref, dim_ref.0.index())
        };

        regions_by_dimension
            .entry(dim_key.clone())
            .or_insert_with(|| (dim_ref.0, HashMap::new()))
            .1
            .insert(*region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, region_state, timeout_timer, empty_timer));
    }

    // Get camera target dimension and position
    let (camera_dim_ref, camera_chunk_pos, camera_tile_pos, camera_region_pos) = camera_dimension.iter().next()
        .map(|(dim_ref, transform)| {
            let chunk_pos = ChunkPos::from(transform.translation());
            let tile_pos = GlobalTilePos::from(transform.translation().xy());
            let region_pos = chunk_pos.to_region_pos();
            (Some(dim_ref), Some(chunk_pos), Some(tile_pos), Some(region_pos))
        })
        .unwrap_or((None, None, None, None));

    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = regions_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        let camera_dim_key = if let Ok(camera_str_id) = id_query.get(camera_ref.0) {
            format!("{} ({})", camera_str_id.as_str(), camera_ref.0.index())
        } else {
            format!("{:?} ({})", camera_ref, camera_ref.0.index())
        };
        sorted_dims.sort_by(|a, b| {
            if a == &camera_dim_key { std::cmp::Ordering::Less }
            else if b == &camera_dim_key { std::cmp::Ordering::Greater }
            else { a.cmp(b) }
        });
    }

    egui::Window::new("Regions Grid")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(700.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Regions: {}", region_query.iter().count()));
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some((_, regions_map)) = regions_by_dimension.get(dim_key) {
                    let is_camera_dim = camera_dim_ref.map_or(false, |camera_ref| {
                        let camera_key = if let Ok(camera_str_id) = id_query.get(camera_ref.0) {
                            format!("{} ({})", camera_str_id.as_str(), camera_ref.0.index())
                        } else {
                            format!("{:?} ({})", camera_ref, camera_ref.0.index())
                        };
                        dim_key == &camera_key
                    });
                    let header_label = format!("{} - {} regions", dim_key, regions_map.len());
                    egui::CollapsingHeader::new(&header_label)
                        .default_open(is_camera_dim)
                        .show(ui, |ui| {
                        // Create grid of region positions
                        let positions: Vec<RegionPos> = regions_map.keys().copied().collect();
                        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
                            positions.iter().map(|p| p.0.x).min(),
                            positions.iter().map(|p| p.0.x).max(),
                            positions.iter().map(|p| p.0.y).min(),
                            positions.iter().map(|p| p.0.y).max(),
                        ) {
                            egui::Grid::new(format!("regions_grid_{:?}", dim_key.replace(" ", "_")))
                                .spacing([5.0, 5.0])
                                .show(ui, |ui| {
                                    for y in (min_y..=max_y).rev() {
                                        for x in min_x..=max_x {
                                            let pos = RegionPos(IVec2::new(x, y));
                                            if let Some((entity, name, ..)) = regions_map.get(&pos) {
                                                let is_selected = selected_entities.selected_regions.contains(entity);
                                                let is_camera_pos = camera_region_pos.map_or(false, |cam_pos| cam_pos == pos);

                                                let mut label = format!("{},{}", x, y);
                                                if let Some(n) = name {
                                                    label = format!("{}\n{}", label, n);
                                                }

                                                let button_response = if is_camera_pos {
                                                    ui.selectable_label(is_selected, egui::RichText::new(&label).color(egui::Color32::YELLOW).strong())
                                                } else {
                                                    ui.selectable_label(is_selected, &label)
                                                };

                                                if button_response.clicked() {
                                                    if is_selected {
                                                        selected_entities.selected_regions.clear();
                                                    } else {
                                                        selected_entities.selected_regions.clear();
                                                        selected_entities.selected_regions.insert(*entity);
                                                    }
                                                }
                                            } else {
                                                ui.label("");
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                        }
                    });
                }
            }

            // Show details for selected regions in stable order
            let mut selected_region_details: Vec<_> = regions_by_dimension.iter()
                .flat_map(|(_, (_, map))| map.iter())
                .filter(|(_, (entity, ..))| selected_entities.selected_regions.contains(entity))
                .collect();
            selected_region_details.sort_by_key(|(_, (entity, ..))| entity.index());

            for (region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, region_state, timeout, despawn)) in selected_region_details {
                let name_str = name.map(|n| format!("{}", n)).unwrap_or_else(|| "unnamed".to_string());
                    egui::CollapsingHeader::new(format!("Details: {} (Entity: {:?})", name_str, entity))
                        .default_open(true)
                        .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Inspect Entity").clicked() {
                                selected_entities.selected_regions.clear();
                                selected_entities.selected_regions.insert(*entity);
                                window_visible.region_details = true;
                            }
                            if ui.button("River Debug").clicked() {
                                selected_entities.selected_river_debug_region = Some(*entity);
                                window_visible.river_debug = true;
                            }
                            if ui.button("River Samples").clicked() {
                                selected_entities.selected_river_debug_region = Some(*entity);
                                window_visible.river_sample_values = true;
                            }
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                if let Some(grid_sgcs) = grid {
                                    ui.label("GridOfSgcs:");
                                    ui.indent("grid_sgcs", |ui| {
                                        // Only highlight if camera is in the same dimension as the region
                                        let highlight_pos = if let (Some(cam_pos), Some(cam_dim)) = (camera_chunk_pos, camera_dim_ref) {
                                            // Find the region's dimension ref
                                            let region_dim_matches = region_query.get(*entity).map(|(_, _, region_dim_ref, _, ..)| region_dim_ref == cam_dim).unwrap_or(false);
                                            let cam_region_pos = cam_pos.to_region_pos();
                                            if region_dim_matches && cam_region_pos == *region_pos {
                                                Some(cam_pos)
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        };
                                        if let Some(clicked_sgc_ent) = grid_sgcs.render_grid(ui, highlight_pos, Some(*region_pos)) {
                                            selected_entities.selected_exempted_entity = Some(clicked_sgc_ent);
                                            window_visible.exempted_entity_details = true;
                                        }
                                    });
                                }
                            });

                            ui.vertical(|ui| {
                                if let Some(claim) = claim_list {
                                    ui.label(format!("ClaimList: {}/{}", claim.processed_up_to_i, tilemap::regioning::regioning_components::MAX_CLAIMS));
                                }

                                if let Some(planned) = planned_tiles {
                                    ui.label(format!("PlannedTiles pending: {}", planned.pending_chunks_count()));
                                }

                                if let Some(chunks) = chunks_active {
                                    ui.label(format!("ChunksActive: {}", chunks.entities().len()));
                                }

                                if let Some(count_sgcs) = counts {
                                    ui.label(format!("CountSgcs: {}", count_sgcs.0.len()));
                                }

                                if *timeout {
                                    ui.label("⏱ PendingOfferTimeout");
                                }

                                if *despawn {
                                    ui.label("🗑 DespawnOnTimeout");
                                }

                                ui.label(format!("State: {:?}", region_state));
                            });
                        });

                    });
            }
        });
    window_visible.regions_list = open;

    let mut open_sample_values_from_river = false;
    if window_visible.river_debug {
        let mut river_open = window_visible.river_debug;
        egui::Window::new("River Debug")
            .default_pos([screen_rect.left() + 24.0, screen_rect.top() + 120.0])
            .resizable(true)
            .movable(true)
            .open(&mut river_open)
            .show(ctx, |ui| {
                let Some(region_ent) = selected_entities.selected_river_debug_region else {
                    ui.colored_label(egui::Color32::RED, "No region selected. Use the River Debug button in Regions Grid.");
                    return;
                };
                let Ok((_, _, dim_ref, region_pos, name, _, _, _, _, _, _, _, _)) = region_query.get(region_ent) else {
                    ui.colored_label(egui::Color32::RED, "Selected region no longer exists.");
                    return;
                };
                let title_name = name.map(|n| n.to_string()).unwrap_or_else(|| "unnamed".to_string());
                ui.label(format!("Region: {} ({:?})", title_name, region_pos));
                ui.label(format!("Entity: {:?}", region_ent));
                ui.label(format!("Dimension: {:?}", dim_ref));
                ui.separator();

                let Some(river_debug) = river_debug.as_ref() else {
                    ui.colored_label(egui::Color32::RED, "River debug resource unavailable.");
                    return;
                };
                let Some(river_info) = river_debug.0.get(&(*dim_ref, *region_pos)) else {
                    ui.colored_label(egui::Color32::RED, "No river debug data for this region yet.");
                    return;
                };

                ui.horizontal(|ui| {
                    ui.label(format!("successes: {}", river_info.success_count));
                    ui.label(egui::RichText::new(format!("failures: {}", river_info.failure_count)).color(egui::Color32::RED));
                    ui.label(format!("active probes: {}", river_info.active_probe_chunks.len()));
                    ui.label(format!("river tiles: {}", river_info.river_tiles.len()));
                    ui.label(format!("sampled points: {}", river_info.sampled_points.len()));
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("River Chunk Map").strong());
                    if ui.button("Sample Values Map").clicked() {
                        open_sample_values_from_river = true;
                    }
                });
                render_river_chunk_debug_map(ui, *region_pos, camera_chunk_pos, river_info);
                ui.separator();
                ui.label(egui::RichText::new("River Tile Preview").strong());
                render_river_tile_preview_map(ui, *region_pos, river_info);
                ui.separator();
                ui.label(egui::RichText::new("Recent Events").strong());
                for event in river_info.recent_events.iter().rev().take(24) {
                    let color = if event.is_failure {
                        egui::Color32::RED
                    } else {
                        egui::Color32::LIGHT_BLUE
                    };
                    ui.label(egui::RichText::new(format!(
                        "offer {} @ {:?}: {}",
                        event.offer_i, event.start_chunk, event.reason
                    )).color(color));
                }
            });
        window_visible.river_debug = river_open;
    }
    if open_sample_values_from_river {
        window_visible.river_sample_values = true;
    }

    if window_visible.river_sample_values {
        let mut samples_open = window_visible.river_sample_values;
        egui::Window::new("River Sample Values")
            .default_pos([screen_rect.left() + 430.0, screen_rect.top() + 120.0])
            .resizable(true)
            .movable(true)
            .open(&mut samples_open)
            .show(ctx, |ui| {
                let Some(region_ent) = selected_entities.selected_river_debug_region else {
                    ui.colored_label(egui::Color32::RED, "No region selected. Use River Samples button in Regions Grid.");
                    return;
                };
                let Ok((_, _, dim_ref, region_pos, name, _, _, _, _, _, _, _, _)) = region_query.get(region_ent) else {
                    ui.colored_label(egui::Color32::RED, "Selected region no longer exists.");
                    return;
                };
                let title_name = name.map(|n| n.to_string()).unwrap_or_else(|| "unnamed".to_string());
                ui.label(format!("Region: {} ({:?})", title_name, region_pos));
                ui.label(format!("Entity: {:?}", region_ent));
                ui.label(format!("Dimension: {:?}", dim_ref));
                ui.separator();

                let Some(river_debug) = river_debug.as_ref() else {
                    ui.colored_label(egui::Color32::RED, "River debug resource unavailable.");
                    return;
                };
                let Some(river_info) = river_debug.0.get(&(*dim_ref, *region_pos)) else {
                    ui.colored_label(egui::Color32::RED, "No river debug data for this region yet.");
                    return;
                };
                let camera_tile_in_region = if camera_dim_ref == Some(dim_ref) {
                    camera_tile_pos
                } else {
                    None
                };
                render_river_sample_values_map(ui, *region_pos, camera_tile_in_region, river_info);
            });
        window_visible.river_sample_values = samples_open;
    }
}

fn render_river_chunk_debug_map(
    ui: &mut egui::Ui,
    region_pos: RegionPos,
    camera_chunk_pos: Option<ChunkPos>,
    river_info: &RiverRegionDebugInfo,
) {
    let cell = 10.0;
    let width = REGION_SIZE_IN_CHUNKS.x() as usize;
    let height = REGION_SIZE_IN_CHUNKS.y() as usize;
    let size = egui::vec2(width as f32 * cell, height as f32 * cell);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    for y in 0..height {
        for x in 0..width {
            let chunk = region_pos.to_chunkpos() + IVec2::new(x as i32, y as i32);
            let mut fill = egui::Color32::from_rgb(26, 26, 26);
            if river_info.claimed_chunks.contains(&chunk) {
                fill = egui::Color32::from_rgb(30, 120, 220);
            }
            if river_info.active_probe_chunks.contains(&chunk) {
                fill = egui::Color32::from_rgb(220, 140, 30);
            }
            if river_info.failed_chunks.contains(&chunk) {
                fill = egui::Color32::from_rgb(210, 30, 30);
            }

            let draw_y = height - 1 - y;
            let r = egui::Rect::from_min_size(
                egui::pos2(rect.left() + x as f32 * cell, rect.top() + draw_y as f32 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            painter.rect_filled(r, 0.0, fill);
        }
    }

    if let Some(camera_chunk_pos) = camera_chunk_pos {
        let local = camera_chunk_pos - region_pos.to_chunkpos();
        if local.0.x >= 0
            && local.0.x < REGION_SIZE_IN_CHUNKS.x()
            && local.0.y >= 0
            && local.0.y < REGION_SIZE_IN_CHUNKS.y()
        {
            let x = local.0.x as usize;
            let y = local.0.y as usize;
            let draw_y = height - 1 - y;
            let r = egui::Rect::from_min_size(
                egui::pos2(rect.left() + x as f32 * cell, rect.top() + draw_y as f32 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            painter.rect_stroke(
                r,
                0.0,
                egui::Stroke::new(1.5, egui::Color32::YELLOW),
                egui::StrokeKind::Outside,
            );
        }
    }

    ui.label("Legend: blue=river chunks, orange=active probe, red=failed attempts");
}

fn render_river_tile_preview_map(
    ui: &mut egui::Ui,
    region_pos: RegionPos,
    river_info: &RiverRegionDebugInfo,
) {
    let size = egui::vec2(360.0, 360.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 18, 18));

    let (min_chunk, max_chunk_excl) = region_pos.chunk_bounds();
    let min_tile = min_chunk.to_tilepos();
    let max_tile_excl = max_chunk_excl.to_tilepos();
    let span_x = (max_tile_excl.0.x - min_tile.0.x).max(1) as f32;
    let span_y = (max_tile_excl.0.y - min_tile.0.y).max(1) as f32;

    for tile in &river_info.river_tiles {
        let nx = ((tile.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 0.9999);
        let ny = ((tile.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 0.9999);
        let px = rect.left() + nx * rect.width();
        let py = rect.bottom() - ny * rect.height();
        let dot = egui::Rect::from_center_size(egui::pos2(px, py), egui::vec2(2.0, 2.0));
        painter.rect_filled(dot, 0.0, egui::Color32::from_rgb(45, 160, 255));
    }

    for chunk in &river_info.failed_chunks {
        let center = chunk.to_tilepos() + IVec2::new((ChunkPos::CHUNK_SIZE.x / 2) as i32, (ChunkPos::CHUNK_SIZE.y / 2) as i32);
        let nx = ((center.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 0.9999);
        let ny = ((center.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 0.9999);
        let px = rect.left() + nx * rect.width();
        let py = rect.bottom() - ny * rect.height();
        let mark = egui::Rect::from_center_size(egui::pos2(px, py), egui::vec2(5.0, 5.0));
        painter.rect_filled(mark, 0.0, egui::Color32::RED);
    }

    ui.label("Legend: blue=river tiles preview, red=failed attempt chunk centers");
}

fn render_river_sample_values_map(
    ui: &mut egui::Ui,
    region_pos: RegionPos,
    camera_tile_pos: Option<GlobalTilePos>,
    river_info: &RiverRegionDebugInfo,
) {
    let size = egui::vec2(420.0, 420.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 18, 18));

    if river_info.sampled_points.is_empty() {
        ui.colored_label(egui::Color32::RED, "No sampled points captured for this region yet.");
        return;
    }

    let (base_min_chunk, base_max_chunk_excl) = region_pos.chunk_bounds();
    let base_min_tile = base_min_chunk.to_tilepos();
    let base_max_tile_excl = base_max_chunk_excl.to_tilepos();

    let mut min_tile = base_min_tile;
    let mut max_tile_excl = base_max_tile_excl;
    let mut min_sample_val = f32::INFINITY;
    let mut max_sample_val = f32::NEG_INFINITY;
    for (tile, sampled_val) in &river_info.sampled_points {
        min_tile.0.x = min_tile.0.x.min(tile.0.x);
        min_tile.0.y = min_tile.0.y.min(tile.0.y);
        max_tile_excl.0.x = max_tile_excl.0.x.max(tile.0.x + 1);
        max_tile_excl.0.y = max_tile_excl.0.y.max(tile.0.y + 1);
        min_sample_val = min_sample_val.min(*sampled_val);
        max_sample_val = max_sample_val.max(*sampled_val);
    }
    if !min_sample_val.is_finite() || !max_sample_val.is_finite() {
        min_sample_val = 0.0;
        max_sample_val = 0.0;
    }

    let span_x = (max_tile_excl.0.x - min_tile.0.x).max(1) as f32;
    let span_y = (max_tile_excl.0.y - min_tile.0.y).max(1) as f32;

    for (tile, sampled_val) in &river_info.sampled_points {
        let nx = ((tile.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 0.9999);
        let ny = ((tile.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 0.9999);
        let px = rect.left() + nx * rect.width();
        let py = rect.bottom() - ny * rect.height();
        let dot = egui::Rect::from_center_size(egui::pos2(px, py), egui::vec2(2.0, 2.0));
        painter.rect_filled(dot, 0.0, sample_value_color(*sampled_val, min_sample_val, max_sample_val));
    }

    for chunk in &river_info.failed_chunks {
        let center = chunk.to_tilepos() + IVec2::new((ChunkPos::CHUNK_SIZE.x / 2) as i32, (ChunkPos::CHUNK_SIZE.y / 2) as i32);
        let nx = ((center.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 0.9999);
        let ny = ((center.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 0.9999);
        let px = rect.left() + nx * rect.width();
        let py = rect.bottom() - ny * rect.height();
        let mark = egui::Rect::from_center_size(egui::pos2(px, py), egui::vec2(5.0, 5.0));
        painter.rect_filled(mark, 0.0, egui::Color32::RED);
    }

    if let Some(camera_tile_pos) = camera_tile_pos
        && camera_tile_pos.0.x >= min_tile.0.x
        && camera_tile_pos.0.x < max_tile_excl.0.x
        && camera_tile_pos.0.y >= min_tile.0.y
        && camera_tile_pos.0.y < max_tile_excl.0.y
    {
        let nx = ((camera_tile_pos.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 0.9999);
        let ny = ((camera_tile_pos.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 0.9999);
        let px = rect.left() + nx * rect.width();
        let py = rect.bottom() - ny * rect.height();
        let marker = egui::Rect::from_center_size(egui::pos2(px, py), egui::vec2(6.0, 6.0));
        painter.rect_stroke(
            marker,
            0.0,
            egui::Stroke::new(1.5, egui::Color32::YELLOW),
            egui::StrokeKind::Outside,
        );
    }

    let region_nx_min = ((base_min_tile.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 1.0);
    let region_ny_min = ((base_min_tile.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 1.0);
    let region_nx_max = ((base_max_tile_excl.0.x - min_tile.0.x) as f32 / span_x).clamp(0.0, 1.0);
    let region_ny_max = ((base_max_tile_excl.0.y - min_tile.0.y) as f32 / span_y).clamp(0.0, 1.0);
    let region_rect = egui::Rect::from_min_max(
        egui::pos2(
            rect.left() + region_nx_min * rect.width(),
            rect.bottom() - region_ny_max * rect.height(),
        ),
        egui::pos2(
            rect.left() + region_nx_max * rect.width(),
            rect.bottom() - region_ny_min * rect.height(),
        ),
    );
    painter.rect_stroke(
        region_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(180)),
        egui::StrokeKind::Inside,
    );

    ui.label(format!(
        "Legend: altitude color=inlandness [{:.3}..{:.3}], red=failed chunk centers, yellow=current camera tile, gray=selected region bounds",
        min_sample_val,
        max_sample_val
    ));
}

fn sample_value_color(value: f32, min_val: f32, max_val: f32) -> egui::Color32 {
    let t = if (max_val - min_val).abs() < f32::EPSILON {
        0.5
    } else {
        ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
    };
    let (a, b, local_t) = if t < 0.25 {
        ((10, 38, 112), (40, 140, 210), t / 0.25)
    } else if t < 0.5 {
        ((40, 140, 210), (52, 176, 90), (t - 0.25) / 0.25)
    } else if t < 0.75 {
        ((52, 176, 90), (194, 172, 76), (t - 0.5) / 0.25)
    } else {
        ((194, 172, 76), (240, 240, 240), (t - 0.75) / 0.25)
    };
    egui::Color32::from_rgb(
        lerp_u8(a.0, b.0, local_t),
        lerp_u8(a.1, b.1, local_t),
        lerp_u8(a.2, b.2, local_t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let af = a as f32;
    let bf = b as f32;
    (af + (bf - af) * t).round().clamp(0.0, 255.0) as u8
}
