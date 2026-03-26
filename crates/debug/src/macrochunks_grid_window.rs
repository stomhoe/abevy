use bevy::{platform::collections::HashMap, prelude::*};
use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::debug_resources::{DebugChunkingUiState, DebugSelectedEntities, DubugWindowsVisibility};

use camera::camera_components::CameraTarget;
use common::common_components::StrId;
use tilemap::chunking::{chunking_components::{Chunk, MacroChunk, MacroChunkRef, TerrGenState}, macro_chunk_components::{BiomeDistribution, MacroChunkBiomePendingSampleState}};
use ::tilemap_shared::*;
use ::being_shared::*;
use wildlife::{NaturalSpawnOrigin, NaturalSpawnReservationIndex, SeededNaturalWildlifeMacroChunks};

#[derive(Clone, Copy, Default)]
struct MacroChunkChunkStats {
    total_chunks: usize,
    pending_chunks: usize,
    ready_chunks: usize,
    ops_launched_chunks: usize,
    finished_chunks: usize,
    disabled_chunks: usize,
}

impl MacroChunkChunkStats {
    fn push(self, terrgen_state: TerrGenState) -> Self {
        let mut next = self;
        next.total_chunks += 1;
        match terrgen_state {
            TerrGenState::Pending => next.pending_chunks += 1,
            TerrGenState::Ready => next.ready_chunks += 1,
            TerrGenState::OpsLaunched => next.ops_launched_chunks += 1,
            TerrGenState::Finished => next.finished_chunks += 1,
            TerrGenState::Disabled => next.disabled_chunks += 1,
        }
        next
    }
}

#[derive(Clone, Copy, Default)]
struct MacroChunkPendingWildlifeStats {
    reserved_beings: usize,
    watched_chunks: usize,
}

struct MacroChunkCell {
    entity: Entity,
    dim_ref: DimensionRef,
    pos: MacroChunkPos,
    biome_distribution: BiomeDistribution,
    biome_sampling_state: Option<MacroChunkBiomePendingSampleState>,
    loaded: bool,
    chunk_stats: MacroChunkChunkStats,
    chunk_states: Vec<(Entity, ChunkPos, TerrGenState)>,
    pending_wildlife: MacroChunkPendingWildlifeStats,
    seeded_for_wildlife: bool,
}

#[derive(Clone)]
struct OriginBeingEntry {
    entity: Entity,
    label: String,
}

fn origin_being_label(
    entity: Entity,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    id_query: &Query<&StrId>,
) -> String {
    if let Some(bit_ref) = bit_ref {
        if let Ok(str_id) = id_query.get(bit_ref.0) {
            return format!("{:?} | bit {}", entity, str_id.as_str());
        }
        return format!("{:?} | bit {:?}", entity, bit_ref.0);
    }
    if let Some(race_ref) = race_ref {
        if let Ok(str_id) = id_query.get(race_ref.0) {
            return format!("{:?} | race {}", entity, str_id.as_str());
        }
        return format!("{:?} | race {:?}", entity, race_ref.0);
    }
    format!("{:?}", entity)
}

fn chunk_state_color(terrgen_state: TerrGenState) -> egui::Color32 {
    match terrgen_state {
        TerrGenState::Pending => egui::Color32::from_rgb(95, 95, 95),
        TerrGenState::Ready => egui::Color32::from_rgb(90, 140, 225),
        TerrGenState::OpsLaunched => egui::Color32::from_rgb(215, 155, 60),
        TerrGenState::Finished => egui::Color32::from_rgb(70, 160, 95),
        TerrGenState::Disabled => egui::Color32::from_rgb(135, 65, 65),
    }
}

fn chunk_state_name(terrgen_state: TerrGenState) -> &'static str {
    match terrgen_state {
        TerrGenState::Pending => "Pending",
        TerrGenState::Ready => "Ready",
        TerrGenState::OpsLaunched => "OpsLaunched",
        TerrGenState::Finished => "Finished",
        TerrGenState::Disabled => "Disabled",
    }
}

fn short_macrochunk_label(label: &str) -> String {
    let mut chars = label.chars().filter(|c| c.is_ascii_alphanumeric());
    let Some(first) = chars.next() else {
        return "..".to_string();
    };
    let second = chars.next().unwrap_or(first);
    format!("{}{}", first.to_ascii_uppercase(), second.to_ascii_uppercase())
}

