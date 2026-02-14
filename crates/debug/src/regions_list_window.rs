use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use std::collections::{BTreeMap, HashMap};

use camera::camera_components::CameraTarget;
use common::common_components::*;
use game_common::game_common_components::DespawnTimer;
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
        Option<&PendingOfferTimeout>,
        Option<&DespawnTimer>,
        Has<AllTilesPrepared>,
        Has<BuildingStarted>,
        Has<AllClaimsProcessed>,
    ), With<Region>>,
    camera_dimension: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    id_query: Query<&StrId>,
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
    let mut regions_by_dimension: BTreeMap<String, (Entity, HashMap<RegionPos, (Entity, Option<&Name>, Option<&GridOfSgcs>, Option<&ClaimList>, Option<&RegionPlannedTiles>, Option<&ChunksActiveInRegion>, Option<&CountsOfSgcs>, Option<&PendingOfferTimeout>, Option<&DespawnTimer>, bool, bool, bool)>)> =
        BTreeMap::new();

    for (entity, _region, dim_ref, region_pos, name, grid, claim_list, planned_tiles, chunks_active, counts, pending_timeout, empty_timer, has_all_tiles, has_building_started, has_all_claims) in region_query.iter() {
        let dim_key = if let Ok(str_id) = id_query.get(dim_ref.0) {
            format!("{} ({})", str_id.as_str(), dim_ref.0.index())
        } else {
            format!("{:?} ({})", dim_ref, dim_ref.0.index())
        };

        regions_by_dimension
            .entry(dim_key.clone())
            .or_insert_with(|| (dim_ref.0, HashMap::new()))
            .1
            .insert(*region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, pending_timeout, empty_timer, has_all_tiles, has_building_started, has_all_claims));
    }

    // Get camera target dimension and position
    let (camera_dim_ref, camera_chunk_pos, camera_region_pos) = camera_dimension.iter().next()
        .map(|(dim_ref, transform)| {
            let chunk_pos = ChunkPos::from(transform.translation());
            let region_pos = chunk_pos.to_region_pos();
            (Some(dim_ref), Some(chunk_pos), Some(region_pos))
        })
        .unwrap_or((None, None, None));

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

            for (region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, pending_timeout, empty_timer, has_all_tiles, has_building_started, has_all_claims)) in selected_region_details {
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
                                        grid_sgcs.render_grid(ui, highlight_pos, Some(*region_pos));
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

                                if pending_timeout.is_some() {
                                    ui.label("⏱ PendingOfferTimeout");
                                }

                                if empty_timer.is_some() {
                                    ui.label("🗑 EmptyRegionDespawnTimer");
                                }

                                if *has_all_tiles {
                                    ui.label("✓ AllTilesPrepared");
                                }

                                if *has_building_started {
                                    ui.label("▶ BuildingStarted");
                                }

                                if *has_all_claims {
                                    ui.label("✓ AllClaimsProcessed");
                                }
                            });
                        });
                    });
            }
        });
    window_visible.regions_list = open;
}
