use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use game_common::game_common_timers::{DespawnOnTimeout, MessageOnTimeout};
use std::collections::{BTreeMap, HashMap};

use camera::camera_components::CameraTarget;
use common::common_components::*;
use regioning::natural::river::{RiverDebugData, RiverRegionDebugInfo, RiverRegionPlan};
use tilemap_shared::*;

use debug_shared::{DebugChunkingUiState, DebugSelectedEntities, DubugWindowsVisibility};
use crate::dimension_changer_window::dimension_changer_button;

fn region_dim_key_for_ref(
    dim_ref: &DimensionRef,
    dimension_map: &DimensionEntityMap,
    id_query: &Query<&StrId>,
) -> String {
    let Some(dim_ent) = dimension_map.0.get_cloned(dim_ref.0).ok() else {
        return format!("{:?}", dim_ref);
    };
    if let Ok(str_id) = id_query.get(dim_ent) {
        format!("{} ({})", str_id.as_str(), dim_ent.index())
    } else {
        format!("{:?} ({})", dim_ref, dim_ent.index())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RiverSamplePreviewKey {
    dimension_ref: DimensionRef,
    region_pos: RegionPos,
    debug_revision: u64,
    plan_revision: u64,
    show_sources: bool,
    show_mouths: bool,
    show_failed_centers: bool,
}

#[derive(Default)]
pub struct RiverSamplePreviewCache {
    key: Option<RiverSamplePreviewKey>,
    texture: Option<egui::TextureHandle>,
    image_size: [usize; 2],
    min_tile: IVec2,
    sample_step: IVec2,
    display_min: f32,
    display_max: f32,
    raw_min: f32,
    raw_max: f32,
}

impl RiverSamplePreviewCache {
    fn clear(&mut self) {
        self.key = None;
        self.texture = None;
    }
}

struct RiverSamplePreviewBuild {
    image: egui::ColorImage,
    image_size: [usize; 2],
    min_tile: IVec2,
    sample_step: IVec2,
    display_min: f32,
    display_max: f32,
    raw_min: f32,
    raw_max: f32,
}

fn build_river_sample_preview(
    _dimension_ref: DimensionRef,
    region_pos: RegionPos,
    _river_debug: &RiverDebugData,
    river_info: Option<&RiverRegionDebugInfo>,
    river_plan: Option<&RiverRegionPlan>,
    show_sources: bool,
    show_mouths: bool,
    show_failed_centers: bool,
) -> RiverSamplePreviewBuild {
    let (base_min_chunk, base_max_chunk_excl) = region_pos.chunk_bounds();
    let base_min_tile = base_min_chunk.to_tilepos();
    let base_max_tile_excl = base_max_chunk_excl.to_tilepos();

    let mut min_tile = base_min_tile;
    let mut max_tile_excl = base_max_tile_excl;
    let mut min_sample_val = f32::INFINITY;
    let mut max_sample_val = f32::NEG_INFINITY;
    let mut sample_values: Vec<f32> = Vec::new();
    if let Some(info) = river_info {
        sample_values.reserve(info.sampled_points.len());
        for (tile, sampled_val) in &info.sampled_points {
            if !region_pos.contains_chunkpos(tile.to_chunkpos()) {
                continue;
            }
            min_tile.0.x = min_tile.0.x.min(tile.0.x);
            min_tile.0.y = min_tile.0.y.min(tile.0.y);
            max_tile_excl.0.x = max_tile_excl.0.x.max(tile.0.x + 1);
            max_tile_excl.0.y = max_tile_excl.0.y.max(tile.0.y + 1);
            min_sample_val = min_sample_val.min(*sampled_val);
            max_sample_val = max_sample_val.max(*sampled_val);
            sample_values.push(*sampled_val);
        }
    }
    if !min_sample_val.is_finite() || !max_sample_val.is_finite() {
        min_sample_val = 0.0;
        max_sample_val = 0.0;
    }
    let (display_min_val, display_max_val) = robust_display_range(&mut sample_values, min_sample_val, max_sample_val);

    let mut all_sampled_points: bevy::platform::collections::HashMap<GlobalTilePos, f32> =
        bevy::platform::collections::HashMap::default();
    if let Some(info) = river_info {
        all_sampled_points.extend(
            info.sampled_points
                .iter()
                .filter(|(pos, _)| region_pos.contains_chunkpos(pos.to_chunkpos()))
                .map(|(pos, val)| (*pos, *val)),
        );
    }
    let (sample_step_x, sample_step_y) = estimate_sample_step_tiles(&all_sampled_points);
    let image_width = ((max_tile_excl.0.x - min_tile.0.x + sample_step_x - 1) / sample_step_x).max(1) as usize;
    let image_height = ((max_tile_excl.0.y - min_tile.0.y + sample_step_y - 1) / sample_step_y).max(1) as usize;
    let mut image = egui::ColorImage::new(
        [image_width, image_height],
        vec![egui::Color32::from_rgb(18, 18, 18); image_width.saturating_mul(image_height)],
    );
    let step_x = sample_step_x.max(1);
    let step_y = sample_step_y.max(1);
    let set_cell = |image: &mut egui::ColorImage, cell_x: usize, cell_y: usize, color: egui::Color32| {
        if cell_x < image_width && cell_y < image_height {
            image.pixels[cell_y * image_width + cell_x] = color;
        }
    };

    if let Some(info) = river_info {
        for (tile, sampled_val) in &info.sampled_points {
            if !region_pos.contains_chunkpos(tile.to_chunkpos()) {
                continue;
            }
            let cell_x = ((tile.0.x - min_tile.0.x) / step_x).max(0) as usize;
            let cell_y = image_height
                .saturating_sub(1)
                .saturating_sub(((tile.0.y - min_tile.0.y) / step_y).max(0) as usize);
            set_cell(
                &mut image,
                cell_x,
                cell_y,
                sample_value_color(*sampled_val, min_sample_val, max_sample_val),
            );
        }
    }
    if let Some(plan) = river_plan {
        for tile in plan.iter_river_tiles_sorted() {
            if !region_pos.contains_chunkpos(tile.to_chunkpos()) {
                continue;
            }
            let cell_x = ((tile.0.x - min_tile.0.x) / step_x).max(0) as usize;
            let cell_y = image_height
                .saturating_sub(1)
                .saturating_sub(((tile.0.y - min_tile.0.y) / step_y).max(0) as usize);
            set_cell(&mut image, cell_x, cell_y, egui::Color32::from_rgb(45, 160, 255));
        }
    }
    if show_failed_centers && let Some(info) = river_info {
        for chunk in &info.failed_chunks {
            let center = chunk.to_tilepos() + IVec2::new((ChunkPos::CHUNK_SIZE.x / 2) as i32, (ChunkPos::CHUNK_SIZE.y / 2) as i32);
            if !region_pos.contains_chunkpos(center.to_chunkpos()) {
                continue;
            }
            let cell_x = ((center.0.x - min_tile.0.x) / step_x).max(0) as usize;
            let cell_y = image_height
                .saturating_sub(1)
                .saturating_sub(((center.0.y - min_tile.0.y) / step_y).max(0) as usize);
            set_cell(&mut image, cell_x, cell_y, egui::Color32::RED);
        }
    }
    if show_sources {
        if let Some(river_info) = river_info {
            for src in &river_info.river_source_points {
                let cell_x = ((src.0.x - min_tile.0.x) / step_x).max(0) as usize;
                let cell_y = image_height
                    .saturating_sub(1)
                    .saturating_sub(((src.0.y - min_tile.0.y) / step_y).max(0) as usize);
                set_cell(&mut image, cell_x, cell_y, egui::Color32::from_rgb(64, 255, 96));
            }
        }
    }
    if show_mouths {
        if let Some(river_info) = river_info {
            for mouth in &river_info.river_mouth_points {
                let cell_x = ((mouth.0.x - min_tile.0.x) / step_x).max(0) as usize;
                let cell_y = image_height
                    .saturating_sub(1)
                    .saturating_sub(((mouth.0.y - min_tile.0.y) / step_y).max(0) as usize);
                set_cell(&mut image, cell_x, cell_y, egui::Color32::from_rgb(255, 230, 64));
            }
        }
    }
    if let Some(river_info) = river_info {
        for failed in &river_info.failed_probe_points {
            let cell_x = ((failed.0.x - min_tile.0.x) / step_x).max(0) as usize;
            let cell_y = image_height
                .saturating_sub(1)
                .saturating_sub(((failed.0.y - min_tile.0.y) / step_y).max(0) as usize);
            set_cell(&mut image, cell_x, cell_y, egui::Color32::from_rgb(255, 120, 64));
        }
    }

    RiverSamplePreviewBuild {
        image,
        image_size: [image_width, image_height],
        min_tile: min_tile.0,
        sample_step: IVec2::new(step_x, step_y),
        display_min: display_min_val,
        display_max: display_max_val,
        raw_min: min_sample_val,
        raw_max: max_sample_val,
    }
}

fn summarize_global_tile_set(points: &bevy::platform::collections::HashSet<GlobalTilePos>) -> String {
    if points.is_empty() {
        return "none".to_string();
    }
    let mut tiles = points.iter().copied().collect::<Vec<_>>();
    tiles.sort_unstable_by_key(|tile| (tile.0.y, tile.0.x));
    tiles
        .into_iter()
        .map(|tile| format!("({},{})", tile.0.x, tile.0.y))
        .collect::<Vec<_>>()
        .join(", ")
}

fn summarize_chunk_set(chunks: &bevy::platform::collections::HashSet<ChunkPos>) -> String {
    if chunks.is_empty() {
        return "none".to_string();
    }
    let mut chunks = chunks.iter().copied().collect::<Vec<_>>();
    chunks.sort_unstable_by_key(|chunk| (chunk.0.y, chunk.0.x));
    chunks
        .into_iter()
        .map(|chunk| format!("({},{})", chunk.0.x, chunk.0.y))
        .collect::<Vec<_>>()
        .join(", ")
}

fn summarize_mouth_fail_stats(river_info: &RiverRegionDebugInfo) -> Vec<(String, u32, f32)> {
    let total = river_info.mouth_reject_stats.total_rejections.max(1) as f32;
    let mut entries = river_info
        .mouth_reject_stats
        .counts
        .iter()
        .map(|(reason, count)| (format!("{:?}", reason), *count, (*count as f32 / total) * 100.0))
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries
}

#[allow(unused_parens, )]
pub fn regions_list_window(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut chunking_ui: ResMut<DebugChunkingUiState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut regions_list_was_open: Local<bool>,
    mut river_sample_preview: Local<RiverSamplePreviewCache>,
    region_query: Query<(
        Entity,
        &Region,
        &DimensionRef,
        &RegionPos,
        Option<&Name>,
        Option<&GridOfSgcs>,
        Option<&ClaimList>,
        Option<&RegionPlannedTiles>,
        Option<&ActiveChunksInRegion>,
        Option<&CountsOfSgcs>,
        Option<&RiverRegionPlan>,
        &RegionState,
        Has<MessageOnTimeout>,
        Has<DespawnOnTimeout>,
    ), With<Region>>,
    camera_dimension: Query<(Entity, &DimensionRef, &GlobalTransform, Has<GlobalTilePos>), With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
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
    let mut regions_by_dimension: BTreeMap<String, (Entity, HashMap<RegionPos, (Entity, Option<&Name>, Option<&GridOfSgcs>, Option<&ClaimList>, Option<&RegionPlannedTiles>, Option<&ActiveChunksInRegion>, Option<&CountsOfSgcs>, RegionState, bool, bool)>)> =
        BTreeMap::new();

    for (entity, _region, dim_ref, region_pos, name, grid, claim_list, planned_tiles, chunks_active, counts, _river_plan, &region_state, timeout_timer, empty_timer) in region_query.iter() {
        let dim_key = region_dim_key_for_ref(dim_ref, &dimension_map, &id_query);

        regions_by_dimension
            .entry(dim_key.clone())
            .or_insert_with(|| {
                let dim_ent = dimension_map
                    .0
                    .get_cloned(dim_ref.0)
                    .ok()
                    .unwrap_or(Entity::PLACEHOLDER);
                (dim_ent, HashMap::new())
            })
            .1
            .insert(*region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, region_state, timeout_timer, empty_timer));
    }

    // Get camera target dimension and position
    let (camera_entity, camera_dim_ref, camera_chunk_pos, camera_tile_pos, camera_region_pos, camera_has_gpos) = camera_dimension.iter().next()
        .map(|(entity, dim_ref, transform, has_gpos)| {
            let chunk_pos = ChunkPos::from(transform.translation());
            let tile_pos = GlobalTilePos::from(transform.translation().xy());
            let region_pos = chunk_pos.to_region_pos();
            (Some(entity), Some(dim_ref), Some(chunk_pos), Some(tile_pos), Some(region_pos), Some(has_gpos))
        })
        .unwrap_or((None, None, None, None, None, None));

    let opening_now = window_visible.regions_list && !*regions_list_was_open;
    if opening_now
        && let (Some(cam_dim_ref), Some(cam_region_pos)) = (camera_dim_ref, camera_region_pos)
        && let Some((entity, ..)) = region_query.iter().find(|(_, _, dim_ref, region_pos, ..)| {
            *dim_ref == cam_dim_ref && **region_pos == cam_region_pos
        })
    {
        selected_entities.selected_regions.clear();
        selected_entities.selected_regions.insert(entity);
        selected_entities.selected_river_debug_region = Some(entity);
    }

    if chunking_ui.follow_camera_region
        && let (Some(cam_dim_ref), Some(cam_region_pos)) = (camera_dim_ref, camera_region_pos)
        && let Some((entity, ..)) = region_query.iter().find(|(_, _, dim_ref, region_pos, ..)| {
            *dim_ref == cam_dim_ref && **region_pos == cam_region_pos
        })
    {
        selected_entities.selected_regions.clear();
        selected_entities.selected_regions.insert(entity);
        selected_entities.selected_river_debug_region = Some(entity);
    }

    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = regions_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        let camera_dim_key = region_dim_key_for_ref(camera_ref, &dimension_map, &id_query);
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
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        dimension_changer_button(ui, &mut window_visible);
                        ui.add_sized(
                            [280.0, 32.0],
                            egui::Checkbox::new(
                                &mut chunking_ui.follow_camera_region,
                                egui::RichText::new("Follow Camera Region").size(18.0).strong(),
                            ),
                        );
                        let status = if chunking_ui.follow_camera_region { "ON" } else { "OFF" };
                        let status_color = if chunking_ui.follow_camera_region {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::LIGHT_RED
                        };
                        ui.label(egui::RichText::new(status).strong().color(status_color));
                    });
                });
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some((_, regions_map)) = regions_by_dimension.get(dim_key) {
                    let is_camera_dim = camera_dim_ref.map_or(false, |camera_ref| {
                        dim_key == &region_dim_key_for_ref(camera_ref, &dimension_map, &id_query)
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
                                                    if !(is_camera_dim && is_camera_pos) {
                                                        chunking_ui.follow_camera_region = false;
                                                    }
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
                        });
                        ui.separator();
                        let highlight_pos = if let (Some(cam_pos), Some(cam_dim)) = (camera_chunk_pos, camera_dim_ref) {
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
                        let mut render_grid_panel = |ui: &mut egui::Ui,
                                                 grid: Option<&GridOfSgcs>,
                                                 highlight_pos: Option<ChunkPos>,
                                                 region_pos: RegionPos,
                                                 selected_entities: &mut DebugSelectedEntities,
                                                 window_visible: &mut DubugWindowsVisibility| {
                            if let Some(grid_sgcs) = grid {
                                ui.label("GridOfSgcs:");
                                ui.indent("grid_sgcs", |ui| {
                                    if let Some((clicked_sgc_ent_opt, clicked_chunk_pos)) = grid_sgcs.render_grid(ui, highlight_pos, Some(region_pos)) {
                                        if let Some(clicked_sgc_ent) = clicked_sgc_ent_opt {
                                            selected_entities.selected_exempted_entity = Some(clicked_sgc_ent);
                                            selected_entities.selected_tile = None;
                                            window_visible.tile_details = true;
                                        }
                                        if let Some(camera_entity) = camera_entity {
                                            let center_gpos = clicked_chunk_pos.center_gpos();
                                            commands.entity(camera_entity).insert(Transform::from_translation(center_gpos.to_pixelpos().extend(0.0)));
                                            if camera_has_gpos == Some(true) {
                                                commands.entity(camera_entity).insert(center_gpos);
                                            }
                                        }
                                    }
                                });
                            }
                        };
                        let render_region_stats = |ui: &mut egui::Ui| {
                            if let Some(claim) = claim_list {
                                ui.label(format!("ClaimList: {}/{}", claim.processed_up_to_i, MAX_CHUNK_CLAIMS_PER_REGION));
                            }

                            if let Some(planned) = planned_tiles {
                                ui.label(format!("PlannedTiles pending: {}", planned.pending_chunks_count()));
                            }

                            if let Some(chunks) = chunks_active {
                                ui.label(format!("ChunksActive: {}", chunks.iter().len()));
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
                        };

                        if ui.available_width() < 560.0 {
                            render_grid_panel(ui, *grid, highlight_pos, *region_pos, &mut selected_entities, &mut window_visible);
                            ui.separator();
                            render_region_stats(ui);
                        } else {
                            ui.columns(2, |columns| {
                                render_grid_panel(&mut columns[0], *grid, highlight_pos, *region_pos, &mut selected_entities, &mut window_visible);
                                render_region_stats(&mut columns[1]);
                            });
                        }

                    });
            }
        });
    window_visible.regions_list = open;
    *regions_list_was_open = window_visible.regions_list;

    if window_visible.river_debug {
        let mut river_open = window_visible.river_debug;
        egui::Window::new("River Debug")
            .default_pos([screen_rect.left() + 24.0, screen_rect.top() + 120.0])
            .resizable(true)
            .movable(true)
            .vscroll(true)
            .open(&mut river_open)
            .show(ctx, |ui| {
                let Some(region_ent) = selected_entities.selected_river_debug_region else {
                    river_sample_preview.clear();
                    ui.colored_label(egui::Color32::RED, "No region selected. Use the River Debug button in Regions Grid.");
                    return;
                };
                let Ok(region_data) = region_query.get(region_ent) else {
                    river_sample_preview.clear();
                    ui.colored_label(egui::Color32::RED, "Selected region no longer exists.");
                    return;
                };
                let dim_ref = region_data.2;
                let region_pos = region_data.3;
                let name = region_data.4;
                let title_name = name.map(|n| n.to_string()).unwrap_or_else(|| "unnamed".to_string());
                ui.label(format!("Region: {} ({:?})", title_name, region_pos));
                ui.label(format!("Entity: {:?}", region_ent));
                ui.label(format!("Dimension: {:?}", dim_ref));
                ui.separator();

                let Some(river_debug) = river_debug.as_ref() else {
                    river_sample_preview.clear();
                    ui.colored_label(egui::Color32::RED, "River debug resource unavailable.");
                    return;
                };
                let Some(river_info) = river_debug.data.get(&(*dim_ref, *region_pos)) else {
                    river_sample_preview.clear();
                    ui.colored_label(egui::Color32::RED, "No river debug data for this region yet.");
                    return;
                };
                let river_plan = region_data.10;
                ui.horizontal(|ui| {
                    ui.label(format!("successes: {}", river_info.success_count));
                    ui.label(egui::RichText::new(format!("failures: {}", river_info.failure_count)).color(egui::Color32::RED));
                    ui.label(format!("active probes: {}", river_info.active_probe_chunks.len()));
                    ui.label(format!("river tiles: {}", river_plan.map(|plan| plan.river_tile_count()).unwrap_or(0)));
                    ui.label(format!("sampled points: {}", river_info.sampled_points.len()));
                });
                ui.separator();
                ui.label("River Sample Values");
                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(
                        &mut selected_entities.river_samples_show_sources,
                        "Show sources",
                    );
                    ui.checkbox(
                        &mut selected_entities.river_samples_show_mouths,
                        "Show mouths",
                    );
                    ui.checkbox(
                        &mut selected_entities.river_samples_show_region_bounds,
                        "Show region bounds",
                    );
                    ui.checkbox(
                        &mut selected_entities.river_samples_show_failed_centers,
                        "Show failed centers",
                    );
                });
                let camera_tile_in_region = if camera_dim_ref == Some(dim_ref) {
                    camera_tile_pos
                } else {
                    None
                };
                let key = RiverSamplePreviewKey {
                    dimension_ref: *dim_ref,
                    region_pos: *region_pos,
                    debug_revision: river_debug.revision,
                    plan_revision: river_plan.map(|plan| plan.river_tile_count() as u64).unwrap_or(0),
                    show_sources: selected_entities.river_samples_show_sources,
                    show_mouths: selected_entities.river_samples_show_mouths,
                    show_failed_centers: selected_entities.river_samples_show_failed_centers,
                };
                if river_sample_preview.key != Some(key) {
                    let preview = build_river_sample_preview(
                        *dim_ref,
                        *region_pos,
                        river_debug,
                        Some(river_info),
                        river_plan,
                        selected_entities.river_samples_show_sources,
                        selected_entities.river_samples_show_mouths,
                        selected_entities.river_samples_show_failed_centers,
                    );
                    let texture_name = format!(
                        "river_debug_sample_preview_{:?}_{:?}",
                        dim_ref.0,
                        region_pos
                    );
                    river_sample_preview.texture = Some(ctx.load_texture(
                        texture_name,
                        preview.image,
                        egui::TextureOptions::NEAREST,
                    ));
                    river_sample_preview.key = Some(key);
                    river_sample_preview.image_size = preview.image_size;
                    river_sample_preview.min_tile = preview.min_tile;
                    river_sample_preview.sample_step = preview.sample_step;
                    river_sample_preview.display_min = preview.display_min;
                    river_sample_preview.display_max = preview.display_max;
                    river_sample_preview.raw_min = preview.raw_min;
                    river_sample_preview.raw_max = preview.raw_max;
                }
                let Some(texture) = river_sample_preview.texture.as_ref() else {
                    ui.colored_label(egui::Color32::RED, "River preview texture could not be created.");
                    return;
                };
                let aspect = (river_sample_preview.image_size[0] as f32 / river_sample_preview.image_size[1] as f32).max(0.01);
                let mut map_w = ui.available_width().clamp(240.0, 900.0);
                let mut map_h = (map_w / aspect).clamp(200.0, 700.0);
                if map_w / aspect > map_h {
                    map_w = map_h * aspect;
                } else {
                    map_h = map_w / aspect;
                }
                let (rect, _) = ui.allocate_exact_size(egui::vec2(map_w.max(120.0), map_h.max(120.0)), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 18, 18));
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                let cell_w = rect.width() / river_sample_preview.image_size[0] as f32;
                let cell_h = rect.height() / river_sample_preview.image_size[1] as f32;
                let draw_cell = |tile: IVec2, color: egui::Color32, stroke: f32, painter: &egui::Painter| {
                    let cell_x = ((tile.x - river_sample_preview.min_tile.x).max(0) / river_sample_preview.sample_step.x.max(1)) as usize;
                    let cell_y = river_sample_preview
                        .image_size[1]
                        .saturating_sub(1)
                        .saturating_sub(((tile.y - river_sample_preview.min_tile.y).max(0) / river_sample_preview.sample_step.y.max(1)) as usize);
                    let x = rect.left() + cell_x as f32 * cell_w;
                    let y = rect.top() + cell_y as f32 * cell_h;
                    let cell_rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_w, cell_h));
                    painter.rect_stroke(cell_rect.shrink(0.5), 0.0, egui::Stroke::new(stroke, color), egui::StrokeKind::Inside);
                };
                let draw_dot = |tile: IVec2, color: egui::Color32, radius: f32, painter: &egui::Painter| {
                    let cell_x = ((tile.x - river_sample_preview.min_tile.x).max(0) / river_sample_preview.sample_step.x.max(1)) as usize;
                    let cell_y = river_sample_preview
                        .image_size[1]
                        .saturating_sub(1)
                        .saturating_sub(((tile.y - river_sample_preview.min_tile.y).max(0) / river_sample_preview.sample_step.y.max(1)) as usize);
                    let center_x = rect.left() + (cell_x as f32 + 0.5) * cell_w;
                    let center_y = rect.top() + (cell_y as f32 + 0.5) * cell_h;
                    painter.circle_filled(egui::pos2(center_x, center_y), radius, color);
                };
                let draw_region_rect = |region_pos: RegionPos, fill: egui::Color32, stroke: egui::Color32, stroke_w: f32, painter: &egui::Painter| {
                    let (base_min_chunk, base_max_chunk_excl) = region_pos.chunk_bounds();
                    let base_min_tile = base_min_chunk.to_tilepos();
                    let base_max_tile_excl = base_max_chunk_excl.to_tilepos();
                    let min_local_x = ((base_min_tile.0.x - river_sample_preview.min_tile.x).max(0) / river_sample_preview.sample_step.x.max(1)) as usize;
                    let min_local_y = ((base_min_tile.0.y - river_sample_preview.min_tile.y).max(0) / river_sample_preview.sample_step.y.max(1)) as usize;
                    let max_local_x = ((base_max_tile_excl.0.x - river_sample_preview.min_tile.x).max(0) / river_sample_preview.sample_step.x.max(1)) as usize;
                    let max_local_y = ((base_max_tile_excl.0.y - river_sample_preview.min_tile.y).max(0) / river_sample_preview.sample_step.y.max(1)) as usize;
                    let region_rect = egui::Rect::from_min_max(
                        egui::pos2(
                            rect.left() + min_local_x as f32 * cell_w,
                            rect.top() + min_local_y as f32 * cell_h,
                        ),
                        egui::pos2(
                            rect.left() + max_local_x as f32 * cell_w,
                            rect.top() + max_local_y as f32 * cell_h,
                        ),
                    );
                    painter.rect_filled(region_rect, 0.0, fill);
                    painter.rect_stroke(
                        region_rect,
                        0.0,
                        egui::Stroke::new(stroke_w, stroke),
                        egui::StrokeKind::Inside,
                    );
                    region_rect
                };
                if selected_entities.river_samples_show_region_bounds {
                    draw_region_rect(
                        *region_pos,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0),
                        egui::Color32::from_gray(210),
                        1.8,
                        &painter,
                    );
                }
                if let Some(camera_tile_pos) = camera_tile_in_region {
                    let dot_radius = (cell_w.min(cell_h) * 0.38).clamp(2.0, 7.5);
                    draw_dot(camera_tile_pos.0, egui::Color32::RED, dot_radius, &painter);
                    draw_cell(camera_tile_pos.0, egui::Color32::from_rgb(255, 210, 210), 1.0, &painter);
                }
                render_river_sample_values_map(
                    ui,
                    *region_pos,
                    river_sample_preview.display_min,
                    river_sample_preview.display_max,
                    river_sample_preview.raw_min,
                    river_sample_preview.raw_max,
                );
                ui.separator();
                ui.label("River Components");
                ui.monospace(format!("active probe chunks: {}", summarize_chunk_set(&river_info.active_probe_chunks)));
                ui.monospace(format!("failed chunks: {}", summarize_chunk_set(&river_info.failed_chunks)));
                ui.monospace(format!("river source points: {}", summarize_global_tile_set(&river_info.river_source_points)));
                ui.monospace(format!("river mouth points: {}", summarize_global_tile_set(&river_info.river_mouth_points)));
                ui.separator();
                ui.label("River Mouth Fail Causes");
                ui.monospace(format!("total rejects: {}", river_info.mouth_reject_stats.total_rejections));
                let mouth_fail_stats = summarize_mouth_fail_stats(river_info);
                if mouth_fail_stats.is_empty() {
                    ui.monospace("none");
                } else {
                    for (reason, count, percent) in mouth_fail_stats {
                        ui.monospace(format!("{}: {} ({:.1}%)", reason, count, percent));
                    }
                }
                ui.monospace(format!("sampled points: {}", river_info.sampled_points.len()));
                if let Some(plan) = river_plan {
                    ui.monospace(format!("river tiles in plan: {}", plan.river_tile_count()));
                } else {
                    ui.monospace("river tiles in plan: none");
                }
            });
        window_visible.river_debug = river_open;
    }
}