fn dominant_biome(distribution: &BiomeDistribution, id_query: &Query<&StrId>) -> Option<(String, f32)> {
    distribution
        .produced_biome_sampler
        .iter()
        .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(Ordering::Equal))
        .map(|(biome_ent, weight)| {
            let label = id_query
                .get(*biome_ent)
                .map(|str_id| str_id.as_str().to_string())
                .unwrap_or_else(|_| format!("{:?}", biome_ent));
            (label, *weight)
        })
}

fn biome_distribution_lines(distribution: &BiomeDistribution, id_query: &Query<&StrId>) -> Vec<String> {
    let total_weight: f32 = distribution.produced_biome_sampler.iter().map(|(_, weight)| *weight).sum();
    let mut lines: Vec<_> = distribution
        .produced_biome_sampler
        .iter()
        .map(|(biome_ent, weight)| {
            let label = id_query
                .get(*biome_ent)
                .map(|str_id| str_id.as_str().to_string())
                .unwrap_or_else(|_| format!("{:?}", biome_ent));
            let pack_stats = distribution.averaged_pack_count_multiplier_stats(*biome_ent);
            let share = if total_weight > 0.0 { (*weight / total_weight) * 100.0 } else { 0.0 };
            (label, *weight, share, pack_stats)
        })
        .collect();
    lines.sort_by(|lhs, rhs| rhs.1.partial_cmp(&lhs.1).unwrap_or(Ordering::Equal));
    lines
        .into_iter()
        .map(|(label, weight, share, pack_stats)| {
            format!(
                "{} | weight {:.2} ({:.1}%) | pack mean {:.2} std {:.2}",
                label,
                weight,
                share,
                pack_stats.averaged_mean(),
                pack_stats.averaged_std_dev(),
            )
        })
        .collect()
}

fn macrochunk_fill_color(cell: &MacroChunkCell) -> egui::Color32 {
    match cell.biome_sampling_state {
        Some(MacroChunkBiomePendingSampleState::Unsampled) => egui::Color32::from_rgb(52, 52, 52),
        Some(MacroChunkBiomePendingSampleState::Sampling { .. }) => egui::Color32::from_rgb(170, 120, 60),
        None if cell.seeded_for_wildlife => egui::Color32::from_rgb(60, 140, 82),
        None => egui::Color32::from_rgb(65, 105, 165),
    }
}

fn macrochunk_stroke(cell: &MacroChunkCell, is_selected: bool, is_camera_macrochunk: bool) -> egui::Stroke {
    if is_camera_macrochunk {
        egui::Stroke::new(2.0, egui::Color32::YELLOW)
    } else if is_selected {
        egui::Stroke::new(1.5, egui::Color32::WHITE)
    } else if cell.chunk_states.iter().any(|(_, _, state)| *state != TerrGenState::Pending) {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 200, 235))
    } else {
        egui::Stroke::new(0.5, egui::Color32::from_gray(35))
    }
}

fn macrochunk_hover_text(cell: &MacroChunkCell, id_query: &Query<&StrId>) -> String {
    let mut lines = vec![
        format!("Macrochunk: {}", cell.pos),
        format!("Entity: {:?}", cell.entity),
        format!("Loaded: {}", cell.loaded),
        format!("Discovered: {}", cell.chunk_states.iter().any(|(_, _, state)| *state != TerrGenState::Pending)),
        format!("Wildlife seeded: {}", cell.seeded_for_wildlife),
        format!(
            "Chunks: total {} | pending {} | ready {} | ops {} | finished {} | disabled {}",
            cell.chunk_stats.total_chunks,
            cell.chunk_stats.pending_chunks,
            cell.chunk_stats.ready_chunks,
            cell.chunk_stats.ops_launched_chunks,
            cell.chunk_stats.finished_chunks,
            cell.chunk_stats.disabled_chunks,
        ),
        format!(
            "Pending wildlife: reserved {} | watched chunks {}",
            cell.pending_wildlife.reserved_beings,
            cell.pending_wildlife.watched_chunks,
        ),
    ];
    match cell.biome_sampling_state {
        Some(MacroChunkBiomePendingSampleState::Unsampled) => lines.push("Biome state: Unsampled".to_string()),
        Some(MacroChunkBiomePendingSampleState::Sampling { remaining_samples }) => {
            lines.push(format!("Biome state: Sampling, {} remaining", remaining_samples));
            if let Some((label, weight)) = dominant_biome(&cell.biome_distribution, id_query) {
                lines.push(format!("Current dominant biome: {} ({:.2})", label, weight));
            }
        }
        None => {
            lines.push("Biome state: Sampled".to_string());
            if let Some((label, weight)) = dominant_biome(&cell.biome_distribution, id_query) {
                lines.push(format!("Dominant biome: {} ({:.2})", label, weight));
            }
        }
    }
    lines.join("\n")
}

