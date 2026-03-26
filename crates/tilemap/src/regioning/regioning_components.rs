use game_common::game_common_components::{ TemplEntiRef};

use bevy::{ecs::entity::EntityHashMap, platform::collections::{HashMap, HashSet}, prelude::*};
use ::tilemap_shared::*;

use crate::{regioning::regioning_messages::ChunksClaim, tile::tile_components::*};
use ::tilemap_shared::DeleteOtherTilesInSamePos;
use bevy_inspector_egui::{egui, };

use common::{common_components::*, };
use serde::{Deserialize, Serialize};

pub use ::tilemap_shared::ActiveChunksInRegion;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(ClaimList, RegionPlannedTiles, RegionState, Visibility, Transform, AssetScoped)]
pub struct Region;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum RegionState {
    #[default]
    OfferingChunks,
    ClaimsProcessed,
    BuildingStarted,
    AllTilesPrepared,
}

#[derive(Component, Debug, Clone)]
#[require(CountsOfSgcs, GridOfSgcs)]
pub struct ClaimList {
    pub processed_up_to_i: usize,
    pub claims: Vec<Option<ChunksClaim>>,
    pub skipped_is: HashSet<usize>,
    pub advance_timer: Timer,
}
impl ClaimList {
    pub fn advance_processed_upto_i(&mut self) {
        self.processed_up_to_i += 1;
        self.advance_timer.reset();
    }
    pub fn reached_end(&self) -> bool {
        self.processed_up_to_i >= MAX_CLAIMS
    }
}
impl Default for ClaimList {
    fn default() -> Self {
        let mut claims = Vec::with_capacity(MAX_CLAIMS);
        claims.resize(MAX_CLAIMS, None);
        Self {
            claims,
            processed_up_to_i: 0,
            skipped_is: HashSet::new(),
            advance_timer: Timer::from_seconds(0.02, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct CountsOfSgcs (pub EntityHashMap<u32>,);

pub type TilesFromBuilder = Vec<(GlobalTilePos, TemplEntiRef, Option<DeleteOtherTilesInSamePos>)>;

#[derive(Debug, Clone)]
pub struct PendingBuildOrder {
    pub chunks: Vec<ChunkPos>,
    pub timer: Timer,
}

#[derive(Component, Debug, Default, Clone)]
pub struct RegionPlannedTiles {
    tiles_to_spawn_on_chunk_load_map: HashMap<ChunkPos, TilesFromBuilder>,
    terrgen_disabled_gpos_on_chunk_load_map: HashMap<ChunkPos, HashSet<GlobalTilePos>>,
    // store pending build orders along with their timeout timer
    pending_build_orders: HashMap<u64, PendingBuildOrder>,
    pending_chunks: HashSet<ChunkPos>,
}

impl RegionPlannedTiles {
    pub fn new(order_i: u64, chunk_positions: &[ChunkPos], timeout_secs: f32) -> Self {
        let mut planned = Self::default();
        planned.add_build_order_pending(order_i, chunk_positions, timeout_secs);
        planned
    }
    pub fn add_build_order_pending(&mut self, order_i: u64, chunk_positions: &[ChunkPos], timeout_secs: f32) {
        let timer = Timer::from_seconds(timeout_secs, TimerMode::Once);
        if let Some(previous) = self.pending_build_orders.insert(order_i, PendingBuildOrder {
            chunks: chunk_positions.to_vec(),
            timer,
        }) {
            for pos in previous.chunks {
                self.pending_chunks.remove(&pos);
            }
        }
        for &pos in chunk_positions {
            self.pending_chunks.insert(pos);
        }
    }

    pub fn is_chunk_pending_build(&self, chunk_pos: ChunkPos) -> bool {
        self.pending_chunks.contains(&chunk_pos)
    }
    pub fn pending_chunks_count(&self) -> usize {
        self.pending_chunks.len()
    }

    pub fn add_planned_tiles_and_remove_from_pending(
        &mut self,
        order_i: u64,
        chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)>,
        terrgen_disabled_gpos_for_chunks: Vec<(ChunkPos, HashSet<GlobalTilePos>)>,
    ) -> Result<bool, BevyError> {
        let Some(order) = self.pending_build_orders.remove(&order_i) else {
            return Err(BevyError::from(format!(
                "Build order {} is not pending",
                order_i
            )));
        };
        let mut provided_chunks: HashSet<ChunkPos> = HashSet::new();
        for (chunk_pos, tile_data) in chunk_tiles {
            if !order.chunks.contains(&chunk_pos) {
                return Err(BevyError::from(format!(
                    "ChunkPos {:?} is not part of build order {}",
                    chunk_pos, order_i
                )));
            }
            for (tile_pos, _, _) in &tile_data {
                chunk_pos.is_tilepos_within_chunk(*tile_pos)?;
            }
            self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new).extend(tile_data);
            provided_chunks.insert(chunk_pos);
        }
        for (chunk_pos, blocked_gpos) in terrgen_disabled_gpos_for_chunks {
            if !order.chunks.contains(&chunk_pos) {
                return Err(BevyError::from(format!(
                    "ChunkPos {:?} is not part of build order {}",
                    chunk_pos, order_i
                )));
            }
            self.terrgen_disabled_gpos_on_chunk_load_map
                .entry(chunk_pos)
                .or_default()
                .extend(blocked_gpos);
            provided_chunks.insert(chunk_pos);
        }
        for chunk_pos in order.chunks {
            if !provided_chunks.contains(&chunk_pos) {
                self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new);
                self.terrgen_disabled_gpos_on_chunk_load_map.entry(chunk_pos).or_default();
            }
            self.pending_chunks.remove(&chunk_pos);
        }
        Ok(self.pending_build_orders.is_empty())
    }

    pub fn get(&self, chunk_pos: &ChunkPos,) -> Option<&TilesFromBuilder> {
        self.tiles_to_spawn_on_chunk_load_map.get(chunk_pos)
    }

    pub fn take_terrgen_disabled_gpos(&mut self, chunk_pos: ChunkPos) -> HashSet<GlobalTilePos> {
        self.terrgen_disabled_gpos_on_chunk_load_map.remove(&chunk_pos).unwrap_or_default()
    }

    pub fn pending_build_orders_iter(&self) -> impl Iterator<Item = (&u64, &PendingBuildOrder)> {
        self.pending_build_orders.iter()
    }

    pub fn pending_build_orders_iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut PendingBuildOrder)> {
        self.pending_build_orders.iter_mut()
    }

