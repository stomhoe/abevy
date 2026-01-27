use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

use crate::debug_resources::{DubugWindowsVisibility, DebugSelectedEntities};

// Import needed components
use tilemap::chunking_components::Chunk;
use tilemap::regioning::regioning_components::{Region, GridOfSgcs, ClaimList, RegionPlannedTiles, ChunksActiveInRegion, CountsOfSgcs, PendingOfferTimeout, EmptyRegionDespawnTimer, AllTilesPrepared, BuildingStarted, AllClaimsProcessed};
use tilemap_shared::{ChunkPos, RegionPos};
use being::being_components::Being;
use dimension_shared::DimensionRef;
use camera::camera_components::CameraTarget;

#[allow(unused_parens)]
pub fn chunks_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    chunk_query: Query<(Entity, &Chunk, &DimensionRef, &ChunkPos, Option<&Name>, &Children), With<Chunk>>,
    dimension_query: Query<&Name>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
) {
    if !window_visible.chunks_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 10.0;
    let default_y = screen_rect.top() + 10.0;

    // Get camera target dimension if available
    let camera_dim_ref = camera_dimension.iter().next();
    
    // Group chunks by dimension and position
    let mut chunks_by_dimension: BTreeMap<String, HashMap<ChunkPos, (Entity, Option<&Name>, &Children)>> =
        BTreeMap::new();

    for (entity, _chunk, dim_ref, chunk_pos, name, children) in chunk_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };
        
        chunks_by_dimension
            .entry(dim_name)
            .or_insert_with(HashMap::new)
            .insert(*chunk_pos, (entity, name, children));
    }
    
    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = chunks_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        if let Ok(camera_name) = dimension_query.get(camera_ref.0) {
            let camera_dim_str = format!("{}", camera_name);
            sorted_dims.sort_by(|a, b| {
                if a == &camera_dim_str { std::cmp::Ordering::Less }
                else if b == &camera_dim_str { std::cmp::Ordering::Greater }
                else { a.cmp(b) }
            });
        }
    }

    egui::Window::new("Chunks Grid")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(600.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Chunks: {}", chunk_query.iter().count()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        window_visible.chunks_list = false;
                    }
                });
            });
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(chunks_map) = chunks_by_dimension.get(dim_key) {
                    ui.collapsing(dim_key, |ui| {
                        // Create grid of chunk positions
                        let positions: Vec<ChunkPos> = chunks_map.keys().copied().collect();
                        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
                            positions.iter().map(|p| p.0.x).min(),
                            positions.iter().map(|p| p.0.x).max(),
                            positions.iter().map(|p| p.0.y).min(),
                            positions.iter().map(|p| p.0.y).max(),
                        ) {
                            egui::Grid::new(format!("chunks_grid_{}", dim_key))
                                .spacing([5.0, 5.0])
                                .show(ui, |ui| {
                                    for y in (min_y..=max_y).rev() {
                                        for x in min_x..=max_x {
                                            let pos = ChunkPos(IVec2::new(x, y));
                                            if let Some((entity, name, children)) = chunks_map.get(&pos) {
                                                let is_selected = selected_entities.selected_chunks.contains(entity);
                                                let mut label = format!("{},{}\n{} children", x, y, children.len());
                                                if let Some(n) = name {
                                                    label = format!("{}\n{}", label, n);
                                                }
                                                if ui.selectable_label(is_selected, &label).clicked() {
                                                    if is_selected {
                                                        selected_entities.selected_chunks.remove(entity);
                                                    } else {
                                                        selected_entities.selected_chunks.insert(*entity);
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
        });
}

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
        Option<&EmptyRegionDespawnTimer>,
        Has<AllTilesPrepared>,
        Has<BuildingStarted>,
        Has<AllClaimsProcessed>,
    ), With<Region>>,
    dimension_query: Query<&Name>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
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

    // Group regions by dimension and position
    let mut regions_by_dimension: BTreeMap<String, HashMap<RegionPos, (Entity, Option<&Name>, Option<&GridOfSgcs>, Option<&ClaimList>, Option<&RegionPlannedTiles>, Option<&ChunksActiveInRegion>, Option<&CountsOfSgcs>, Option<&PendingOfferTimeout>, Option<&EmptyRegionDespawnTimer>, bool, bool, bool)>> =
        BTreeMap::new();

    for (entity, _region, dim_ref, region_pos, name, grid, claim_list, planned_tiles, chunks_active, counts, pending_timeout, empty_timer, has_all_tiles, has_building_started, has_all_claims) in region_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };
        regions_by_dimension
            .entry(dim_name)
            .or_insert_with(HashMap::new)
            .insert(*region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, pending_timeout, empty_timer, has_all_tiles, has_building_started, has_all_claims));
    }
    
    // Get camera target dimension if available
    let camera_dim_ref = camera_dimension.iter().next();
    
    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = regions_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        if let Ok(camera_name) = dimension_query.get(camera_ref.0) {
            let camera_dim_str = format!("{}", camera_name);
            sorted_dims.sort_by(|a, b| {
                if a == &camera_dim_str { std::cmp::Ordering::Less }
                else if b == &camera_dim_str { std::cmp::Ordering::Greater }
                else { a.cmp(b) }
            });
        }
    }

    egui::Window::new("Regions Grid")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(700.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Regions: {}", region_query.iter().count()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        window_visible.regions_list = false;
                    }
                });
            });
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(regions_map) = regions_by_dimension.get(dim_key) {
                    ui.collapsing(dim_key, |ui| {
                        // Create grid of region positions
                        let positions: Vec<RegionPos> = regions_map.keys().copied().collect();
                        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
                            positions.iter().map(|p| p.0.x).min(),
                            positions.iter().map(|p| p.0.x).max(),
                            positions.iter().map(|p| p.0.y).min(),
                            positions.iter().map(|p| p.0.y).max(),
                        ) {
                            egui::Grid::new(format!("regions_grid_{}", dim_key))
                                .spacing([5.0, 5.0])
                                .show(ui, |ui| {
                                    for y in (min_y..=max_y).rev() {
                                        for x in min_x..=max_x {
                                            let pos = RegionPos(IVec2::new(x, y));
                                            if let Some((entity, name, _grid, _claim_list, _planned_tiles, _chunks_active, _counts, _pending_timeout, _empty_timer, _has_all_tiles, _has_building_started, _has_all_claims)) = regions_map.get(&pos) {
                                                let is_selected = selected_entities.selected_regions.contains(entity);
                                                let mut label = format!("{},{}", x, y);
                                                if let Some(n) = name {
                                                    label = format!("{}\n{}", label, n);
                                                }
                                                
                                                if ui.selectable_label(is_selected, &label).clicked() {
                                                    if is_selected {
                                                        selected_entities.selected_regions.remove(entity);
                                                    } else {
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
                .flat_map(|(_, map)| map.iter())
                .filter(|(_, (entity, ..))| selected_entities.selected_regions.contains(entity))
                .collect();
            selected_region_details.sort_by_key(|(_, (entity, ..))| entity.index());
            
            for (_region_pos, (entity, name, grid, claim_list, planned_tiles, chunks_active, counts, pending_timeout, empty_timer, has_all_tiles, has_building_started, has_all_claims)) in selected_region_details {
                let name_str = name.map(|n| format!("{}", n)).unwrap_or_else(|| "unnamed".to_string());
                    ui.collapsing(format!("Details: {} ({:?})", name_str, entity), |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                if let Some(grid_sgcs) = grid {
                                    ui.label("GridOfSgcs:");
                                    ui.indent("grid_sgcs", |ui| {
                                        grid_sgcs.render_grid(ui);
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
}

#[allow(unused_parens)]
pub fn beings_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    being_query: Query<(Entity, &Being, Option<&Name>), With<Being>>,
    dimension_ref_query: Query<&DimensionRef>,
    dimension_query: Query<&Name>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
) {
    if !window_visible.beings_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 350.0;
    let default_y = screen_rect.top() + 10.0;

    // Group beings by dimension
    let mut beings_by_dimension: BTreeMap<String, Vec<(Entity, Option<&Name>)>> = BTreeMap::new();

    for (entity, _being, name) in being_query.iter() {
        if let Ok(dim_ref) = dimension_ref_query.get(entity) {
            let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
                format!("{}", n)
            } else {
                format!("{:?}", dim_ref)
            };
            beings_by_dimension
                .entry(dim_name)
                .or_insert_with(Vec::new)
                .push((entity, name));
        }
    }
    
    // Get camera target dimension if available
    let camera_dim_ref = camera_dimension.iter().next();
    
    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = beings_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        if let Ok(camera_name) = dimension_query.get(camera_ref.0) {
            let camera_dim_str = format!("{}", camera_name);
            sorted_dims.sort_by(|a, b| {
                if a == &camera_dim_str { std::cmp::Ordering::Less }
                else if b == &camera_dim_str { std::cmp::Ordering::Greater }
                else { a.cmp(b) }
            });
        }
    }

    egui::Window::new("Beings List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(350.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Beings: {}", being_query.iter().count()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        window_visible.beings_list = false;
                    }
                });
            });
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(beings) = beings_by_dimension.get(dim_key) {
                ui.collapsing(format!("{} ({})", dim_key, beings.len()), |ui| {
                    for (entity, name) in beings.iter() {
                        let label = if let Some(n) = name {
                            format!("{} ({:?})", n, entity)
                        } else {
                            format!("Unnamed ({:?})", entity)
                        };
                        ui.label(label);
                    }
                });
                }
            }
        });
}
