use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy_inspector_egui::bevy_inspector;
use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use std::collections::{BTreeMap, HashMap};

use crate::debug_resources::{DubugWindowsVisibility, DebugSelectedEntities};

// Import needed components
use tilemap::chunking_components::*;
use tilemap::chunking_resources::{AaChunkRangeSettings, LoadedChunks};
use tilemap::regioning::regioning_components::*;
use tilemap::terrain_gen::terrgen_operaton_list_components::*;
use tilemap::terrain_gen::terrgen_components::{Terrgen, FnlNoiseComp};
use tilemap::terrain_gen::terrgen_resources::RegisteredPositions;
use tilemap::tile::tile_components::*;
use tilemap::tile::tile_shader::tile_shader_components::TileShaderRef;
use ::tilemap_shared::*;
use bevy_ecs_tilemap::prelude::{TileStorage, TilePos};
use game_common::game_common_components::{DespawnTimer, EntityZero, EntityZeroRef};
use ::sprite_shared::*;
use common::common_components::*;

use being::being_components::Being;
use dimension_shared::DimensionRef;
use camera::camera_components::CameraTarget;
use sprite::sprite_components::SpriteConfig;

// Color palette for unique tile types - readable and distinct colors
const TILE_COLORS: &[egui::Color32] = &[
    egui::Color32::from_rgb(100, 200, 100),  // Green
    egui::Color32::from_rgb(100, 150, 255),  // Light Blue
    egui::Color32::from_rgb(255, 200, 100),  // Orange
    egui::Color32::from_rgb(200, 100, 255),  // Purple
    egui::Color32::from_rgb(255, 150, 150),  // Light Red
    egui::Color32::from_rgb(100, 200, 200),  // Cyan
    egui::Color32::from_rgb(255, 255, 100),  // Yellow
    egui::Color32::from_rgb(200, 200, 100),  // Olive
    egui::Color32::from_rgb(150, 200, 150),  // Light Green
    egui::Color32::from_rgb(200, 150, 255),  // Light Purple
];

fn get_color_for_str_id(str_id: &str) -> egui::Color32 {
    let mut hash: usize = 0;
    for byte in str_id.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
    }
    TILE_COLORS[hash % TILE_COLORS.len()]
}