    pub fn take_pending_build_order(&mut self, order_i: u64) -> Option<PendingBuildOrder> {
        self.pending_build_orders.remove(&order_i)
    }

    pub fn mark_chunk_timed_out(&mut self, chunk_pos: ChunkPos) {
        self.pending_chunks.remove(&chunk_pos);
        self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new);
        self.terrgen_disabled_gpos_on_chunk_load_map.entry(chunk_pos).or_default();
    }
    pub fn planned_tiles_at_gpos(&self, gpos: GlobalTilePos) -> Option<&[(GlobalTilePos, TemplEntiRef, Option<DeleteOtherTilesInSamePos>)]> {
        let chunk_pos = gpos.to_chunkpos();
        self.tiles_to_spawn_on_chunk_load_map.get(&chunk_pos).map(Vec::as_slice)
    }

}

pub const MAX_CLAIMS: usize = REGION_SIZE_IN_CHUNKS.area_usize();

#[derive(Debug, Clone)]
pub struct RegionGrid<T: Copy + Eq> {
    grid: Vec<Vec<T>>,
    count: u64,
}
impl<T: Copy + Eq> Default for RegionGrid<T> {
    fn default() -> Self {
        let total_cells = REGION_SIZE_IN_CHUNKS.area_usize();
        let mut grid = Vec::with_capacity(total_cells);
        grid.resize_with(total_cells, Vec::new);
        Self {
            grid,
            count: 0,
        }
    }
}