fn render_river_sample_values_map(
    ui: &mut egui::Ui,
    region_pos: RegionPos,
    display_min: f32,
    display_max: f32,
    raw_min: f32,
    raw_max: f32,
) {
    ui.label(format!("Region preview for {:?}", region_pos));
    ui.label(format!(
        "Legend: altitude color=inlandness (black=lowest, white=highest), robust range [{:.3}..{:.3}]",
        display_min,
        display_max
    ));
    ui.label(format!("Raw inlandness range: [{:.3}..{:.3}]", raw_min, raw_max));
}

fn sample_value_color(rank: f32, min_val: f32, max_val: f32) -> egui::Color32 {
    let t = if (max_val - min_val).abs() <= f32::EPSILON {
        1.0
    } else {
        ((rank - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
    };
    let g = (t * 255.0) as u8;
    egui::Color32::from_rgb(g, g, g)
}

fn robust_display_range(values: &mut [f32], fallback_min: f32, fallback_max: f32) -> (f32, f32) {
    if values.is_empty() {
        return (fallback_min, fallback_max);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let last = values.len().saturating_sub(1);
    let lo_i = ((last as f32) * 0.02).round() as usize;
    let hi_i = ((last as f32) * 0.98).round() as usize;
    let lo = values[lo_i.min(last)];
    let hi = values[hi_i.min(last)];
    if (hi - lo).abs() < f32::EPSILON {
        (fallback_min, fallback_max)
    } else {
        (lo, hi)
    }
}

fn estimate_sample_step_tiles(sampled_points: &bevy::platform::collections::HashMap<GlobalTilePos, f32>) -> (i32, i32) {
    let mut xs: Vec<i32> = sampled_points.keys().map(|p| p.0.x).collect();
    let mut ys: Vec<i32> = sampled_points.keys().map(|p| p.0.y).collect();
    xs.sort_unstable();
    ys.sort_unstable();
    xs.dedup();
    ys.dedup();
    let step_x = xs
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(1)
        .max(1);
    let step_y = ys
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(1)
        .max(1);
    (step_x, step_y)
}