fn render_macrochunks_grid(
    ui: &mut egui::Ui,
    dim_key: &str,
    macrochunks_map: &HashMap<MacroChunkPos, MacroChunkCell>,
    selected_macrochunk: Option<Entity>,
    camera_macrochunk_pos: Option<MacroChunkPos>,
    id_query: &Query<&StrId>,
) -> Option<Entity> {
    let positions: Vec<_> = macrochunks_map.keys().copied().collect();
    let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
        positions.iter().map(|pos| pos.0.x).min(),
        positions.iter().map(|pos| pos.0.x).max(),
        positions.iter().map(|pos| pos.0.y).min(),
        positions.iter().map(|pos| pos.0.y).max(),
    ) else {
        ui.label("No macrochunks in this dimension.");
        return None;
    };

    let width = (max_x - min_x + 1) as f32;
    let height = (max_y - min_y + 1) as f32;
    let max_grid_width = ui.available_width().max(120.0);
    let cell_side = ((max_grid_width / width).floor()).clamp(8.0, 28.0);
    let grid_size = egui::vec2(width * cell_side, height * cell_side);
    let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let mut clicked_macrochunk = None;

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let row = (max_y - y) as f32;
            let col = (x - min_x) as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + col * cell_side, rect.top() + row * cell_side),
                egui::vec2(cell_side, cell_side),
            );
            let pos = MacroChunkPos(IVec2::new(x, y));
            let id = ui.make_persistent_id(("macrochunks_grid_cell", dim_key, x, y));
            let Some(cell) = macrochunks_map.get(&pos) else {
                painter.rect_filled(cell_rect, 0.0, egui::Color32::from_gray(20));
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_gray(28)),
                    egui::StrokeKind::Inside,
                );
                continue;
            };

            let is_selected = selected_macrochunk == Some(cell.entity);
            let is_camera_macrochunk = camera_macrochunk_pos == Some(pos);
            let response = ui
                .interact(cell_rect, id, egui::Sense::click())
                .on_hover_text(macrochunk_hover_text(cell, id_query));

            painter.rect_filled(cell_rect, 0.0, macrochunk_fill_color(cell));
            painter.rect_stroke(
                cell_rect,
                0.0,
                macrochunk_stroke(cell, is_selected, is_camera_macrochunk),
                egui::StrokeKind::Inside,
            );

            if cell_side >= 16.0 {
                let label = match cell.biome_sampling_state {
                    Some(MacroChunkBiomePendingSampleState::Unsampled) => "..".to_string(),
                    Some(MacroChunkBiomePendingSampleState::Sampling { .. }) => "..".to_string(),
                    None => dominant_biome(&cell.biome_distribution, id_query)
                        .map(|(label, _)| short_macrochunk_label(&label))
                        .unwrap_or_else(|| "??".to_string()),
                };
                painter.text(
                    cell_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional((cell_side * 0.42).clamp(8.0, 14.0)),
                    egui::Color32::WHITE,
                );
            }

            if response.clicked() {
                clicked_macrochunk = Some(cell.entity);
            }
        }
    }

    clicked_macrochunk
}