impl<T: Copy + Eq> RegionGrid<T> {
    #[inline]
    const fn width() -> usize {
        REGION_SIZE_IN_CHUNKS.0.x as usize
    }
    #[inline]
    fn flat_index(x: usize, y: usize) -> usize {
        y * Self::width() + x
    }
    #[inline]
    fn cell(&self, x: usize, y: usize) -> &Vec<T> {
        &self.grid[Self::flat_index(x, y)]
    }
    #[inline]
    fn cell_mut(&mut self, x: usize, y: usize) -> &mut Vec<T> {
        &mut self.grid[Self::flat_index(x, y)]
    }
    #[inline]
    fn cell_opt(&self, x: usize, y: usize) -> Option<&Vec<T>> {
        self.grid.get(Self::flat_index(x, y))
    }
    #[inline]
    fn get_local_pos(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> Result<(usize, usize), ChunkOccupyError> {
        let local_chunk_pos = global_chunk_pos - region_pos.to_chunkpos();
        match (local_chunk_pos.0.x < 0, local_chunk_pos.0.x >= REGION_SIZE_IN_CHUNKS.x(), local_chunk_pos.0.y < 0, local_chunk_pos.0.y >= REGION_SIZE_IN_CHUNKS.y()) {
            (true, _, _, _) => Err(ChunkOccupyError::OutOfRegionBounds(CardinalDirection::West)),
            (_, true, _, _) => Err(ChunkOccupyError::OutOfRegionBounds(CardinalDirection::East)),
            (_, _, true, _) => Err(ChunkOccupyError::OutOfRegionBounds(CardinalDirection::South)),
            (_, _, _, true) => Err(ChunkOccupyError::OutOfRegionBounds(CardinalDirection::North)),
            _ => Ok((local_chunk_pos.0.x as usize, local_chunk_pos.0.y as usize)),
        }
    }
    pub fn is_occupied(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> bool {
        self.get_local_pos(global_chunk_pos, region_pos)
        .map(|(x, y)| !self.cell(x, y).is_empty())
        .unwrap_or(false)
    }
    pub fn is_available(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> bool {
        !self.is_occupied(global_chunk_pos, region_pos)
    }
    pub fn occupy(&mut self, global_chunk_pos: ChunkPos, region_pos: RegionPos, value: T) -> Result<(), ChunkOccupyError> {
        let (x, y) = self.get_local_pos(global_chunk_pos, region_pos)?;
        let was_empty = {
            let cell = self.cell_mut(x, y);
            if cell.iter().any(|&existing| existing == value) {
                return Err(ChunkOccupyError::AlreadyOccupied);
            }
            let was_empty = cell.is_empty();
            cell.push(value);
            was_empty
        };
        if was_empty {
            self.count += 1;
        }
        Ok(())
    }
    pub fn free(&mut self, global_chunk_pos: ChunkPos, region_pos: RegionPos, value: T) {
        if let Ok((x, y)) = self.get_local_pos(global_chunk_pos, region_pos) {
            let cell = self.cell_mut(x, y);
            if let Some(i) = cell.iter().position(|&existing| existing == value) {
                cell.swap_remove(i);
            }
            if cell.is_empty() {
                self.count -= 1;
            }
        }
    }
    pub fn occupied_count(&self) -> u64 {
        self.count
    }
    pub fn get_value(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> Option<T> {
        self.get_local_pos(global_chunk_pos, region_pos)
            .ok()
            .and_then(|(x, y)| self.cell(x, y).first().copied())
    }
    pub fn get_values(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> Option<&[T]> {
        self.get_local_pos(global_chunk_pos, region_pos)
            .ok()
            .map(|(x, y)| self.cell(x, y).as_slice())
    }
}
pub enum ChunkOccupyError {
    AlreadyOccupied,
    OutOfRegionBounds(CardinalDirection),
}

#[derive(Component, Debug, Default, Clone)]
pub struct GridOfSgcs(pub RegionGrid<Entity>);

impl GridOfSgcs {
    pub fn sampled_structure_at_gpos(&self, gpos: GlobalTilePos, region_pos: RegionPos) -> Option<Entity> {
        self.0.get_value(gpos.to_chunkpos(), region_pos)
    }
    pub fn render_grid(&self, ui: &mut egui::Ui, current_position: Option<ChunkPos>, region_pos: Option<RegionPos>) -> Option<Entity> {
        let width = REGION_SIZE_IN_CHUNKS.x() as usize;
        let height = REGION_SIZE_IN_CHUNKS.y() as usize;
        let cell_side = (ui.available_width() / width.max(1) as f32).clamp(1.0, 28.0);
        let cell_w = cell_side;
        let cell_h = cell_side;
        let grid_size = egui::vec2(width as f32 * cell_w, height as f32 * cell_h);
        let (rect, response) = ui.allocate_exact_size(grid_size, egui::Sense::click());
        let painter = ui.painter_at(rect);
        let mut clicked_entity: Option<Entity> = None;
        if response.clicked()
            && let Some(pointer_pos) = response.interact_pointer_pos()
            && rect.contains(pointer_pos)
        {
            let cell_x = ((pointer_pos.x - rect.left()) / cell_w).floor() as usize;
            let display_y = ((pointer_pos.y - rect.top()) / cell_h).floor() as usize;
            if cell_x < width && display_y < height {
                let grid_y = (height - 1) - display_y;
                if let Some(cell) = self.0.cell_opt(cell_x, grid_y)
                    && !cell.is_empty()
                {
                    clicked_entity = Some(cell[0]);
                }
            }
        }

        let local_pos = if let (Some(chunk_pos), Some(region_pos)) = (current_position, region_pos) {
            let local_chunk_pos = chunk_pos - region_pos.to_chunkpos();
            let is_in_bounds = local_chunk_pos.0.x >= 0
                && local_chunk_pos.0.x < REGION_SIZE_IN_CHUNKS.x()
                && local_chunk_pos.0.y >= 0
                && local_chunk_pos.0.y < REGION_SIZE_IN_CHUNKS.y();
            if is_in_bounds {
                Some((local_chunk_pos.0.x as usize, local_chunk_pos.0.y as usize))
            } else {
                None
            }
        } else {
            None
        };

        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 18, 18));
        let clip = ui.clip_rect().intersect(rect);
        if clip.is_positive() {
            let x_start = (((clip.left() - rect.left()) / cell_w).floor() as i32).max(0) as usize;
            let x_end = (((clip.right() - rect.left()) / cell_w).ceil() as i32).min(width as i32) as usize;
            let y_start = (((clip.top() - rect.top()) / cell_h).floor() as i32).max(0) as usize;
            let y_end = (((clip.bottom() - rect.top()) / cell_h).ceil() as i32).min(height as i32) as usize;
            for display_y in y_start..y_end {
                let grid_y = height - 1 - display_y;
                for x in x_start..x_end {
                    let cell = self.0.cell(x, grid_y);
                let cell_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + x as f32 * cell_w, rect.top() + display_y as f32 * cell_h),
                    egui::vec2(cell_w, cell_h),
                );

                let is_highlight = if let Some((local_x, local_y)) = local_pos {
                    let grid_y = (REGION_SIZE_IN_CHUNKS.y() as usize - 1) - display_y;
                    local_x == x && local_y == grid_y
                } else {
                    false
                };

                    if !cell.is_empty() {
                            let entity = cell[0];
                            let hashed = entity
                                .to_bits()
                                .wrapping_mul(0x9E37_79B9_7F4A_7C15);
                            let hue = ((hashed & 0xFFFF) as f32) / 65535.0;
                            let mut fill: egui::Color32 = egui::ecolor::Hsva::new(hue, 0.72, 0.88, 1.0).into();
                            if is_highlight {
                                fill = fill.gamma_multiply(1.18);
                            }
                            painter.rect_filled(cell_rect, 0.0, fill);
                    }

                    if is_highlight {
                        painter.rect_stroke(
                            cell_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
        }
        clicked_entity
    }
}
