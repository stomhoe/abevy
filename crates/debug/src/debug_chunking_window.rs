use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use std::collections::{BTreeMap, HashMap};

use crate::debug_resources::{DebugChunkingUiState, DebugSelectedEntities, DubugWindowsVisibility};

use camera::camera_components::CameraTarget;
use common::common_components::*;
use game_common::game_common_components::*;
use ::sprite_shared::*;
use tilemap::chunking::chunking_components::*;
use tilemap_shared::*;
use tilemap::tile::tile_components::*;
use tilemap::tile::tile_shader::tile_shader_components::TileShaderRef;
use tilemap::tile::tile_shader::tile_shader_resources::TileShaderEntityMap;

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

fn short_tile_label(str_id: &str) -> String {
    let mut chars = str_id.chars();
    let Some(first) = chars.next() else {
        return "..".to_string();
    };
    let last = str_id.chars().last().unwrap_or(first);
    format!("{}{}", first, last)
}

fn dimension_name_for_ref(
    dim_ref: &DimensionRef,
    dimension_map: &DimensionEntityMap,
    id_query: &Query<&StrId>,
) -> String {
    let Some(dim_ent) = dimension_map.0.get_cloned(dim_ref.0).ok() else {
        return format!("{:?}", dim_ref);
    };
    id_query
        .get(dim_ent)
        .map(|s| s.as_str().to_string())
        .unwrap_or_else(|_| format!("{:?}", dim_ref))
}

#[allow(unused_parens)]
fn render_tilemap_grid(
    ui: &mut egui::Ui,
    tile_storage: &TileStorage,
    tile_query: &Query<(Entity, &TemplEntiRef, Option<&InitialPos>), With<Tile>>,
    templ_query: &Query<&TileStrId, With<Templ>>,
    selected_tile: &mut Option<Entity>,
    camera_tile_pos: Option<GlobalTilePos>,
) -> Option<Entity> {
    let size = tile_storage.size;

    // Only render if not too large (avoid performance issues)
    if size.x > 50 || size.y > 50 {
        ui.label(format!("Grid too large to display: {}x{}", size.x, size.y));
        return None;
    }

    let cell_w = 22.0f32;
    let cell_h = 18.0f32;
    let grid_size = egui::vec2(size.x as f32 * cell_w, size.y as f32 * cell_h);
    let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let mut clicked_tile = None;
    for y in (0..size.y).rev() {
        for x in 0..size.x {
            let row = (size.y - 1 - y) as f32;
            let col = x as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + col * cell_w, rect.top() + row * cell_h),
                egui::vec2(cell_w, cell_h),
            );
            let tile_pos = TilePos { x, y };
            let id = ui.make_persistent_id(("tilemap_grid_cell", x, y));
            let response = ui.interact(cell_rect, id, egui::Sense::click());

            if let Some(tile_entity) = tile_storage.checked_get(&tile_pos) {
                let is_selected = selected_tile.map_or(false, |s| s == tile_entity);
                let mut is_camera_tile = false;
                if let Ok((_, templ_ref, initial_pos)) = tile_query.get(tile_entity) {
                    is_camera_tile = camera_tile_pos
                        .zip(initial_pos.map(|p| p.pos))
                        .map_or(false, |(cam_pos, tile_pos)| cam_pos == tile_pos);
                    if let Ok(str_id) = templ_query.get(templ_ref.0) {
                        let str_id_str = str_id.as_str();
                        let label = short_tile_label(str_id_str);
                        let fill = get_color_for_str_id(str_id_str).gamma_multiply(0.25);
                        painter.rect_filled(cell_rect, 0.0, fill);
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(10.0),
                            get_color_for_str_id(str_id_str),
                        );
                    } else {
                        painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgb(30, 30, 30));
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "??",
                            egui::FontId::proportional(10.0),
                            egui::Color32::GRAY,
                        );
                    }
                } else {
                    painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgb(28, 28, 28));
                    painter.text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "..",
                        egui::FontId::proportional(10.0),
                        egui::Color32::LIGHT_GRAY,
                    );
                }

                if is_camera_tile {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        egui::StrokeKind::Outside,
                    );
                } else if is_selected {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(1.5, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                } else {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_gray(50)),
                        egui::StrokeKind::Inside,
                    );
                }

                if response.clicked() {
                    clicked_tile = Some(tile_entity);
                }
            } else {
                painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgb(16, 16, 16));
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_gray(35)),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }

    clicked_tile
}

