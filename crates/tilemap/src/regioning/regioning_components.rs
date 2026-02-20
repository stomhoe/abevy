use game_common::game_common_components::{ EntityZeroRef};

use bevy::{ecs::entity::EntityHashMap, platform::collections::{HashMap, HashSet}, prelude::*};
use ::tilemap_shared::*;

use crate::{chunking::chunking_components::Chunk, regioning::regioning_messages::ChunksClaim, tile::tile_components::*};
use bevy_inspector_egui::{egui, };

use common::{common_components::*, };
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(ClaimList, RegionPlannedTiles, Visibility, Transform, AssetScoped)]
pub struct Region;

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = Chunk)]
pub struct ChunksActiveInRegion(Vec<Entity>);
impl ChunksActiveInRegion { pub fn entities(&self) -> &[Entity] { &self.0 } }

#[derive(Component, Debug, Clone)]
#[require(CountsOfSgcs, GridOfSgcs)]
pub struct ClaimList {
    pub processed_up_to_i: usize,
    pub claims: [Option<ChunksClaim>; MAX_CLAIMS],
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
        Self {
            claims: [(); MAX_CLAIMS].map(|_| None),
            processed_up_to_i: 0,
            skipped_is: HashSet::new(),
            advance_timer: Timer::from_seconds(0.02, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct CountsOfSgcs (pub EntityHashMap<u32>,);

pub type TilesFromBuilder = Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOtherTiles>)>;

#[derive(Debug, Clone)]
pub struct PendingBuildOrder {
    pub chunks: Vec<ChunkPos>,
    pub timer: Timer,
}

#[derive(Component, Debug, Default, Clone)]
pub struct RegionPlannedTiles {
    tiles_to_spawn_on_chunk_load_map: HashMap<ChunkPos, TilesFromBuilder>,
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
        for chunk_pos in order.chunks {
            if !provided_chunks.contains(&chunk_pos) {
                self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new);
            }
            self.pending_chunks.remove(&chunk_pos);
        }
        Ok(self.pending_build_orders.is_empty())
    }

    pub fn get(&self, chunk_pos: &ChunkPos,) -> Option<&TilesFromBuilder> {
        self.tiles_to_spawn_on_chunk_load_map.get(chunk_pos)
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
    }

}

pub const MAX_CLAIMS: usize = REGION_SIZE_IN_CHUNKS.area_usize();

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct AllTilesPrepared;

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct BuildingStarted;


#[derive(Debug, )]
pub struct RegionGrid<T: Copy> { grid: [[Option<T>; REGION_SIZE_IN_CHUNKS.0.x as usize]; REGION_SIZE_IN_CHUNKS.0.y as usize], count: u64, }
impl<T: Copy> Default for RegionGrid<T> {
    fn default() -> Self {
        Self {
            grid: [[const { None }; REGION_SIZE_IN_CHUNKS.0.x as usize]; REGION_SIZE_IN_CHUNKS.0.y as usize],
            count: 0,
        }
    }
}

