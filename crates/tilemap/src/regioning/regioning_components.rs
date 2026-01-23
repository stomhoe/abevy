#[allow(unused_imports)] use bevy::prelude::*;
use game_common::game_common_components::{Direction, EntityZeroRef};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::{EntityHashMap, EntityHashSet, MapEntities}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet, hash_map::Entry}, prelude::*};
use tilemap_shared::{ChunkPos, GlobalTilePos, HashablePosVec, REGION_SIZE_IN_CHUNKS, RegionPos};

use crate::{chunking_components::Chunk, regioning::regioning_messages::ClaimedChunks, tile::tile_components::*};
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;


use common::{common_components::*, };

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(SessionScoped, TgenHotLoadingScoped, ClaimList, RegionPlannedTiles, Visibility, Transform )]
pub struct Region;

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = Chunk)]
pub struct ChunksActiveInRegion(Vec<Entity>);
impl ChunksActiveInRegion { pub fn entities(&self) -> &[Entity] { &self.0 } }






#[derive(Component, Debug, Reflect)]
#[require(CountsOfSgcs, GridOfSgcs)]
pub struct ClaimList { 
    pub processed_up_to_i: usize,
    pub claims: [Option<ClaimedChunks>; MAX_CLAIMS],
}

#[derive(Component, Debug, Reflect, Default)]
pub struct CountsOfSgcs (pub EntityHashMap<u32>,);

impl Default for ClaimList {
    fn default() -> Self {
        Self { 
            claims: [(); MAX_CLAIMS].map(|_| None),
            processed_up_to_i: 0,
        }
    }
}

pub type TilesFromBuilder = Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOtherTiles>)>;

#[derive(Component, Debug, )]
//a region's component, doesn't need dimension
pub struct RegionPlannedTiles { 
    tiles_to_spawn_on_chunk_load_map: HashMap<ChunkPos, TilesFromBuilder>,
    // store pending chunks along with the time (seconds since startup) when they were added
    chunks_pending_build: HashMap<ChunkPos, f64>,
}
impl Default for RegionPlannedTiles {
    fn default() -> Self {
        Self { tiles_to_spawn_on_chunk_load_map: HashMap::new(), chunks_pending_build: HashMap::new() }
    }
}
impl RegionPlannedTiles {
    pub fn new(chunk_positions: &[ChunkPos], now: f64) -> Self {
        Self {
            tiles_to_spawn_on_chunk_load_map: HashMap::new(),
            chunks_pending_build: chunk_positions.iter().copied().map(|p| (p, now)).collect(),
        }
    }
    
    pub fn add_chunks_pending_build(&mut self, chunk_positions: &[ChunkPos], now: f64) {
        for &pos in chunk_positions {
            self.chunks_pending_build.insert(pos, now);
        }
    }
    
    pub fn is_chunk_pending_build(&self, chunk_pos: ChunkPos) -> bool {
        self.chunks_pending_build.contains_key(&chunk_pos)
    }
    pub fn pending_chunks_count(&self) -> usize {
        self.chunks_pending_build.len()
    }
    
    pub fn add_planned_tiles_and_remove_from_pending(
        &mut self,
        chunk_pos: ChunkPos,
        tile_data: TilesFromBuilder,
    ) -> Result<bool, BevyError> {
        if !self.chunks_pending_build.contains_key(&chunk_pos) {
            return Err(BevyError::from(format!(
                "ChunkPos {:?} is not pending a build order",
                chunk_pos
            )));
        }
        
        for (tile_pos, _, _) in &tile_data {
            chunk_pos.is_tilepos_within_chunk(*tile_pos)?;
        }
        
        self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new).extend(tile_data);
        self.chunks_pending_build.remove(&chunk_pos);
        Ok(self.chunks_pending_build.is_empty())
    }
    
    pub fn get(
        &self,
        chunk_pos: &ChunkPos,
    ) -> Option<&TilesFromBuilder> {
        self.tiles_to_spawn_on_chunk_load_map.get(chunk_pos)
    }
    
    pub fn pending_chunks_iter(&self) -> impl Iterator<Item = (&ChunkPos, &f64)> {
        self.chunks_pending_build.iter()
    }
    
    pub fn remove_pending_chunk(&mut self, chunk_pos: &ChunkPos) -> bool {
        self.chunks_pending_build.remove(chunk_pos).is_some()
    }
    
    pub fn mark_chunk_timed_out(&mut self, chunk_pos: ChunkPos) {
        // remove from pending set and make an empty entry in the map so other systems see there are no tiles
        self.chunks_pending_build.remove(&chunk_pos);
        self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new);
    }
    
}