#[allow(unused_parens)]
fn render_spritetiles_grid(
    ui: &mut egui::Ui,
    chunk_pos: ChunkPos,
    child_entities: &[Entity],
    tile_storage_query: &Query<(Entity, &TileStorage, Option<&AcZ>, Option<&TileShaderRef>), With<TileStorage>>,
    spritetile_gpos_query: &Query<(Entity, &GlobalTilePos, Option<&TemplEntiRef>)>,
    id_query: &Query<&StrId>,
    selected_sprite: &mut Option<Entity>,
    camera_tile_pos: Option<GlobalTilePos>,
) -> Option<Entity> {
    let size = ChunkPos::CHUNK_SIZE;
    let chunk_origin = chunk_pos.to_tilepos();
    let mut by_local_pos: HashMap<(u32, u32), Vec<(Entity, String)>> = HashMap::new();

    for &child_entity in child_entities {
        if tile_storage_query.get(child_entity).is_ok() {
            continue;
        }
        let Ok((ent, gpos, templ_ref)) = spritetile_gpos_query.get(child_entity) else {
            continue;
        };
        if ChunkPos::from(*gpos) != chunk_pos {
            continue;
        }
        let local = gpos.0 - chunk_origin.0;
        if local.x < 0 || local.y < 0 || local.x >= size.x as i32 || local.y >= size.y as i32 {
            continue;
        }
        let display_str = if let Some(templ_ref) = templ_ref
            && let Ok(str_id) = id_query.get(templ_ref.0)
        {
            str_id.as_str().to_string()
        } else {
            format!("{}", ent.index())
        };
        by_local_pos
            .entry((local.x as u32, local.y as u32))
            .or_default()
            .push((ent, display_str));
    }

    let cell_w = 22.0f32;
    let cell_h = 18.0f32;
    let grid_size = egui::vec2(size.x as f32 * cell_w, size.y as f32 * cell_h);
    let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let mut clicked_spritetile = None;
    let camera_local = camera_tile_pos.map(|cam_pos| cam_pos.0 - chunk_origin.0);
    for y in (0..size.y).rev() {
        for x in 0..size.x {
            let row = (size.y - 1 - y) as f32;
            let col = x as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + col * cell_w, rect.top() + row * cell_h),
                egui::vec2(cell_w, cell_h),
            );
            let id = ui.make_persistent_id(("spritetiles_grid_cell", chunk_pos.0.x, chunk_pos.0.y, x, y));
            let response = ui.interact(cell_rect, id, egui::Sense::click());
            let is_camera_tile = camera_local.map_or(false, |local| local.x == x as i32 && local.y == y as i32);

            if let Some(sprite_stack) = by_local_pos.get(&(x, y)) {
                let (sprite_entity, sprite_id) = &sprite_stack[0];
                let is_selected = selected_sprite.map_or(false, |s| s == *sprite_entity);

                let label = short_tile_label(sprite_id);
                let fill = get_color_for_str_id(sprite_id).gamma_multiply(0.25);
                painter.rect_filled(cell_rect, 0.0, fill);
                painter.text(
                    cell_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    get_color_for_str_id(sprite_id),
                );
                if sprite_stack.len() > 1 {
                    painter.rect_stroke(
                        cell_rect.shrink(1.0),
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                        egui::StrokeKind::Inside,
                    );
                }
                if is_camera_tile {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        egui::StrokeKind::Outside,
                    );
                } else if is_selected {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(1.5, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                } else {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_gray(50)),
                        egui::StrokeKind::Inside,
                    );
                }
                if response.clicked() {
                    clicked_spritetile = Some(*sprite_entity);
                }
            } else {
                painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgb(16, 16, 16));
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_gray(35)),
                    egui::StrokeKind::Inside,
                );
                if is_camera_tile {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }
    }

    clicked_spritetile
}