#[allow(unused_parens)]
fn render_tilemap_grid(
    ui: &mut egui::Ui,
    tile_storage: &TileStorage,
    tile_query: &Query<(Entity, &EntityZeroRef), With<Tile>>,
    ezero_query: &Query<&TileStrId, With<EntityZero>>,
    selected_tile: &mut Option<Entity>,
) -> Option<Entity> {
    let size = tile_storage.size;
    
    // Only render if not too large (avoid performance issues)
    if size.x > 50 || size.y > 50 {
        ui.label(format!("Grid too large to display: {}x{}", size.x, size.y));
        return None;
    }
    
    let mut clicked_tile = None;
    
    egui::Grid::new("tilemap_tiles_grid")
        .spacing([2.0, 2.0])
        .show(ui, |ui| {
            for y in (0..size.y).rev() {
                for x in 0..size.x {
                    let tile_pos = TilePos { x, y };
                    
                    if let Some(tile_entity) = tile_storage.checked_get(&tile_pos) {
                        // Try to get EntityZeroRef for this tile
                        if let Ok((_, ezero_ref)) = tile_query.get(tile_entity) {
                            // Query the EntityZero to get its StrId
                            if let Ok(str_id) = ezero_query.get(ezero_ref.0) {
                                let str_id_str = str_id.as_str();
                                let label = format!("{}", str_id_str);
                                let color = get_color_for_str_id(str_id_str);
                                let is_selected = selected_tile.map_or(false, |s| s == tile_entity);
                                let response = ui.selectable_label(is_selected, egui::RichText::new(label).small().color(color));
                                if response.clicked() {
                                    clicked_tile = Some(tile_entity);
                                }
                            } else {
                                let label = format!("Ez:{:?}", ezero_ref.0.index());
                                let color = egui::Color32::GRAY;
                                let is_selected = selected_tile.map_or(false, |s| s == tile_entity);
                                let response = ui.selectable_label(is_selected, egui::RichText::new(label).small().color(color));
                                if response.clicked() {
                                    clicked_tile = Some(tile_entity);
                                }
                            }
                        } else {
                            let is_selected = selected_tile.map_or(false, |s| s == tile_entity);
                            let response = ui.selectable_label(is_selected, egui::RichText::new(format!("Ent:{:?}", tile_entity.index())).small());
                            if response.clicked() {
                                clicked_tile = Some(tile_entity);
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new(".").small());
                    }
                }
                ui.end_row();
            }
        });
    
    clicked_tile
}

#[allow(unused_parens)]
pub fn chunks_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut chunk_settings: ResMut<AaChunkRangeSettings>,
    chunk_query: Query<(
        Entity,
        &Chunk,
        &DimensionRef,
        &ChunkPos,
        Option<&Children>,
        Option<&TilesToSave>,
        Has<TerrGenOpsLaunched>,
        Has<ReadyForTerrgen>,
        Option<&ActivatingChunks>,
    ), With<Chunk>>,

    camera_dimension: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    loaded_chunks: Res<LoadedChunks>,
    // Query for child entities to check their components
    tile_storage_query: Query<(Entity, Option<&Name>, &TileStorage, Option<&AcZ>, Option<&TileShaderRef>), With<TileStorage>>,
    tile_query: Query<(Entity, &EntityZeroRef), With<Tile>>,
    ezero_query: Query<&TileStrId, With<EntityZero>>,
    id_query: Query<&StrId>,
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
            let chunk_pos = ChunkPos::from(transform.translation());
            (Some(dim_ref), Some(chunk_pos))
        })
        .unwrap_or((None, None));
    
    // Group chunks by dimension and position
    let mut chunks_by_dimension: BTreeMap<String, HashMap<ChunkPos, (Entity, Option<&Children>, Option<&TilesToSave>, bool, bool, Option<&ActivatingChunks>)>> =
        BTreeMap::new();

    for (entity, _chunk, dim_ref, chunk_pos, children, tiles_to_save, has_terrgen_ops, has_ready_for_terrgen, activating_chunks) in chunk_query.iter() {
        let dim_name = if let Ok(str_id) = id_query.get(dim_ref.0) {
            str_id.as_str().to_string()
        } else {
            format!("{:?}", dim_ref)
        };
        
        chunks_by_dimension
            .entry(dim_name)
            .or_insert_with(HashMap::new)
            .insert(*chunk_pos, (entity, children, tiles_to_save, has_terrgen_ops, has_ready_for_terrgen, activating_chunks));
    }
    
    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = chunks_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        if let Ok(camera_str_id) = id_query.get(camera_ref.0) {
            let camera_dim_str = camera_str_id.as_str().to_string();
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

            // Chunk Range Settings
            ui.heading("Range Settings");
            ui.horizontal(|ui| {
                ui.label("Visibility Distance:");
                ui.add(egui::Slider::new(&mut chunk_settings.chunk_visib_max_dist, 100.0..=10000.0));
            });
            ui.horizontal(|ui| {
                ui.label("Active Distance:");
                ui.add(egui::Slider::new(&mut chunk_settings.chunk_active_max_dist, 100.0..=10000.0));
            });
            ui.horizontal(|ui| {
                ui.label("Discovery Range:");
                ui.add(egui::Slider::new(&mut chunk_settings.discovery_range, 1..=10));
            });
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(chunks_map) = chunks_by_dimension.get(dim_key) {
                    let is_camera_dim = camera_dim_ref.map_or(false, |camera_ref| {
                        if let Ok(camera_str_id) = id_query.get(camera_ref.0) {
                            dim_key == &camera_str_id.as_str().to_string()
                        } else {
                            false
                        }
                    });
                    egui::CollapsingHeader::new(dim_key)
                        .default_open(is_camera_dim)
                        .show(ui, |ui| {
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
                                    // Get currently selected chunk (single select)
                                    let selected_chunk = selected_entities.selected_chunks.iter().next().copied();
                                    
                                    for y in (min_y..=max_y).rev() {
                                        for x in min_x..=max_x {
                                            let pos = ChunkPos(IVec2::new(x, y));
                                            if let Some((entity, children, _tiles_to_save, _has_terrgen_ops, _has_ready_for_terrgen, _activating_chunks)) = chunks_map.get(&pos) {
                                                let is_selected = selected_chunk == Some(*entity);
                                                let is_camera_pos = camera_chunk_pos.map_or(false, |cam_pos| cam_pos == pos);
                                                
                                                let children_count = children.map_or(0, |c| c.len());
                                                
                                                // Check if any child has TileStorage (TilemapType)
                                                let has_tilemap_child = children
                                                    .map(|children| {
                                                        children.iter().any(|child| tile_storage_query.get(child).is_ok())
                                                    })
                                                    .unwrap_or(false);
                                                
                                                let label = format!("{},{}\n{} children", x, y, children_count);
                                                
                                                let mut rich_text = egui::RichText::new(&label).small();
                                                
                                                // Apply red color if no tilemap children (includes chunks with 0 children)
                                                if !has_tilemap_child {
                                                    rich_text = rich_text.color(egui::Color32::RED);
                                                }
                                                
                                                // Apply camera position styling (overrides other colors unless red)
                                                if is_camera_pos && has_tilemap_child {
                                                    rich_text = rich_text.color(egui::Color32::YELLOW).strong();
                                                }
                                                
                                                let button_response = ui.selectable_label(is_selected, rich_text);
                                                
                                                if button_response.clicked() {
                                                    // Single select: clear previous selection and select new chunk
                                                    selected_entities.selected_chunks.clear();
                                                    selected_entities.selected_chunks.insert(*entity);
                                                    window_visible.chunk_details = true;  // Show chunk details window
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
            
            // Display LoadedChunks resource sorted by distance to camera in a grid
            ui.separator();
            egui::CollapsingHeader::new("Loaded Chunks Resource (Grid)")
                .default_open(true)
                .show(ui, |ui| {
                // Get camera position and current chunk
                let camera_chunk_pos = camera_dimension.iter().next()
                    .map(|(_, transform)| ChunkPos::from(transform.translation()));
                
                // Group chunks by dimension
                let mut chunks_by_dim: BTreeMap<String, Vec<(Entity, ChunkPos)>> = BTreeMap::new();
                
                for ((dim_ref, chunk_pos), entity) in loaded_chunks.0.iter() {
                    let dim_str_id = if let Ok(str_id) = id_query.get(dim_ref.0) {
                        str_id.as_str().to_string()
                    } else {
                        format!("{:?}", dim_ref)
                    };
                    
                    chunks_by_dim
                        .entry(dim_str_id)
                        .or_insert_with(Vec::new)
                        .push((*entity, *chunk_pos));
                }
                
                ui.label(format!("Total entries: {}", loaded_chunks.0.len()));
                
                // Display each dimension's chunks in a grid
                for (dim_name, chunks) in chunks_by_dim.into_iter() {
                    let header_label = format!("{} - {} chunks", dim_name, chunks.len());
                    egui::CollapsingHeader::new(&header_label)
                        .default_open(true)
                        .show(ui, |ui| {
                            // Find the bounding box of chunks
                            let (min_x, max_x, min_y, max_y) = chunks.iter().fold(
                                (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                                |(min_x, max_x, min_y, max_y), (_, pos)| {
                                    (
                                        min_x.min(pos.0.x),
                                        max_x.max(pos.0.x),
                                        min_y.min(pos.0.y),
                                        max_y.max(pos.0.y),
                                    )
                                }
                            );
                            
                            let grid_width = (max_x - min_x + 1) as usize;
                            let grid_height = (max_y - min_y + 1) as usize;
                            
                            // Only render if grid is not too large
                            if grid_width > 100 || grid_height > 100 {
                                ui.label(format!("Grid too large to display: {}x{}", grid_width, grid_height));
                            } else {
                                // Create a map for quick lookup
                                let chunk_map: std::collections::HashMap<(i32, i32), (Entity, ChunkPos)> = 
                                    chunks.iter().map(|(entity, pos)| ((pos.0.x, pos.0.y), (*entity, *pos))).collect();
                                
                                // Get currently selected chunk (should be 0 or 1)
                                let selected_chunk = selected_entities.selected_chunks.iter().next().copied();
                                
                                // Render grid
                                egui::Grid::new(format!("loaded_chunks_grid_{}", dim_name))
                                    .spacing([2.0, 2.0])
                                    .show(ui, |ui| {
                                        for y in (min_y..=max_y).rev() {
                                            for x in min_x..=max_x {
                                                if let Some((entity, chunk_pos)) = chunk_map.get(&(x, y)) {
                                                    let is_camera_chunk = camera_chunk_pos
                                                        .map(|cam_pos| cam_pos == *chunk_pos)
                                                        .unwrap_or(false);
                                                    let is_selected = selected_chunk == Some(*entity);
                                                    
                                                    let bg_color = if is_camera_chunk {
                                                        egui::Color32::from_rgb(100, 200, 100)
                                                    } else {
                                                        egui::Color32::DARK_GRAY
                                                    };
                                                    
                                                    let text_color = if is_selected {
                                                        egui::Color32::YELLOW
                                                    } else {
                                                        egui::Color32::WHITE
                                                    };
                                                    
                                                    if ui.selectable_label(
                                                        is_selected,
                                                        egui::RichText::new(format!("{},{}", x, y))
                                                            .background_color(bg_color)
                                                            .color(text_color)
                                                    ).clicked() {
                                                        // Single select: clear previous selection and select new chunk
                                                        selected_entities.selected_chunks.clear();
                                                        selected_entities.selected_chunks.insert(*entity);
                                                        window_visible.chunk_details = true;
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
            });

            
            // Show details for selected chunks in stable order
            let mut selected_chunk_details: Vec<_> = chunks_by_dimension.iter()
                .flat_map(|(_, map)| map.iter())
                .filter(|(_, (entity, ..))| selected_entities.selected_chunks.contains(entity))
                .collect();
            selected_chunk_details.sort_by_key(|(_, (entity, ..))| entity.index());
            
            for (chunk_pos, (entity, children, tiles_to_save, has_terrgen_ops, has_ready_for_terrgen, activating_chunks)) in selected_chunk_details {
                egui::CollapsingHeader::new(format!("Details: {:?} ({:?})", chunk_pos, entity))
                    .default_open(true)
                    .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let children_count = children.map_or(0, |c| c.len());
                        ui.label(format!("Children count: {}", children_count));
                        
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
                        
                        // Display children details
                        ui.separator();
                        if let Some(children_ref) = children {
                            egui::CollapsingHeader::new("Children:")
                                .default_open(true)
                                .show(ui, |ui| {
                                for child_entity in children_ref.iter() {
                                    // Check if child is a tilemap with TileStorage
                                    if let Ok((tmap_entity, tmap_name, tile_storage, ac_z, shader_ref)) = tile_storage_query.get(child_entity) {
                                        let tmap_name_str = tmap_name.map(|n| format!("{}", n)).unwrap_or_else(|| "unnamed".to_string());
                                        
                                        // Build the label with AcZ and shader info
                                        let mut label = format!("📦 Tilemap: {}", tmap_name_str);
                                        if let Some(z) = ac_z {
                                            label.push_str(&format!(" [Z: {:.1}]", z.0));
                                        }
                                        if let Some(shader_ref) = shader_ref {
                                            if let Ok(shader_str) = id_query.get(shader_ref.0) {
                                                label.push_str(&format!(" [Shader: {}]", shader_str.as_str()));
                                            }
                                        }
                                        
                                        ui.collapsing(label, |ui| {
                                            ui.label(format!("Size: {}x{}", tile_storage.size.x, tile_storage.size.y));
                                            
                                            // Draw tilemap grid
                                            ui.label("Tile Grid:");
                                            ui.indent("tilemap_grid", |ui| {
                                                if let Some(clicked_tile) = render_tilemap_grid(ui, tile_storage, &tile_query, &ezero_query, &mut selected_entities.selected_tile) {
                                                    selected_entities.selected_tile = Some(clicked_tile);
                                                    window_visible.tile_details = true;  // Show tile details window when selected
                                                }
                                            });
                                        });
                                    } else {
                                        // Display generic child info
                                        ui.label(format!("Child: {:?}", child_entity));
                                    }
                                }
                            });
                        } else {
                            ui.label("No children");
                        }
                    });
                });
            }
        });
}

#[allow(unused_parens)]
pub fn tile_details_inspector(world: &mut World) {
    let selected_tile_entity = world.resource::<DebugSelectedEntities>().selected_tile;
    
    if selected_tile_entity.is_none() {
        return;
    }
    
    let selected_tile_entity = selected_tile_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.tile_details {
        return;
    }
    
    // Try to get the TileStrId from the referenced EntityZero
    let tile_str_id = if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        if let Some(ezero_ref) = entity_ref.get::<game_common::game_common_components::EntityZeroRef>() {
            if let Ok(ezero_entity) = world.get_entity(ezero_ref.0) {
                if let Some(str_id) = ezero_entity.get::<TileStrId>() {
                    Some(format!("{}", str_id))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Tile Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            // Display TileStrId if available, otherwise show Entity ID
            if let Some(str_id) = tile_str_id {
                ui.heading(format!("Tile: {}", str_id));
            } else {
                ui.heading(format!("Tile Entity: {:?}", selected_tile_entity));
            }
            ui.separator();
            
            ui.label("All Components on this Tile:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_tile_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_tile = None;
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.tile_details = false;
        }
    }
}

#[allow(unused_parens)]
pub fn chunk_details_inspector(world: &mut World) {
    let selected_chunk_entity = world.resource::<DebugSelectedEntities>().selected_chunks.iter().next().copied();
    
    if selected_chunk_entity.is_none() {
        return;
    }
    
    let selected_chunk_entity = selected_chunk_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.chunk_details {
        return;
    }
    
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Chunk Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 320.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            if let Ok(entity_ref) = world.get_entity(selected_chunk_entity) {
                if let Some(chunk_pos) = entity_ref.get::<ChunkPos>() {
                    ui.heading((*chunk_pos).to_string());
                }
            }
            ui.separator();
            
            ui.label("All Components on this Chunk:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_chunk_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_chunks.clear();
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.chunk_details = false;
        }
    }
}

#[allow(unused_parens)]
pub fn portals_details_inspector(world: &mut World) {
    let selected_portal_entity = world.resource::<DebugSelectedEntities>().selected_portals.iter().next().copied();
    
    if selected_portal_entity.is_none() {
        return;
    }
    
    let selected_portal_entity = selected_portal_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.portal_details {
        return;
    }
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Portal Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            if let Ok(entity_ref) = world.get_entity(selected_portal_entity) {
                if let Some(global_pos) = entity_ref.get::<GlobalTilePos>() {
                    ui.heading(format!("Portal at ({}, {})", (*global_pos).x(), (*global_pos).y()));
                }
            }
            ui.separator();
            
            ui.label("All Components on this Portal:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_portal_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_portals.clear();
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.portal_details = false;
        }
    }
}

#[allow(unused_parens)]
pub fn tilemap_details_inspector(world: &mut World) {
    // This will be set from chunks_list_window when a tilemap is selected
    let selected_tilemap_entity = if let Some(selected_entities) = world.get_resource::<DebugSelectedEntities>() {
        selected_entities.selected_operationlist  // Reusing for now as tilemap selection - could add dedicated field
    } else {
        None
    };
    
    if selected_tilemap_entity.is_none() {
        return;
    }
    
    let selected_tilemap_entity = selected_tilemap_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.tilemap_details {
        return;
    }
    
    let _ = window_visible;  // Release the resource borrow
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Tilemap Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 630.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.heading(format!("Tilemap Entity: {:?}", selected_tilemap_entity));
            ui.separator();
            
            ui.label("All Components on this Tilemap:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_tilemap_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_operationlist = None;
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.tilemap_details = false;
        }
    }
}

#[allow(unused_parens)]
pub fn being_details_inspector(world: &mut World) {
    let selected_being_entity = if let Some(selected_entities) = world.get_resource::<DebugSelectedEntities>() {
        selected_entities.selected_being
    } else {
        None
    };
    
    if selected_being_entity.is_none() {
        return;
    }
    
    let selected_being_entity = selected_being_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.being_details {
        return;
    }
    
    let _ = window_visible;  // Release the resource borrow
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Being Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.heading(format!("Being Entity: {:?}", selected_being_entity));
            ui.separator();
            
            ui.label("All Components on this Being:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_being_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_being = None;
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.being_details = false;
        }
    }
}

#[allow(unused_parens)]
pub fn exempted_entity_details_inspector(world: &mut World) {
    let selected_entity = world.resource::<DebugSelectedEntities>().selected_exempted_entity;
    
    if selected_entity.is_none() {
        return;
    }
    
    let selected_entity = selected_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.exempted_entity_details {
        return;
    }
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Exempted Entity Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.heading(format!("Exempted Entity: {:?}", selected_entity));
            ui.separator();
            
            ui.label("All Components on this Entity:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_exempted_entity = None;
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.exempted_entity_details = false;
        }
    }
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
}

#[allow(unused_parens)]
pub fn beings_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
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
                let is_camera_dim = camera_dim_ref.map_or(false, |camera_ref| {
                    if let Ok(camera_name) = dimension_query.get(camera_ref.0) {
                        dim_key == &format!("{}", camera_name)
                    } else {
                        false
                    }
                });
                egui::CollapsingHeader::new(format!("{} ({})", dim_key, beings.len()))
                    .default_open(is_camera_dim)
                    .show(ui, |ui| {
                    for (entity, name) in beings.iter() {
                        let label = if let Some(n) = name {
                            format!("{} ({:?})", n, entity)
                        } else {
                            format!("Unnamed ({:?})", entity)
                        };
                        let is_selected = selected_entities.selected_being == Some(*entity);
                        if ui.selectable_label(is_selected, label).clicked() {
                            selected_entities.selected_being = Some(*entity);
                            window_visible.being_details = true;
                        }
                    }
                });
                }
            }
        });
}

#[allow(unused_parens)]
pub fn portals_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    portal_query: Query<(Entity, &DimensionRef, &GlobalTilePos, Option<&EntityZeroRef>, &PortalTo), With<PortalTo>>,
    dimension_query: Query<&Name>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    ezero_query: Query<&TileStrId, With<EntityZero>>,
    target_query: Query<Entity>,
) {
    if !window_visible.portals_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 400.0;
    let default_y = screen_rect.top() + 10.0;

    // Get camera position and dimension
    let camera_info = camera_query.iter().next();
    let camera_pos = camera_info.map(|(_, transform)| transform.translation().xy());
    let camera_dim_ref = camera_info.map(|(dim_ref, _)| dim_ref);

    // Group portals by dimension
    let mut portals_by_dimension: BTreeMap<String, Vec<(Entity, GlobalTilePos, Option<EntityZeroRef>, Vec2, bool, f32)>> = BTreeMap::new();

    for (entity, dim_ref, global_pos, ezero_ref, portal_to) in portal_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };
        
        // Calculate direction vector if in same dimension
        let direction = if camera_dim_ref.map(|c| c == dim_ref).unwrap_or(false) {
            if let Some(cam_pos) = camera_pos {
                let portal_pixel_pos: Vec2 = (*global_pos).into();
                portal_pixel_pos - cam_pos
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };
        
        let distance = direction.length();
        
        // Check if the target entity exists
        let target_exists = target_query.get(portal_to.dest_portal).is_ok();
        
        portals_by_dimension
            .entry(dim_name)
            .or_insert_with(Vec::new)
            .push((entity, *global_pos, ezero_ref.copied(), direction, target_exists, distance));
    }
    
    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = portals_by_dimension.keys().cloned().collect();
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
    
    // Helper function to get directional arrow
    let get_arrow = |dir: Vec2| -> &'static str {
        if dir == Vec2::ZERO {
            "?"
        } else {
            let angle = dir.y.atan2(dir.x);
            let normalized = ((angle * 4.0 / std::f32::consts::PI + 8.5) as i32 % 8) as usize;
            match normalized {
                0 => "→",
                1 => "↗",
                2 => "↑",
                3 => "↖",
                4 => "←",
                5 => "↙",
                6 => "↓",
                7 => "↘",
                _ => "?",
            }
        }
    };

    egui::Window::new("Portals List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Portals: {}", portal_query.iter().count()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.portals_list = false;
                    }
                });
            });
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(mut portals) = portals_by_dimension.remove(dim_key) {
                    // Sort portals by distance (closest first)
                    portals.sort_by(|a, b| {
                        a.5.partial_cmp(&b.5).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    
                    egui::CollapsingHeader::new(format!("{} ({})", dim_key, portals.len()))
                        .default_open(true)
                        .show(ui, |ui| {
                            for (entity, _global_pos, ezero_ref, direction, target_exists, distance) in portals.iter() {
                                // Check if this portal is selected
                                let is_selected = selected_entities.selected_portals.contains(entity);
                                
                                // Get the StrId from EntityZero
                                let str_id_str = if let Some(ezero_ref) = ezero_ref {
                                    if let Ok(str_id) = ezero_query.get(ezero_ref.0) {
                                        format!("{}", str_id)
                                    } else {
                                        "Unknown".to_string()
                                    }
                                } else {
                                    "NoType".to_string()
                                };
                                
                                let arrow = get_arrow(*direction);
                                let portal_label = format!("{} {} {:?} [{}]", arrow, str_id_str, entity, distance.round() as i32);
                                
                                let text = egui::RichText::new(&portal_label);
                                let text = if !target_exists {
                                    text.color(egui::Color32::RED)
                                } else if is_selected {
                                    text.color(egui::Color32::YELLOW)
                                } else {
                                    text.color(egui::Color32::WHITE)
                                };
                                
                                if ui.selectable_label(is_selected, text).clicked() {
                                    // Single select: clear previous selection and select new portal
                                    selected_entities.selected_portals.clear();
                                    selected_entities.selected_portals.insert(*entity);
                                    window_visible.portal_details = true;
                                }
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
        Query<&mut FnlNoiseComp, With<Terrgen>>,
        Query<(Entity, Option<&Name>, &FnlNoiseComp), With<Terrgen>>,
    )>,
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
    let noise_data: Vec<(Entity, String)> = queries.p3().iter()
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
        .default_width(1200.0)
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

            // Select OperationList dropdown and Noise Component selector on same row
            let oplist_label = selected_entities.selected_operationlist
                .and_then(|ent| operationlist_vec.iter().find(|(e, _)| *e == ent).map(|(_, l)| l.clone()))
                .unwrap_or_else(|| "Select OperationList".to_string());
            
            let noise_label = selected_entities.selected_noise
                .and_then(|ent| noise_data.iter().find(|(e, _)| *e == ent).map(|(_, l)| l.clone()))
                .unwrap_or_else(|| "None".to_string());
            
            ui.horizontal(|ui| {
                ui.label("OperationList:");
                egui::ComboBox::from_id_salt(0u64)
                    .selected_text(&oplist_label)
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for (entity, label) in operationlist_vec.iter() {
                            ui.selectable_value(
                                &mut selected_entities.selected_operationlist,
                                Some(*entity),
                                label,
                            );
                        }
                    });
                
                ui.separator();
                
                ui.label("Noise:");
                egui::ComboBox::from_id_salt(999u64)
                    .selected_text(&noise_label)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (entity, label) in noise_data.iter() {
                            ui.selectable_value(
                                &mut selected_entities.selected_noise,
                                Some(*entity),
                                label,
                            );
                        }
                    });
            });

            ui.separator();

            // Side-by-side layout for editors
            ui.columns(2, |columns| {
                // LEFT COLUMN: OperationList Editor
                columns[0].heading("Operations:");
                
                if let Some(oplist_entity) = selected_entities.selected_operationlist {
                    if let Ok(mut oplist) = queries.p1().get_mut(oplist_entity) {
                        let trunk_len = oplist.trunk.len();
                        let mut remove_op_idx = None;
                        
                        for idx in 0..trunk_len {
                            let op_str = oplist.trunk[idx].0.as_ref().to_string();
                            let var_idx = oplist.trunk[idx].2;
                            
                            columns[0].horizontal(|ui| {
                                ui.label(format!("Op {}: {} (Var{})", idx, op_str, var_idx));
                                
                                if idx > 0 && ui.button("⬆").clicked() {
                                    oplist.trunk.swap(idx, idx - 1);
                                }
                                
                                if idx < trunk_len - 1 && ui.button("⬇").clicked() {
                                    oplist.trunk.swap(idx, idx + 1);
                                }
                                
                                if ui.button("✕").clicked() {
                                    remove_op_idx = Some(idx);
                                }
                            });
                            
                            columns[0].horizontal(|ui| {
                                ui.label("  Operands:");
                                if ui.button("+ Add").clicked() {
                                    oplist.trunk[idx].1.push(Operand {
                                        complement: false,
                                        element: OperandElement::default(),
                                    });
                                }
                            });
                            
                            let op_count = oplist.trunk[idx].1.len();
                            let mut remove_opd_idx = None;
                            
                            for opd_idx in 0..op_count {
                                let mut removed = false;
                                columns[0].horizontal(|ui| {
                                    if ui.button("✕").clicked() {
                                        removed = true;
                                    }
                                    
                                    ui.checkbox(&mut oplist.trunk[idx].1[opd_idx].complement, "Complement");
                                    ui.label(format!("Opd {}:", opd_idx));
                                    
                                    let current_type = match &oplist.trunk[idx].1[opd_idx].element {
                                        OperandElement::Value(_) => "Value",
                                        OperandElement::StackArray(_) => "StackArray",
                                        OperandElement::NoiseEntity(_, _, _, _) => "NoiseEntity",
                                        OperandElement::HashPos(_) => "HashPos",
                                        OperandElement::PoissonDisk(_) => "PoissonDisk",
                                    };
                                    
                                    let combo_id = (idx as u64) * 1000 + (opd_idx as u64);
                                    egui::ComboBox::from_id_salt(combo_id)
                                        .selected_text(current_type)
                                        .width(100.0)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::Value(0.0), "Value");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::StackArray(0), "StackArray");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::NoiseEntity(Entity::PLACEHOLDER, Default::default(), false, 0), "NoiseEntity");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::HashPos(0), "HashPos");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::PoissonDisk(PoissonDisk::new(1, 0).unwrap_or_default()), "PoissonDisk");
                                        });
                                });
                                
                                columns[0].horizontal(|ui| {
                                    if ui.button("✕").clicked() {
                                        removed = true;
                                    }
                                    
                                    match &mut oplist.trunk[idx].1[opd_idx].element {
                                        OperandElement::Value(v) => {
                                            ui.label("Val:");
                                            ui.add(egui::DragValue::new(v).speed(0.1));
                                        }
                                        OperandElement::StackArray(idx_val) => {
                                            ui.label("Idx:");
                                            ui.add(egui::DragValue::new(idx_val).speed(1.0));
                                        }
                                        OperandElement::NoiseEntity(entity_ref, _range, _complementary, seed) => {
                                            let noise_label = noise_data
                                                .iter()
                                                .find(|(ent, _)| *ent == *entity_ref)
                                                .map(|(_, label)| label.clone())
                                                .unwrap_or_else(|| "None".to_string());
                                            
                                            let noise_combo_id = (idx as u64) * 10000 + (opd_idx as u64) * 100 + 50;
                                            egui::ComboBox::from_id_salt(noise_combo_id)
                                                .selected_text(&noise_label)
                                                .width(120.0)
                                                .show_ui(ui, |ui| {
                                                    for (noise_ent, noise_label) in noise_data.iter() {
                                                        ui.selectable_value(entity_ref, *noise_ent, noise_label);
                                                    }
                                                });
                                            
                                            ui.add(egui::DragValue::new(seed).speed(1.0).prefix("S:"));
                                        }
                                        OperandElement::HashPos(hash) => {
                                            ui.label("Hash:");
                                            ui.add(egui::DragValue::new(hash).speed(1.0));
                                        }
                                        OperandElement::PoissonDisk(_) => {
                                            ui.label("PoissonDisk");
                                        }
                                    }
                                });
                                
                                if removed {
                                    remove_opd_idx = Some(opd_idx);
                                }
                            }
                            
                            if let Some(opd_idx) = remove_opd_idx {
                                oplist.trunk[idx].1.remove(opd_idx);
                            }
                        }
                        
                        if let Some(op_idx) = remove_op_idx {
                            oplist.trunk.remove(op_idx);
                        }
                        
                        columns[0].separator();
                        
                        if columns[0].button("+ Add Operation").clicked() {
                            oplist.trunk.push((Operation::Add, vec![], 0));
                        }
                        
                        columns[0].separator();
                        columns[0].heading(format!("Bifurcations: {}", oplist.bifurcations.len()));
                        for (bif_idx, bifur) in oplist.bifurcations.iter().enumerate() {
                            if let Some(oplist_ent) = bifur.oplist {
                                columns[0].label(format!("Bif {}: OpList({:?}), Tiles: {}", 
                                    bif_idx, oplist_ent, bifur.tiles.len()));
                            } else {
                                columns[0].label(format!("Bif {}: No OpList, Tiles: {}", 
                                    bif_idx, bifur.tiles.len()));
                            }
                        }
                    }
                }
                
                // RIGHT COLUMN: Noise Component Editor
                columns[1].heading("Noise Component:");
                
                columns[1].separator();
                
                if let Some(noise_entity) = selected_entities.selected_noise {
                    if let Ok(mut noise_comp) = queries.p2().get_mut(noise_entity) {
                        columns[1].horizontal(|ui| {
                            ui.label("Seed:");
                            ui.add(egui::DragValue::new(&mut noise_comp.0.seed).speed(1));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Offset X:");
                            ui.add(egui::DragValue::new(&mut noise_comp.0.offset.x).speed(1));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Offset Y:");
                            ui.add(egui::DragValue::new(&mut noise_comp.0.offset.y).speed(1));
                        });

                        columns[1].separator();
                        columns[1].heading("Noise Type");
                        let current_type = format!("{:?}", noise_comp.0.noise_type);
                        columns[1].label(format!("Current: {}", current_type));

                        columns[1].separator();
                        columns[1].heading("Fractal Settings");
                        let current_fractal = format!("{:?}", noise_comp.0.fractal_type);
                        columns[1].label(format!("Type: {}", current_fractal));

                        columns[1].horizontal(|ui| {
                            ui.label("Octaves:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.octaves, 1..=10));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Lacunarity:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.lacunarity, 0.1..=4.0).step_by(0.01));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Gain:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.gain, 0.0..=1.0).step_by(0.01));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Weighted Strength:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.weighted_strength, 0.0..=1.0).step_by(0.01));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Ping Pong:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.ping_pong_strength, 0.0..=4.0).step_by(0.01));
                        });

                        columns[1].separator();
                        columns[1].heading("Cellular Settings");
                        let current_cellular_dist = format!("{:?}", noise_comp.0.cellular_distance_function);
                        columns[1].label(format!("Dist: {}", current_cellular_dist));

                        let current_cellular_return = format!("{:?}", noise_comp.0.cellular_return_type);
                        columns[1].label(format!("Return: {}", current_cellular_return));

                        columns[1].horizontal(|ui| {
                            ui.label("Jitter:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.cellular_jitter_modifier, 0.0..=2.0).step_by(0.01));
                        });

                        columns[1].separator();
                        columns[1].heading("Domain Warp");
                        let current_warp_type = format!("{:?}", noise_comp.0.domain_warp_type);
                        columns[1].label(format!("Type: {}", current_warp_type));

                        columns[1].horizontal(|ui| {
                            ui.label("Amplitude:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.domain_warp_amp, 0.0..=2.0).step_by(0.01));
                        });
                    }
                }
            })
        });
}