pub const MAX_CLAIMS: usize = 1024;


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct AllTilesPrepared;


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BuildingStarted;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct PendingOfferTimeout {
    pub offered_at: f64,
}
#[derive(Debug, Reflect, )]
pub struct RegionGrid<T: Copy> { grid: [[Option<T>; REGION_SIZE_IN_CHUNKS.0.x as usize]; REGION_SIZE_IN_CHUNKS.0.y as usize], count: u32, }
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
            (true, _, _, _) => Err(ChunkOccupyError::OutOfRegionBounds(Direction::West)),
            (_, true, _, _) => Err(ChunkOccupyError::OutOfRegionBounds(Direction::East)),
            (_, _, true, _) => Err(ChunkOccupyError::OutOfRegionBounds(Direction::South)),
            (_, _, _, true) => Err(ChunkOccupyError::OutOfRegionBounds(Direction::North)),
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
    
    pub fn occupied_count(&self) -> u32 {
        self.count
    }
}
pub enum ChunkOccupyError<T> {
    AlreadyOccupied(T),
    OutOfRegionBounds(Direction),
}

#[derive(Component, Debug, Reflect, Default)]
pub struct GridOfSgcs(pub RegionGrid<Entity>);

impl GridOfSgcs {
    fn render_grid(&self, ui: &mut egui::Ui) {
        egui::Grid::new(ui.id().with("grid_of_sgcs"))
        .spacing([0.0, 0.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            let prev_item_spacing = ui.spacing_mut().item_spacing;
            ui.spacing_mut().item_spacing.x = 0.0;
            
            let base = ui.text_style_height(&egui::TextStyle::Monospace);
            let cell_size = egui::vec2(base * 1.25, base * 1.25);
            
            const ARRAY_OF_LEGIBLE_CHARS: &[char] = &[
            '0','1','2','3','4','5','6','7','8','9',
            'A','B','C','D','E','F','G','H','I','J','K','L','M',
            'N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
            '!','@','#','$','%','^','&','*','(',')','-','+','=','/','|','~','<','>','?',':',';',
            '░','█','▲','△','▶','▷','▼','▽','◀','◁','◢','◣','◤','◥',
            '◆','◇','⬟','⬢','⬣',
            '■','□','▪','▫',
            '✦','✧','✪','✫','✬','✭','✮','✯',
            '✖','✚','✛','✜','✢','✣','✤','✥',
            '±','×','÷','≈','≠','≤','≥','∞','∑','∏','√','∆','∇','∫',
            '⬅','➡','⬆','⬇','↩','↪','⇐','⇒','⇑','⇓',
            'α','β','γ','δ','ε','ζ','η','θ','λ','μ','π','ω',
            'Φ','Ψ','Ω','Σ','Π',
            '∂','∈','∉','∩','∪','∀','∃',
            '†','‡','°','‰','§','¶','¤','¬','¦',
            '·',
            ];
            for row in self.0.grid.iter().rev() {
                for cell in row {
                    let symbol = match cell {
                        Some(entity) => {
                            let mut hasher = DefaultHasher::new();
                            entity.hash(&mut hasher);
                            let digit = (hasher.finish() % ARRAY_OF_LEGIBLE_CHARS.len() as u64) as usize;
                            ARRAY_OF_LEGIBLE_CHARS[digit].to_string()
                        }
                        None => "·".to_string(),
                    };
                    ui.add_sized(cell_size, egui::Button::new(egui::RichText::new(symbol).monospace()).frame(false));
                }
                ui.end_row();
            }
            
            ui.spacing_mut().item_spacing = prev_item_spacing;
        });
    }
}
impl InspectorPrimitive for GridOfSgcs {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        self.render_grid(ui);
        false
    }
    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        self.render_grid(ui);
    }
}