fn render_chunk_states_map(
    ui: &mut egui::Ui,
    cell: &MacroChunkCell,
    selected_chunk: Option<ChunkPos>,
    current_chunk: Option<ChunkPos>,
    origin_counts_by_chunk: &HashMap<ChunkPos, usize>,
) -> Option<ChunkPos> {
    let width = MacroChunkPos::SIZE_IN_CHUNKS.0.x as f32;
    let height = MacroChunkPos::SIZE_IN_CHUNKS.0.y as f32;
    let available_size = ui.available_size_before_wrap();
    let max_grid_side = available_size.x.min(available_size.y).min(240.0);
    let cell_side = (available_size.x / width)
        .min(max_grid_side / width)
        .floor()
        .clamp(4.0, 24.0);
    let grid_size = egui::vec2(width * cell_side, height * cell_side);
    let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let macrochunk_origin = cell.pos.to_chunkpos();
    let mut chunks_by_local_pos: HashMap<(i32, i32), (Entity, ChunkPos, TerrGenState)> = HashMap::default();
    let mut clicked_chunk = None;

    for &(entity, chunk_pos, terrgen_state) in &cell.chunk_states {
        let local = chunk_pos.0 - macrochunk_origin.0;
        chunks_by_local_pos.insert((local.x, local.y), (entity, chunk_pos, terrgen_state));
    }

    for local_y in (0..MacroChunkPos::SIZE_IN_CHUNKS.0.y as i32).rev() {
        for local_x in 0..MacroChunkPos::SIZE_IN_CHUNKS.0.x as i32 {
            let row = (MacroChunkPos::SIZE_IN_CHUNKS.0.y as i32 - 1 - local_y) as f32;
            let col = local_x as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + col * cell_side, rect.top() + row * cell_side),
                egui::vec2(cell_side, cell_side),
            );
            let response = ui.interact(
                cell_rect,
                ui.make_persistent_id(("macrochunk_chunk_state_cell", cell.entity.index(), local_x, local_y)),
                egui::Sense::click(),
            );
            let chunk_pos = ChunkPos(macrochunk_origin.0 + IVec2::new(local_x, local_y));
            let origin_count = origin_counts_by_chunk.get(&chunk_pos).copied().unwrap_or(0);
            if let Some(&(entity, _, terrgen_state)) = chunks_by_local_pos.get(&(local_x, local_y)) {
                painter.rect_filled(cell_rect, 0.0, chunk_state_color(terrgen_state));
                let stroke = if current_chunk == Some(chunk_pos) {
                    egui::Stroke::new(2.0, egui::Color32::YELLOW)
                } else if selected_chunk == Some(chunk_pos) {
                    egui::Stroke::new(2.0, egui::Color32::WHITE)
                } else {
                    egui::Stroke::new(0.5, egui::Color32::from_gray(35))
                };
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    stroke,
                    egui::StrokeKind::Inside,
                );
                if origin_count > 0 {
                    let radius = (cell_side * 0.18).clamp(2.5, 5.0);
                    painter.circle_filled(
                        egui::pos2(cell_rect.right() - radius - 2.0, cell_rect.top() + radius + 2.0),
                        radius,
                        egui::Color32::from_rgb(110, 255, 160),
                    );
                    if cell_side >= 18.0 {
                        painter.text(
                            egui::pos2(cell_rect.center().x, cell_rect.bottom() - 2.0),
                            egui::Align2::CENTER_BOTTOM,
                            origin_count.to_string(),
                            egui::FontId::proportional((cell_side * 0.3).clamp(8.0, 12.0)),
                            egui::Color32::BLACK,
                        );
                    }
                }
                response.clone().on_hover_text(format!(
                    "Chunk {}\nEntity: {:?}\nState: {}\nOrigin beings: {}",
                    chunk_pos,
                    entity,
                    chunk_state_name(terrgen_state),
                    origin_count,
                ));
            } else {
                painter.rect_filled(cell_rect, 0.0, egui::Color32::from_gray(18));
                let stroke = if current_chunk == Some(chunk_pos) {
                    egui::Stroke::new(2.0, egui::Color32::YELLOW)
                } else if selected_chunk == Some(chunk_pos) {
                    egui::Stroke::new(2.0, egui::Color32::WHITE)
                } else {
                    egui::Stroke::new(0.5, egui::Color32::from_gray(28))
                };
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    stroke,
                    egui::StrokeKind::Inside,
                );
                if origin_count > 0 {
                    let radius = (cell_side * 0.18).clamp(2.5, 5.0);
                    painter.circle_filled(
                        egui::pos2(cell_rect.right() - radius - 2.0, cell_rect.top() + radius + 2.0),
                        radius,
                        egui::Color32::from_rgb(110, 255, 160),
                    );
                }
                response.clone().on_hover_text(format!(
                    "Chunk {}\nState: Missing\nOrigin beings: {}",
                    chunk_pos,
                    origin_count,
                ));
            }
            if response.clicked() {
                clicked_chunk = Some(chunk_pos);
            }
        }
    }

    clicked_chunk
}

