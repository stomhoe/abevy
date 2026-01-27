use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

use crate::debug_resources::{DubugWindowsVisibility, DebugSelectedEntities};

// Import needed components
use tilemap::chunking_components::{Chunk, TilesToSave, TerrGenOpsLaunched, ReadyForTerrgen, ActivatingChunks};
use tilemap::regioning::regioning_components::{Region, GridOfSgcs, ClaimList, RegionPlannedTiles, ChunksActiveInRegion, CountsOfSgcs, PendingOfferTimeout, EmptyRegionDespawnTimer, AllTilesPrepared, BuildingStarted, AllClaimsProcessed};
use tilemap::terrain_gen::terrgen_operaton_list_components::{OperationList, Operation, Operand, OperandElement, VariablesArray};
use tilemap::terrain_gen::terrgen_components::{Terrgen, FnlNoiseComp};
use tilemap_shared::{ChunkPos, RegionPos};


use being::being_components::Being;
use dimension_shared::DimensionRef;
use camera::camera_components::CameraTarget;

#[allow(unused_parens)]
pub fn chunks_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    chunk_query: Query<(
        Entity,
        &Chunk,
        &DimensionRef,
        &ChunkPos,
        Option<&Name>,
        &Children,
        Option<&TilesToSave>,
        Has<TerrGenOpsLaunched>,
        Has<ReadyForTerrgen>,
        Option<&ActivatingChunks>,
    ), With<Chunk>>,
    dimension_query: Query<&Name>,
    camera_dimension: Query<(&DimensionRef, &Transform), With<CameraTarget>>,
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

    // Get camera target dimension and position
    let (camera_dim_ref, camera_chunk_pos) = camera_dimension.iter().next()
        .map(|(dim_ref, transform)| {
            let chunk_pos = ChunkPos::from(transform.translation);
            (Some(dim_ref), Some(chunk_pos))
        })
        .unwrap_or((None, None));
    
    // Group chunks by dimension and position
    let mut chunks_by_dimension: BTreeMap<String, HashMap<ChunkPos, (Entity, Option<&Name>, &Children, Option<&TilesToSave>, bool, bool, Option<&ActivatingChunks>)>> =
        BTreeMap::new();

    for (entity, _chunk, dim_ref, chunk_pos, name, children, tiles_to_save, has_terrgen_ops, has_ready_for_terrgen, activating_chunks) in chunk_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };
        
        chunks_by_dimension
            .entry(dim_name)
            .or_insert_with(HashMap::new)
            .insert(*chunk_pos, (entity, name, children, tiles_to_save, has_terrgen_ops, has_ready_for_terrgen, activating_chunks));
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
                    if ui.button("✖").clicked() {
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
                                            if let Some((entity, name, children, _tiles_to_save, _has_terrgen_ops, _has_ready_for_terrgen, _activating_chunks)) = chunks_map.get(&pos) {
                                                let is_selected = selected_entities.selected_chunks.contains(entity);
                                                let is_camera_pos = camera_chunk_pos.map_or(false, |cam_pos| cam_pos == pos);
                                                
                                                let mut label = format!("{},{}\n{} children", x, y, children.len());
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
            
            // Show details for selected chunks in stable order
            let mut selected_chunk_details: Vec<_> = chunks_by_dimension.iter()
                .flat_map(|(_, map)| map.iter())
                .filter(|(_, (entity, ..))| selected_entities.selected_chunks.contains(entity))
                .collect();
            selected_chunk_details.sort_by_key(|(_, (entity, ..))| entity.index());
            
            for (_chunk_pos, (entity, name, children, tiles_to_save, has_terrgen_ops, has_ready_for_terrgen, activating_chunks)) in selected_chunk_details {
                let name_str = name.map(|n| format!("{}", n)).unwrap_or_else(|| "unnamed".to_string());
                ui.collapsing(format!("Details: {} ({:?})", name_str, entity), |ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Children count: {}", children.len()));
                        
                        if let Some(tiles) = tiles_to_save {
                            ui.label(format!("TilesToSave: {} tiles", tiles.entities().len()));
                        }
                        
                        if *has_terrgen_ops {
                            ui.label("🔧 TerrGenOpsLaunched");
                        }
                        
                        if *has_ready_for_terrgen {
                            ui.label("✓ ReadyForTerrgen");
                        }
                        
                        if let Some(activating) = activating_chunks {
                            ui.label(format!("⏳ ActivatingChunks: {} entities, timer: {:.2}s", 
                                activating.entities.len(), 
                                activating.reactivation_timer.elapsed_secs()));
                        }
                    });
                });
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
    camera_dimension: Query<(&DimensionRef, &Transform), With<CameraTarget>>,
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
    
    // Get camera target dimension and position
    let (camera_dim_ref, camera_region_pos) = camera_dimension.iter().next()
        .map(|(dim_ref, transform)| {
            let chunk_pos = ChunkPos::from(transform.translation);
            let region_pos = chunk_pos.to_region_pos();
            (Some(dim_ref), Some(region_pos))
        })
        .unwrap_or((None, None));
    
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
                    if ui.button("✖").clicked() {
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
                    if ui.button("✖").clicked() {
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

#[allow(unused_parens)]
pub fn terrgen_editor_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut queries: ParamSet<(
        Query<(Entity, Option<&Name>, &OperationList)>,
        Query<&mut OperationList>,
    )>,
    noise_query: Query<(Entity, Option<&Name>, &FnlNoiseComp), With<Terrgen>>,
) {
    if !window_visible.terrgen_editor {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 700.0;
    let default_y = screen_rect.top() + 10.0;

    // Pre-collect noise data to avoid borrow conflicts
    let noise_data: Vec<(Entity, String)> = noise_query.iter()
        .map(|(ent, name, _)| {
            let label = if let Some(n) = name {
                format!("{}", n)
            } else {
                format!("{:?}", ent)
            };
            (ent, label)
        })
        .collect();

    // Pre-collect operationlist data
    let operationlist_vec: Vec<(Entity, String)> = queries.p0().iter()
        .map(|(ent, name, _)| {
            let label = if let Some(n) = name {
                format!("{} ({:?})", n, ent)
            } else {
                format!("OperationList ({:?})", ent)
            };
            (ent, label)
        })
        .collect();

    egui::Window::new("Terrgen Editor")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Terrgen Editor");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.terrgen_editor = false;
                    }
                });
            });
            ui.separator();

            // Select OperationList
            ui.label("Select OperationList:");
            
            for (entity, label) in operationlist_vec.iter() {
                if ui.selectable_label(
                    selected_entities.selected_operationlist == Some(*entity),
                    label,
                ).clicked() {
                    selected_entities.selected_operationlist = Some(*entity);
                }
            }

            ui.separator();

            // Edit selected OperationList
            if let Some(oplist_entity) = selected_entities.selected_operationlist {
                if let Ok(mut oplist) = queries.p1().get_mut(oplist_entity) {
                    ui.heading("Operations:");
                    
                    // Display trunk operations
                    let trunk_len = oplist.trunk.len();
                    let mut remove_op_idx = None;
                    
                    for idx in 0..trunk_len {
                        // Display operation header - extract values first
                        let op_str = oplist.trunk[idx].0.as_ref().to_string();
                        let var_idx = oplist.trunk[idx].2;
                        
                        ui.horizontal(|ui| {
                            ui.label(format!("Op {}: {} (Var{})", idx, op_str, var_idx));
                            
                            // Move up button
                            if idx > 0 && ui.button("⬆").clicked() {
                                oplist.trunk.swap(idx, idx - 1);
                            }
                            
                            // Move down button
                            if idx < trunk_len - 1 && ui.button("⬇").clicked() {
                                oplist.trunk.swap(idx, idx + 1);
                            }
                            
                            // Remove button
                            if ui.button("✕").clicked() {
                                remove_op_idx = Some(idx);
                            }
                        });
                        
                        // Edit operands for this operation
                        ui.horizontal(|ui| {
                            ui.label("  Operands:");
                            if ui.button("+ Add").clicked() {
                                oplist.trunk[idx].1.push(Operand {
                                    complement: false,
                                    element: OperandElement::default(),
                                });
                            }
                        });
                        
                        // Edit each operand
                        let op_count = oplist.trunk[idx].1.len();
                        let mut remove_opd_idx = None;
                        
                        for opd_idx in 0..op_count {
                            let mut removed = false;
                            ui.horizontal(|ui| {
                                if ui.button("✕").clicked() {
                                    removed = true;
                                }
                                
                                ui.checkbox(&mut oplist.trunk[idx].1[opd_idx].complement, "Complement");
                                ui.label(format!("Operand {}:", opd_idx));
                                
                                match &mut oplist.trunk[idx].1[opd_idx].element {
                                    OperandElement::Value(v) => {
                                        ui.label("Value:");
                                        ui.add(egui::DragValue::new(v).speed(0.1));
                                    }
                                    OperandElement::StackArray(idx_val) => {
                                        ui.label("StackArray Idx:");
                                        ui.add(egui::DragValue::new(idx_val).speed(1.0));
                                    }
                                    OperandElement::NoiseEntity(entity_ref, _range, _complementary, seed) => {
                                        ui.label("NoiseEntity:");
                                        
                                        // Show current and allow selection from pre-collected data
                                        for (noise_ent, noise_label) in noise_data.iter() {
                                            if ui.selectable_label(*entity_ref == *noise_ent, noise_label).clicked() {
                                                *entity_ref = *noise_ent;
                                            }
                                        }
                                        
                                        ui.add(egui::DragValue::new(seed).speed(1.0).prefix("Seed: "));
                                    }
                                    OperandElement::HashPos(hash) => {
                                        ui.label("HashPos:");
                                        ui.add(egui::DragValue::new(hash).speed(1.0));
                                    }
                                    OperandElement::PoissonDisk(_) => {
                                        ui.label("PoissonDisk (non-editable)");
                                    }
                                }
                            });
                            
                            if removed {
                                remove_opd_idx = Some(opd_idx);
                            }
                        }
                        
                        // Remove operand if marked
                        if let Some(opd_idx) = remove_opd_idx {
                            oplist.trunk[idx].1.remove(opd_idx);
                        }
                    }
                    
                    // Remove operation if marked
                    if let Some(op_idx) = remove_op_idx {
                        oplist.trunk.remove(op_idx);
                    }
                    
                    ui.separator();
                    
                    // Add new operation button
                    if ui.button("+ Add New Operation").clicked() {
                        oplist.trunk.push((Operation::Add, vec![], 0));
                    }
                    
                    ui.separator();
                    
                    // Display bifurcations
                    ui.heading(format!("Bifurcations: {}", oplist.bifurcations.len()));
                    for (bif_idx, bifur) in oplist.bifurcations.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if let Some(oplist_ent) = bifur.oplist {
                                ui.label(format!("Bifurcation {}: OpList({:?}), Tiles: {}", 
                                    bif_idx, oplist_ent, bifur.tiles.len()));
                            } else {
                                ui.label(format!("Bifurcation {}: No OpList, Tiles: {}", 
                                    bif_idx, bifur.tiles.len()));
                            }
                        });
                    }
                }
            }

            ui.separator();

            // Select and show Noise components
            ui.label("Available Noise Components:");
            
            for (entity, label) in noise_data.iter() {
                if ui.selectable_label(
                    selected_entities.selected_noise == Some(*entity),
                    label,
                ).clicked() {
                    selected_entities.selected_noise = Some(*entity);
                }
            }
        });
}