impl<T: Copy> RegionGrid<T> {
    #[inline]
    fn get_local_pos(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> Result<(usize, usize), ChunkOccupyError<T>> {
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
        .map(|(x, y)| self.grid[y][x].is_some())
        .unwrap_or(false)
    }
    pub fn is_available(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> bool {
        !self.is_occupied(global_chunk_pos, region_pos)
    }
    pub fn occupy(&mut self, global_chunk_pos: ChunkPos, region_pos: RegionPos, value: T) -> Result<(), ChunkOccupyError<T>> {
        let (x, y) = self.get_local_pos(global_chunk_pos, region_pos)?;

        match self.grid[y][x] {
            None => {
                self.grid[y][x] = Some(value);
                self.count += 1;
                Ok(())
            }
            Some(existing) => Err(ChunkOccupyError::AlreadyOccupied(existing)),
        }
    }
    pub fn free(&mut self, global_chunk_pos: ChunkPos, region_pos: RegionPos) {
        if let Ok((x, y)) = self.get_local_pos(global_chunk_pos, region_pos) {
            if self.grid[y][x].is_some() {
                self.grid[y][x] = None;
                self.count -= 1;
            }
        }
    }
    pub fn occupied_count(&self) -> u64 {
        self.count
    }
}
pub enum ChunkOccupyError<T> {
    AlreadyOccupied(T),
    OutOfRegionBounds(CardinalDirection),
}

#[derive(Component, Debug, Default)]
pub struct GridOfSgcs(pub RegionGrid<Entity>);

impl GridOfSgcs {
    pub fn render_grid(&self, ui: &mut egui::Ui, current_position: Option<ChunkPos>, region_pos: Option<RegionPos>) {
        let base = ui.text_style_height(&egui::TextStyle::Monospace);
        let cell_w = (base * 0.9).max(9.0);
        let cell_h = (base * 0.9).max(9.0);
        let width = REGION_SIZE_IN_CHUNKS.x() as usize;
        let height = REGION_SIZE_IN_CHUNKS.y() as usize;
        let grid_size = egui::vec2(width as f32 * cell_w, height as f32 * cell_h);
        let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
        let painter = ui.painter_at(rect);

        const ARRAY_OF_LEGIBLE_CHARS: &[char] = &[
            '0','1','2','3','4','5','6','7','8','9',
            'A','B','C','D','E','F','G','H','I','J','K','L','M',
            'N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
            '!','@','#','$','%','^','&','*','(',')','-','+','=','/','|','~','<','>','?',':',';',
            '░','█','▲','△','▶','▷','▼','▽','◀','◁','◢','◣','◤','◥',
            '◆','◇','⬟','⬢','⬣',
            '■','□','▪','▫',
            '✦','✧','✪','✫','✬','✭','✮','✯',
            '✖','✚','✛','✤','✥',
            '±','×','÷','≈','≠','≤','≥','∞','∑','∏','√','∆','∇','∫',
            '⬅','➡','⬆','⬇','↩','↪','⇐','⇒','⇑','⇓',
            'α','β','γ','δ','ε','ζ','η','θ','λ','μ','π','ω',
            'Φ','Ψ','Ω','Σ','Π',
            '∂','∈','∉','∩','∪','∀','∃',
            '†','‡','°','‰','§','¶','¤','¬','¦',
            '·',
        ];
        let mut entity_to_char: EntityHashMap<char> = EntityHashMap::default();
        let mut char_index = 0;

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

        for (display_y, row) in self.0.grid.iter().rev().enumerate() {
            for (x, cell) in row.iter().enumerate() {
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

                match cell {
                    Some(entity) => {
                        let symbol = *entity_to_char.entry(*entity).or_insert_with(|| {
                            let ch = ARRAY_OF_LEGIBLE_CHARS[char_index % ARRAY_OF_LEGIBLE_CHARS.len()];
                            char_index += 1;
                            ch
                        });

                        let mut h: usize = entity.to_bits() as usize;
                        h = h.wrapping_mul(2654435761usize);
                        let r = 70 + (h & 0x3F) as u8;
                        let g = 90 + ((h >> 6) & 0x5F) as u8;
                        let b = 110 + ((h >> 12) & 0x6F) as u8;
                        let mut fill = egui::Color32::from_rgb(r, g, b).gamma_multiply(0.35);
                        if is_highlight {
                            fill = fill.gamma_multiply(1.35);
                        }
                        painter.rect_filled(cell_rect, 0.0, fill);
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            symbol.to_string(),
                            egui::FontId::proportional((base * 0.9).max(9.0)),
                            egui::Color32::WHITE,
                        );
                    }
                    None => {
                        painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgb(18, 18, 18));
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "·",
                            egui::FontId::proportional((base * 0.85).max(8.0)),
                            egui::Color32::from_gray(90),
                        );
                    }
                }

                let stroke = if is_highlight {
                    egui::Stroke::new(2.0, egui::Color32::YELLOW)
                } else {
                    egui::Stroke::new(0.5, egui::Color32::from_gray(45))
                };
                painter.rect_stroke(cell_rect, 0.0, stroke, egui::StrokeKind::Inside);
            }
        }
    }
}

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct AllClaimsProcessed;