fn show_macrochunk_details(
    ui: &mut egui::Ui,
    window_visible: &mut DubugWindowsVisibility,
    selected_entities: &mut DebugSelectedEntities,
    chunking_ui: &mut DebugChunkingUiState,
    dim_name: &str,
    dim_ref: DimensionRef,
    cell: &MacroChunkCell,
    camera_chunk_pos: Option<ChunkPos>,
    origin_beings_by_chunk: &HashMap<(DimensionRef, ChunkPos), Vec<OriginBeingEntry>>,
    id_query: &Query<&StrId>,
) {
    ui.heading("Selected MacroChunk");
    ui.separator();
    ui.label(format!("Dimension: {}", dim_name));
    ui.label(format!("Entity: {:?}", cell.entity));
    ui.label(format!("{}", cell.pos));
    ui.label(format!("Loaded: {}", cell.loaded));
    ui.label(format!("Discovered: {}", cell.chunk_states.iter().any(|(_, _, state)| *state != TerrGenState::Pending)));
    ui.label(format!("Wildlife seeded: {}", cell.seeded_for_wildlife));
    ui.label(format!(
        "Pending wildlife: reserved {} | watched chunks {}",
        cell.pending_wildlife.reserved_beings,
        cell.pending_wildlife.watched_chunks,
    ));
    ui.separator();
    ui.label(format!(
        "Chunk states: total {} | pending {} | ready {} | ops {} | finished {} | disabled {}",
        cell.chunk_stats.total_chunks,
        cell.chunk_stats.pending_chunks,
        cell.chunk_stats.ready_chunks,
        cell.chunk_stats.ops_launched_chunks,
        cell.chunk_stats.finished_chunks,
        cell.chunk_stats.disabled_chunks,
    ));
    egui::CollapsingHeader::new("Chunk states map")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(chunk_state_color(TerrGenState::Pending), "Pending");
                ui.colored_label(chunk_state_color(TerrGenState::Ready), "Ready");
                ui.colored_label(chunk_state_color(TerrGenState::OpsLaunched), "OpsLaunched");
                ui.colored_label(chunk_state_color(TerrGenState::Finished), "Finished");
                ui.colored_label(chunk_state_color(TerrGenState::Disabled), "Disabled");
                ui.colored_label(egui::Color32::from_gray(18), "Missing");
                ui.colored_label(egui::Color32::YELLOW, "Current chunk");
                ui.colored_label(egui::Color32::from_rgb(110, 255, 160), "NaturalSpawnOrigin present");
            });
            ui.separator();

            if chunking_ui
                .selected_macrochunk_chunk
                .is_some_and(|chunk_pos| !cell.pos.contains_chunkpos(chunk_pos))
            {
                chunking_ui.selected_macrochunk_chunk = None;
            }

            let current_chunk = camera_chunk_pos.filter(|chunk_pos| cell.pos.contains_chunkpos(*chunk_pos));
            let mut origin_counts_by_chunk: HashMap<ChunkPos, usize> = HashMap::default();
            for (&(entry_dim_ref, chunk_pos), origin_beings) in origin_beings_by_chunk.iter() {
                if entry_dim_ref == dim_ref && cell.pos.contains_chunkpos(chunk_pos) {
                    origin_counts_by_chunk.insert(chunk_pos, origin_beings.len());
                }
            }

            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    if let Some(clicked_chunk) = render_chunk_states_map(
                        ui,
                        cell,
                        chunking_ui.selected_macrochunk_chunk,
                        current_chunk,
                        &origin_counts_by_chunk,
                    ) {
                        chunking_ui.selected_macrochunk_chunk = Some(clicked_chunk);
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_min_width(240.0);
                    if let Some(selected_chunk_pos) = chunking_ui.selected_macrochunk_chunk {
                        let selected_chunk_entity = cell
                            .chunk_states
                            .iter()
                            .find_map(|(entity, chunk_pos, _)| (*chunk_pos == selected_chunk_pos).then_some(*entity));
                        let selected_chunk_state = cell
                            .chunk_states
                            .iter()
                            .find_map(|(_, chunk_pos, terrgen_state)| (*chunk_pos == selected_chunk_pos).then_some(*terrgen_state));
                        ui.label(format!("Selected chunk: {}", selected_chunk_pos));
                        if let Some(terrgen_state) = selected_chunk_state {
                            ui.label(format!("TerrGenState: {}", chunk_state_name(terrgen_state)));
                        } else {
                            ui.label("TerrGenState: Missing");
                        }
                        ui.label(format!(
                            "Origin beings: {}",
                            origin_beings_by_chunk
                                .get(&(dim_ref, selected_chunk_pos))
                                .map(|beings| beings.len())
                                .unwrap_or(0)
                        ));
                        let show_chunk_details = ui.add_enabled(
                            selected_chunk_entity.is_some(),
                            egui::Button::new("Show Chunk Details"),
                        );
                        if show_chunk_details.clicked() {
                            selected_entities.selected_chunks.clear();
                            selected_entities.selected_chunks.insert(selected_chunk_entity.unwrap());
                            chunking_ui.chunk_details_open_nonce = chunking_ui.chunk_details_open_nonce.wrapping_add(1);
                            window_visible.chunk_details = true;
                        }
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height().max(160.0))
                            .show(ui, |ui| {
                                if let Some(origin_beings) = origin_beings_by_chunk.get(&(dim_ref, selected_chunk_pos)) {
                                    for origin_being in origin_beings {
                                        ui.label(&origin_being.label);
                                    }
                                } else {
                                    ui.label("No beings have NaturalSpawnOrigin on this chunk.");
                                }
                            });
                    } else {
                        ui.label("Select a chunk in the map to inspect origin beings.");
                    }
                });
            });
        });
    ui.separator();
    match cell.biome_sampling_state {
        Some(MacroChunkBiomePendingSampleState::Unsampled) => {
            ui.label("Biome state: Unsampled");
        }
        Some(MacroChunkBiomePendingSampleState::Sampling { remaining_samples }) => {
            ui.label(format!("Biome state: Sampling, {} remaining", remaining_samples));
            if let Some((label, weight)) = dominant_biome(&cell.biome_distribution, id_query) {
                ui.label(format!("Current dominant biome: {} ({:.2})", label, weight));
            }
            ui.separator();
            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                for line in biome_distribution_lines(&cell.biome_distribution, id_query) {
                    ui.label(line);
                }
            });
        }
        None => {
            ui.label("Biome state: Sampled");
            if let Some((label, weight)) = dominant_biome(&cell.biome_distribution, id_query) {
                ui.label(format!("Dominant biome: {} ({:.2})", label, weight));
            }
            ui.separator();
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                for line in biome_distribution_lines(&cell.biome_distribution, id_query) {
                    ui.label(line);
                }
            });
        }
    }
}