#[allow(unused_parens)]
pub fn registered_positions_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    registered_positions: Res<RegisteredPositions>,
    id_query: Query<&StrId>,
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

    egui::Window::new("Registered Positions")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(600.0)
        .default_height(300.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Registered Positions");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.registered_positions = false;
                    }
                });
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("RegisteredPositions:");
                ui.label(format!("Exempted entities: {}", registered_positions.get_exempted_entities().len()));
                ui.label(format!("Registered entries: {}", registered_positions.get_registered_entries().len()));
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Show exempted entities
                    if !registered_positions.get_exempted_entities().is_empty() {
                        ui.label("Exempted Entities:");
                        for entity in registered_positions.get_exempted_entities().iter() {
                            let is_selected = selected_entities.selected_exempted_entity == Some(*entity);
                            let label = format!("  {:?}", entity);
                            if ui.selectable_label(is_selected, label).clicked() {
                                selected_entities.selected_exempted_entity = Some(*entity);
                                window_visible.exempted_entity_details = true;
                            }
                        }
                        ui.separator();
                    }
                    
                    // Show registered entries
                    if !registered_positions.get_registered_entries().is_empty() {
                        ui.label("Registered Positions:");
                        for (entity, positions) in registered_positions.get_registered_entries().iter() {
                            ui.label(format!("Entity {:?}: {} positions", entity, positions.len()));
                            for (dim_ref, pos) in positions {
                                ui.label(format!("  Dim: {:?}, Pos: {:?}", dim_ref, pos));
                            }
                        }
                    }
                });
        });
}

