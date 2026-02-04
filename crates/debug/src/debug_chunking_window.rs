use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use std::collections::{BTreeMap, HashMap};

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

use camera::camera_components::CameraTarget;
use common::common_components::*;
use dimension_shared::DimensionRef;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use ::sprite_shared::*;
use tilemap::chunking::chunking_components::{ActivatingChunks, Chunk, ReadyForTerrgen, TerrGenOpsLaunched, TilesToSave};
use tilemap::chunking::chunking_resources::{AaChunkRangeSettings, LoadedChunks};
use tilemap::regioning::regioning_components::*;
use tilemap::tile::tile_components::*;
use tilemap::tile::tile_shader::tile_shader_components::TileShaderRef;
use ::tilemap_shared::*;

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
pub fn debug_chunking_window(
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
    tile_storage_query: Query<(Entity, &TileStorage, Option<&AcZ>, Option<&TileShaderRef>), With<TileStorage>>,
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

                                                let label = format!("{},{} e{}\n{} children", x, y, entity.index(), children_count);

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
                                    if let Ok((tmap_entity, tile_storage, ac_z, shader_ref)) = tile_storage_query.get(child_entity) {

                                        // Build the label with AcZ and shader info
                                        let mut label = format!("🗺️ Tilemap ({})", tmap_entity.index());
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

                                            if ui.button("📋 View All Components").clicked() {
                                                selected_entities.selected_tilemap = Some(child_entity);
                                                window_visible.tilemap_details = true;
                                            }

                                            if let Some(clicked_tile) = render_tilemap_grid(ui, tile_storage, &tile_query, &ezero_query, &mut selected_entities.selected_tile) {
                                                selected_entities.selected_tile = Some(clicked_tile);
                                                window_visible.tile_details = true;  // Show tile details window when selected
                                            }
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

            // Display LoadedChunks resource sorted by distance to camera in a grid
            ui.separator();
            egui::CollapsingHeader::new("Loaded Chunks Resource (Grid)")
                .default_open(false)
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
                                                        egui::RichText::new(format!("{},{} [{}]", x, y, entity.index()))
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
        });
}