#[allow(unused_parens, )]
pub fn macrochunks_grid_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut chunking_ui: ResMut<DebugChunkingUiState>,
    macro_chunk_query: Query<
        (
            Entity,
            &DimensionRef,
            &MacroChunkPos,
            &BiomeDistribution,
        ),
        With<MacroChunk>,
    >,
    macro_chunk_biome_sampling_states: Query<&MacroChunkBiomePendingSampleState>,
    chunk_query: Query<(Entity, &MacroChunkRef, &TerrGenState, &ChunkPos), With<Chunk>>,
    camera_dimension: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    loaded_macro_chunks: Res<LoadedMacroChunks>,
    id_query: Query<&StrId>,
    seeded_macrochunks: Option<Res<SeededNaturalWildlifeMacroChunks>>,
    pending_wildlife_by_chunk: Option<Res<NaturalSpawnReservationIndex>>,
    origin_being_query: Query<(Entity, &DimensionRef, &NaturalSpawnOrigin, Option<&BitRef>, Option<&RaceRef>)>,
) {
    if !window_visible.macrochunks_grid {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 24.0;
    let default_y = screen_rect.top() + 24.0;
    let mut open = window_visible.macrochunks_grid;

    let (camera_dim_ref, camera_chunk_pos, camera_macrochunk_pos) = camera_dimension
        .iter()
        .next()
        .map(|(dim_ref, transform)| {
            let camera_chunk_pos = ChunkPos::from(transform.translation());
            (Some(*dim_ref), Some(camera_chunk_pos), Some(camera_chunk_pos.to_macrochunk_pos()))
        })
        .unwrap_or((None, None, None));
    let camera_dim_name = camera_dim_ref.and_then(|dim_ref| {
        id_query
            .get(dim_ref.0)
            .ok()
            .map(|str_id| str_id.as_str().to_string())
    });

    let mut chunk_stats_by_macrochunk: HashMap<Entity, MacroChunkChunkStats> = HashMap::default();
    let mut chunk_states_by_macrochunk: HashMap<Entity, Vec<(Entity, ChunkPos, TerrGenState)>> = HashMap::default();
    for (entity, &macro_chunk_ref, &terrgen_state, &chunk_pos) in chunk_query.iter() {
        let current = chunk_stats_by_macrochunk
            .get(&macro_chunk_ref.0)
            .copied()
            .unwrap_or_default();
        chunk_stats_by_macrochunk.insert(macro_chunk_ref.0, current.push(terrgen_state));
        chunk_states_by_macrochunk
            .entry(macro_chunk_ref.0)
            .or_default()
            .push((entity, chunk_pos, terrgen_state));
    }

    let mut origin_beings_by_chunk: HashMap<(DimensionRef, ChunkPos), Vec<OriginBeingEntry>> = HashMap::default();
    for (entity, &dim_ref, origin, bit_ref, race_ref) in origin_being_query.iter() {
        origin_beings_by_chunk
            .entry((dim_ref, origin.0))
            .or_default()
            .push(OriginBeingEntry {
                entity,
                label: origin_being_label(entity, bit_ref, race_ref, &id_query),
            });
    }
    origin_beings_by_chunk.values_mut().for_each(|entries| entries.sort_by_key(|entry| entry.entity.index()));

    let mut pending_wildlife_by_macrochunk: HashMap<(DimensionRef, MacroChunkPos), MacroChunkPendingWildlifeStats> = HashMap::default();
    if let Some(pending_wildlife) = pending_wildlife_by_chunk.as_ref() {
        for (&(dim_ref, chunk_pos), being_ents) in &pending_wildlife.by_chunk {
            let entry = pending_wildlife_by_macrochunk
                .entry((dim_ref, chunk_pos.to_macrochunk_pos()))
                .or_default();
            entry.watched_chunks += 1;
            let _ = being_ents;
        }
        for &(dim_ref, chunk_pos) in pending_wildlife.reservation_by_being.values() {
            let entry = pending_wildlife_by_macrochunk
                .entry((dim_ref, chunk_pos.to_macrochunk_pos()))
                .or_default();
            entry.reserved_beings += 1;
        }
    }

    let mut macrochunks_by_dimension: BTreeMap<String, HashMap<MacroChunkPos, MacroChunkCell>> = BTreeMap::new();
    let mut selected_macrochunk_dimension = None;

    for (entity, &dim_ref, &macro_chunk_pos, biome_distribution) in macro_chunk_query.iter() {
        let dim_name = id_query
            .get(dim_ref.0)
            .map(|str_id| str_id.as_str().to_string())
            .unwrap_or_else(|_| format!("{:?}", dim_ref));
        if selected_entities.selected_macrochunk == Some(entity) {
            selected_macrochunk_dimension = Some(dim_name.clone());
        }
        let biome_sampling_state = macro_chunk_biome_sampling_states.get(entity).ok().copied();
        let seeded_for_wildlife = seeded_macrochunks
            .as_ref()
            .is_some_and(|seeded| seeded.0.contains(&(dim_ref, macro_chunk_pos)));
        let pending_wildlife = pending_wildlife_by_macrochunk
            .get(&(dim_ref, macro_chunk_pos))
            .copied()
            .unwrap_or_default();
        let chunk_stats = chunk_stats_by_macrochunk
            .get(&entity)
            .copied()
            .unwrap_or_default();
        let chunk_states = chunk_states_by_macrochunk.remove(&entity).unwrap_or_default();
        let loaded = loaded_macro_chunks.0.get(&(dim_ref, macro_chunk_pos)).copied() == Some(entity);
        macrochunks_by_dimension
            .entry(dim_name)
            .or_default()
            .insert(
                macro_chunk_pos,
                MacroChunkCell {
                    entity,
                    dim_ref,
                    pos: macro_chunk_pos,
                    biome_distribution: biome_distribution.clone(),
                    biome_sampling_state,
                    loaded,
                    chunk_stats,
                    chunk_states,
                    pending_wildlife,
                    seeded_for_wildlife,
                },
            );
    }

    if chunking_ui.follow_camera_macrochunk {
        if let (Some(dim_name), Some(camera_macrochunk_pos)) = (camera_dim_name.as_ref(), camera_macrochunk_pos) {
            if let Some(cell) = macrochunks_by_dimension
                .get(dim_name)
                .and_then(|map| map.get(&camera_macrochunk_pos))
            {
                selected_entities.selected_macrochunk = Some(cell.entity);
                selected_macrochunk_dimension = Some(dim_name.clone());
            }
        }
    }

    let mut sorted_dims: Vec<_> = macrochunks_by_dimension.keys().cloned().collect();
    if let Some(camera_dim_name) = camera_dim_name.as_ref() {
        sorted_dims.sort_by(|lhs, rhs| {
            if lhs == camera_dim_name {
                Ordering::Less
            } else if rhs == camera_dim_name {
                Ordering::Greater
            } else {
                lhs.cmp(rhs)
            }
        });
    }

    egui::Window::new("MacroChunks Grid")
        .default_pos([default_x, default_y])
        .default_width(760.0)
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Macrochunks: {}", macro_chunk_query.iter().count()));
            ui.separator();
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [300.0, 28.0],
                            egui::Checkbox::new(
                                &mut chunking_ui.follow_camera_macrochunk,
                                egui::RichText::new("Follow Camera MacroChunk").size(18.0).strong(),
                            ),
                        );
                        let status = if chunking_ui.follow_camera_macrochunk { "ON" } else { "OFF" };
                        let status_color = if chunking_ui.follow_camera_macrochunk {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::LIGHT_RED
                        };
                        ui.label(egui::RichText::new(status).strong().color(status_color));
                    });
                });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(egui::Color32::from_rgb(52, 52, 52), "Unsampled");
                ui.colored_label(egui::Color32::from_rgb(180, 130, 60), "Sampling");
                ui.colored_label(egui::Color32::from_rgb(65, 105, 165), "Sampled");
                ui.colored_label(egui::Color32::from_rgb(60, 140, 82), "Sampled + wildlife seeded");
            });
            ui.separator();

            for dim_key in &sorted_dims {
                let is_camera_dim = camera_dim_name.as_ref() == Some(dim_key);
                egui::CollapsingHeader::new(dim_key)
                    .default_open(is_camera_dim)
                    .show(ui, |ui| {
                        let Some(macrochunks_map) = macrochunks_by_dimension.get(dim_key) else {
                            return;
                        };
                        if let Some(clicked_macrochunk) = render_macrochunks_grid(
                            ui,
                            dim_key,
                            macrochunks_map,
                            selected_entities.selected_macrochunk,
                            if is_camera_dim { camera_macrochunk_pos } else { None },
                            &id_query,
                        ) {
                            if !(is_camera_dim
                                && camera_macrochunk_pos
                                    .and_then(|camera_pos| macrochunks_map.get(&camera_pos))
                                    .is_some_and(|cell| cell.entity == clicked_macrochunk))
                            {
                                chunking_ui.follow_camera_macrochunk = false;
                            }
                            chunking_ui.selected_macrochunk_chunk = None;
                            selected_entities.selected_macrochunk = Some(clicked_macrochunk);
                            selected_macrochunk_dimension = Some(dim_key.clone());
                        }
                    });
            }

            ui.separator();
            let selected_details = selected_entities.selected_macrochunk.and_then(|selected_entity| {
                macrochunks_by_dimension.iter().find_map(|(dim_name, map)| {
                    map.values()
                        .find(|cell| cell.entity == selected_entity)
                        .map(|cell| (dim_name.as_str(), cell))
                })
            });
            if let Some((dim_name, cell)) = selected_details {
                let current_chunk = if camera_dim_name.as_deref() == Some(dim_name) {
                    camera_chunk_pos
                } else {
                    None
                };
                show_macrochunk_details(
                    ui,
                    &mut window_visible,
                    &mut selected_entities,
                    &mut chunking_ui,
                    dim_name,
                    cell.dim_ref,
                    cell,
                    current_chunk,
                    &origin_beings_by_chunk,
                    &id_query,
                );
            } else if let Some(dim_name) = selected_macrochunk_dimension {
                ui.label(format!("Selected macrochunk is no longer loaded in {}.", dim_name));
            } else {
                ui.label("Select a macrochunk to inspect its wildlife and biome state.");
            }
        });

    window_visible.macrochunks_grid = open;
}