#[allow(unused_parens)]
pub fn debug_chunking_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut chunking_ui: ResMut<DebugChunkingUiState>,
    mut chunk_range_settings: ResMut<LoadChunksAround>,
    chunk_query: Query<(
        Entity,
        &Chunk,
        &DimensionRef,
        &ChunkPos,
        Option<&Children>,
        Option<&Tilemaps>,
        Option<&TilesToSave>,
        &TerrGenState,
        Option<&ActivatingChunks>,
    ), With<Chunk>>,

    camera_dimension: Query<(&DimensionRef, &GlobalTransform, Option<&LoadChunksAround>), With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    loaded_chunks: Res<LoadedChunks>,
    tile_shader_map: Res<TileShaderEntityMap>,
    // Query for child entities to check their components
    tile_storage_query: Query<(Entity, &TileStorage, Option<&AcZ>, Option<&TileShaderRef>), With<TileStorage>>,
    tile_query: Query<(Entity, &TemplEntiRef, Option<&InitialPos>), With<Tile>>,
    spritetile_query: Query<(Entity, Has<SpriteTile>), ()>,
    spritetile_gpos_query: Query<(Entity, &GlobalTilePos, Option<&TemplEntiRef>)>,
    templ_query: Query<&TileStrId, With<Templ>>,
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
    let mut open = window_visible.chunks_list;

    // Get camera target dimension and position
    let (camera_dim_ref, camera_chunk_pos, camera_tile_pos, camera_chunk_settings) = camera_dimension
        .iter()
        .next()
        .map(|(dim_ref, transform, chunk_settings)| {
            let chunk_pos = ChunkPos::from(transform.translation());
            let tile_pos = GlobalTilePos::from(transform.translation().xy());
            (Some(dim_ref), Some(chunk_pos), Some(tile_pos), chunk_settings.copied())
        })
        .unwrap_or((None, None, None, None));
    let camera_dim_name = camera_dim_ref.as_ref().map(|camera_ref| dimension_name_for_ref(camera_ref, &dimension_map, &id_query));

    // Group chunks by dimension and position
    let mut chunks_by_dimension: BTreeMap<String, HashMap<ChunkPos, (Entity, Option<&Children>, Option<&Tilemaps>, Option<&TilesToSave>, TerrGenState, Option<&ActivatingChunks>)>> =
        BTreeMap::new();

    for (entity, _chunk, dim_ref, chunk_pos, children, tilemaps, tiles_to_save, terrgen_state, activating_chunks) in chunk_query.iter() {
        let dim_name = dimension_name_for_ref(&dim_ref, &dimension_map, &id_query);

        chunks_by_dimension
            .entry(dim_name)
            .or_insert_with(HashMap::new)
            .insert(*chunk_pos, (entity, children, tilemaps, tiles_to_save, *terrgen_state, activating_chunks));
    }

    if chunking_ui.follow_camera_chunk {
        if let (Some(dim_name), Some(cam_chunk)) = (camera_dim_name.as_ref(), camera_chunk_pos) {
            if let Some(chunks_map) = chunks_by_dimension.get(dim_name) {
                if let Some((entity, ..)) = chunks_map.get(&cam_chunk) {
                    selected_entities.selected_chunks.clear();
                    selected_entities.selected_chunks.insert(*entity);
                }
            }
        }
    }

    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = chunks_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        let camera_dim_str = dimension_name_for_ref(&camera_ref, &dimension_map, &id_query);
        if !camera_dim_str.is_empty() {
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
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Chunks: {}", chunk_query.iter().count()));
            ui.separator();
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [260.0, 32.0],
                            egui::Checkbox::new(
                                &mut chunking_ui.follow_camera_chunk,
                                egui::RichText::new("Follow Camera Chunk").size(18.0).strong(),
                            ),
                        );
                        let status = if chunking_ui.follow_camera_chunk { "ON" } else { "OFF" };
                        let status_color = if chunking_ui.follow_camera_chunk {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::LIGHT_RED
                        };
                        ui.label(egui::RichText::new(status).strong().color(status_color));
                    });
                });

            // Chunk Range Settings
            ui.heading("Range Settings");
                ui.horizontal(|ui| {
                    ui.label("Visibility Distance:");
                    ui.add(egui::DragValue::new(&mut chunk_range_settings.chunk_visib_max_dist).speed(10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Active Distance:");
                    ui.add(egui::DragValue::new(&mut chunk_range_settings.chunk_active_max_dist).speed(10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Discovery Range:");
                    ui.add(egui::DragValue::new(&mut chunk_range_settings.discovery_range).speed(1.0));
                });
                chunk_range_settings.chunk_visib_max_dist = chunk_range_settings.chunk_visib_max_dist.max(0.0);
                chunk_range_settings.chunk_active_max_dist = chunk_range_settings.chunk_active_max_dist.max(0.0);
                chunk_range_settings.discovery_range = chunk_range_settings.discovery_range.max(1);
                if let Some(chunk_settings) = camera_chunk_settings {
                    ui.separator();
                    ui.label("Camera target chunk component:");
                ui.horizontal(|ui| {
                    ui.label("Visibility Distance:");
                    ui.label(format!("{:.1}", chunk_settings.chunk_visib_max_dist));
                });
                ui.horizontal(|ui| {
                    ui.label("Active Distance:");
                    ui.label(format!("{:.1}", chunk_settings.chunk_active_max_dist));
                });
                ui.horizontal(|ui| {
                    ui.label("Discovery Range:");
                    ui.label(format!("{}", chunk_settings.discovery_range));
                });
            } else {
                ui.separator();
                ui.label("Camera target has no ActivateChunksAround");
            }
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(chunks_map) = chunks_by_dimension.get(dim_key) {
                    let is_camera_dim = camera_dim_ref.map_or(false, |camera_ref| {
                        dim_key == &dimension_name_for_ref(camera_ref, &dimension_map, &id_query)
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
                                            if let Some((entity, children, _tilemaps, _tiles_to_save, _terrgen_state, _activating_chunks)) = chunks_map.get(&pos) {
                                                let is_selected = selected_chunk == Some(*entity);
                                                let is_camera_pos = camera_chunk_pos.map_or(false, |cam_pos| cam_pos == pos);

                                                let children_count = children.map_or(0, |c| c.len());

                                                // Check if any child has TileStorage (TilemapType)
                                                let has_tilemap_child = children
                                                    .map(|children| {
                                                        children.iter().any(|child| tile_storage_query.get(child).is_ok())
                                                    })
                                                    .unwrap_or(false);

                                                let label = format!("{},{}\n {} ch:{}", x, y, entity.index(), children_count);

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
                                                    if !(is_camera_dim && is_camera_pos) {
                                                        chunking_ui.follow_camera_chunk = false;
                                                    }
                                                    chunking_ui.chunk_details_open_nonce = chunking_ui.chunk_details_open_nonce.wrapping_add(1);
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
                .flat_map(|(dim_key, map)| map.iter().map(move |(pos, data)| (dim_key, pos, data)))
                .filter(|(_, _, (entity, ..))| selected_entities.selected_chunks.contains(entity))
                .collect();
            selected_chunk_details.sort_by_key(|(_, _, (entity, ..))| entity.index());

            for (dim_key, chunk_pos, (entity, children, tilemaps, tiles_to_save, terrgen_state, activating_chunks)) in selected_chunk_details {
                let should_start_open = camera_dim_name.as_ref().is_some_and(|camera_dim_name| camera_dim_name == dim_key)
                    && camera_chunk_pos == Some(*chunk_pos);
                let is_selected = selected_entities.selected_chunks.contains(entity);
                egui::CollapsingHeader::new(format!("Details: {:?} ({:?})", chunk_pos, entity))
                    .id_salt(("chunk_details_header", entity.index(), if is_selected { chunking_ui.chunk_details_open_nonce } else { 0 }))
                    .default_open(is_selected || should_start_open)
                    .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let children_count = children.map_or(0, |c| c.len());
                        ui.label(format!("Children count: {}", children_count));

                        if let Some(tiles) = tiles_to_save {
                            ui.label(format!("TilesToSave: {} tiles", tiles.entities().len()));
                        }

                        ui.label(format!("TerrGenState: {:?}", terrgen_state));

                        if let Some(activating) = activating_chunks {
                            ui.label(format!("⏳ ActivatingChunks: {} positions",
                                activating.0.len()));
                        }

                        ui.separator();
                        if let Some(tilemaps_ref) = tilemaps {
                            egui::CollapsingHeader::new("Tilemaps")
                                .default_open(true)
                                .show(ui, |ui| {
                                    for tmap_entity in tilemaps_ref.iter() {
                                        if let Ok((tmap_entity, tile_storage, ac_z, shader_ref)) = tile_storage_query.get(tmap_entity) {
                                            let mut label = format!("🗺️ Tilemap ({})", tmap_entity.index());
                                            if let Some(z) = ac_z {
                                                label.push_str(&format!(" [Z: {:.1}]", z.0));
                                            }
                                            if let Some(shader_ref) = shader_ref
                                                && let Ok(shader_ent) = tile_shader_map.0.get_cloned(shader_ref.0)
                                                && let Ok(shader_str) = id_query.get(shader_ent)
                                            {
                                                label.push_str(&format!(" [Shader: {}]", shader_str.as_str()));
                                            }

                                            let shader_key = shader_ref
                                                .and_then(|s| tile_shader_map.0.get_cloned(s.0).ok())
                                                .and_then(|shader_ent| id_query.get(shader_ent).ok().map(|id| id.as_str().to_string()))
                                                .unwrap_or_else(|| "none".to_string());
                                            let z_key = ac_z.map(|z| format!("{:.3}", z.0)).unwrap_or_else(|| "none".to_string());
                                            let tilemap_type = format!(
                                                "shader:{}|z:{}|size:{}x{}",
                                                shader_key,
                                                z_key,
                                                tile_storage.size.x,
                                                tile_storage.size.y
                                            );
                                            let collapsing = egui::CollapsingHeader::new(label)
                                                .id_salt(format!("tilemap_type_{}", tilemap_type))
                                                .default_open(
                                                    chunking_ui.open_tilemap_type.as_deref()
                                                        == Some(tilemap_type.as_str()),
                                                )
                                                .show(ui, |ui| {
                                                    ui.label(format!("Size: {}x{}", tile_storage.size.x, tile_storage.size.y));

                                                    if ui.button("📋 View All Components").clicked() {
                                                        selected_entities.selected_tilemap = Some(tmap_entity);
                                                        window_visible.tilemap_details = true;
                                                    }

                                                    let camera_tile_pos_for_this_dim =
                                                        if camera_dim_name.as_deref() == Some(dim_key.as_str()) {
                                                            camera_tile_pos
                                                        } else {
                                                            None
                                                        };
                                                    if let Some(clicked_tile) = render_tilemap_grid(
                                                        ui,
                                                        tile_storage,
                                                        &tile_query,
                                                        &templ_query,
                                                        &mut selected_entities.selected_tile,
                                                        camera_tile_pos_for_this_dim,
                                                    ) {
                                                        selected_entities.selected_tile = Some(clicked_tile);
                                                        window_visible.tile_details = true;
                                                    }
                                                });
                                            if collapsing.fully_open() {
                                                chunking_ui.open_tilemap_type = Some(tilemap_type);
                                            }
                                        }
                                    }
                                });
                        } else {
                            ui.label("No tilemaps");
                        }

                        egui::CollapsingHeader::new("SpriteTilesMap")
                            .default_open(true)
                            .show(ui, |ui| {
                                let camera_tile_pos_for_this_dim =
                                    if camera_dim_name.as_deref() == Some(dim_key.as_str()) {
                                        camera_tile_pos
                                    } else {
                                        None
                                    };
                                let child_entities = children
                                    .map(|children| children.iter().collect::<Vec<_>>())
                                    .unwrap_or_default();
                                if let Some(clicked_spritetile) = render_spritetiles_grid(
                                    ui,
                                    *chunk_pos,
                                    &child_entities,
                                    &tile_storage_query,
                                    &spritetile_gpos_query,
                                    &id_query,
                                    &mut selected_entities.selected_sprite,
                                    camera_tile_pos_for_this_dim,
                                ) {
                                    selected_entities.selected_sprite = Some(clicked_spritetile);
                                    window_visible.sprite_details = true;
                                }
                            });

                        egui::CollapsingHeader::new("Non-tile Children")
                            .default_open(false)
                            .show(ui, |ui| {
                                if let Some(children_ref) = children {
                                    for child_entity in children_ref.iter() {
                                        let is_tilemap_entity = tilemaps.map_or(false, |tilemaps_ref| {
                                            tilemaps_ref.iter().any(|tmap_entity| tmap_entity == child_entity)
                                        });
                                        if is_tilemap_entity {
                                            continue;
                                        }
                                        if let Ok((_, is_sprite_tile)) = spritetile_query.get(child_entity)
                                            && is_sprite_tile
                                        {
                                            continue;
                                        }
                                        ui.label(format!("Child: {:?}", child_entity));
                                    }
                                } else {
                                    ui.label("No children");
                                }
                            });
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
                    .map(|(_, transform, _)| ChunkPos::from(transform.translation()));

                // Group chunks by dimension
                let mut chunks_by_dim: BTreeMap<String, Vec<(Entity, ChunkPos)>> = BTreeMap::new();

                for ((dim_ref, chunk_pos), entity) in loaded_chunks.0.iter() {
                    let dim_str_id = dimension_name_for_ref(dim_ref, &dimension_map, &id_query);

                    chunks_by_dim
                        .entry(dim_str_id)
                        .or_insert_with(Vec::new)
                        .push((*entity, *chunk_pos));
                }

                ui.label(format!("Total entries: {}", loaded_chunks.0.len()));

                // Display each dimension's chunks in a grid
                for (dim_name, chunks) in chunks_by_dim.into_iter() {
                    let is_camera_dim = camera_dim_name.as_ref().is_some_and(|camera_dim_name| camera_dim_name == &dim_name);
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
                                                        if !(is_camera_dim && is_camera_chunk) {
                                                            chunking_ui.follow_camera_chunk = false;
                                                        }
                                                        chunking_ui.chunk_details_open_nonce = chunking_ui.chunk_details_open_nonce.wrapping_add(1);
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
    window_visible.chunks_list = open;
}