#[allow(unused_parens)]
pub fn sprites_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    sprite_query: Query<(Entity, Option<&Name>), With<SpriteConfig>>,
) {
    if !window_visible.sprites_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 300.0;
    let default_y = screen_rect.bottom() - 400.0;

    // Collect sprites
    let sprites: Vec<(Entity, Option<Name>)> = sprite_query
        .iter()
        .map(|(entity, name)| (entity, name.map(|n| n.clone())))
        .collect();

    egui::Window::new("Sprites List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Sprites: {}", sprites.len()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.sprites_list = false;
                    }
                });
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (entity, name) in sprites.iter() {
                        let label = if let Some(n) = name {
                            format!("{} ({:?})", n, entity)
                        } else {
                            format!("Sprite ({:?})", entity)
                        };
                        let is_selected = selected_entities.selected_sprite == Some(*entity);
                        if ui.selectable_label(is_selected, label).clicked() {
                            selected_entities.selected_sprite = Some(*entity);
                            window_visible.sprite_details = true;
                        }
                    }
                });
        });
}

#[allow(unused_parens)]
pub fn sprite_details_inspector(world: &mut World) {
    let selected_sprite_entity = world.resource::<DebugSelectedEntities>().selected_sprite;
    
    if selected_sprite_entity.is_none() {
        return;
    }
    
    let selected_sprite_entity = selected_sprite_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();
    
    if !window_visible.sprite_details {
        return;
    }
    
    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();
    
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();
    
    let world_ptr = world as *mut World;
    let mut is_open = true;
    
    egui::Window::new("Selected Sprite Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.bottom() - 400.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            if let Ok(entity_ref) = world.get_entity(selected_sprite_entity) {
                if let Some(name) = entity_ref.get::<Name>() {
                    ui.heading(format!("Sprite: {}", name));
                } else {
                    ui.heading(format!("Sprite Entity: {:?}", selected_sprite_entity));
                }
            }
            ui.separator();
            
            ui.label("All Components on this Sprite:");
            ui.separator();
            
            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_sprite_entity, ui);
            }
            
            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_sprite = None;
                }
            }
        });
    
    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.sprite_details = false;
        }
    }
}

