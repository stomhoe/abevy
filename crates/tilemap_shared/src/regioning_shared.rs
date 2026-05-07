use bevy::{ecs::entity::MapEntities, platform::collections::*, prelude::*};
use bevy_inspector_egui::egui;
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::{passes_tag_filters, TagSet}};
use crate::{DimensionRef, RegionPos, regioning_messages::*};
use serde::{Deserialize, Serialize};
use crate::tilemap_shared::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SgcArgValue {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    List(Vec<SgcArgValue>),
    Map(HashMap<String, SgcArgValue>),
    Null,
}

impl Default for SgcArgValue {
    fn default() -> Self {
        Self::Null
    }
}

impl SgcArgValue {
    pub fn as_map(&self) -> Option<&HashMap<String, SgcArgValue>> {
        let Self::Map(map) = self else {
            return None;
        };
        Some(map)
    }

    pub fn as_list(&self) -> Option<&[SgcArgValue]> {
        let Self::List(list) = self else {
            return None;
        };
        Some(list.as_slice())
    }

    pub fn first(&self) -> Option<&str> {
        self.as_list()
            .and_then(|list| list.first())
            .and_then(SgcArgValue::as_str)
            .or_else(|| self.as_str())
    }

    pub fn as_str(&self) -> Option<&str> {
        let Self::Str(value) = self else {
            return None;
        };
        Some(value.as_str())
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Self::Int(value) => u8::try_from(*value).ok(),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Self::Int(value) => u16::try_from(*value).ok(),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Self::Float(value) => Some(*value as f32),
            Self::Int(value) => Some(*value as f32),
            _ => None,
        }
    }

    pub fn as_scalar_string(&self) -> Option<String> {
        match self {
            Self::Str(value) => Some(value.clone()),
            Self::Bool(value) => Some(value.to_string()),
            Self::Int(value) => Some(value.to_string()),
            Self::Float(value) => Some(value.to_string()),
            Self::Null => None,
            Self::List(_) => None,
            Self::Map(_) => None,
        }
    }
}

#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SgcArgsDict(pub HashMap<String, SgcArgValue>);

pub type ArgsDict = SgcArgsDict;

impl SgcArgsDict {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &SgcArgValue)> {
        self.0.iter()
    }

    pub fn insert<T: Into<String>>(&mut self, key: T, value: SgcArgValue) {
        self.0.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&SgcArgValue> {
        self.0.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(SgcArgValue::as_str)
    }

    pub fn get_map(&self, key: &str) -> Option<&HashMap<String, SgcArgValue>> {
        self.get(key).and_then(SgcArgValue::as_map)
    }

    pub fn parse_arg<T: std::str::FromStr + Clone>(&self, key: &str, default: T) -> T {
        self.parse_opt_arg(key).unwrap_or(default)
    }

    pub fn parse_opt_arg<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        let value = self.get(key)?;
        match value {
            SgcArgValue::Str(value) => value.parse::<T>().ok(),
            SgcArgValue::Bool(value) => value.to_string().parse::<T>().ok(),
            SgcArgValue::Int(value) => value.to_string().parse::<T>().ok(),
            SgcArgValue::Float(value) => value.to_string().parse::<T>().ok(),
            SgcArgValue::Null => None,
            SgcArgValue::List(list) => list.first().and_then(SgcArgValue::as_scalar_string).and_then(|value| value.parse::<T>().ok()),
            SgcArgValue::Map(_) => None,
        }
    }

    pub fn room_spawn_shape_keys(&self) -> HashSet<String> {
        let mut shapes = HashSet::default();
        let Some(room_spawn_map) = self.get_map("room_spawn") else {
            for key in self.0.keys() {
                let Some((_, rest)) = key.split_once("room_spawn.") else {
                    continue;
                };
                let Some((shape, _)) = rest.split_once('.') else {
                    continue;
                };
                if shape.is_empty() {
                    continue;
                }
                shapes.insert(shape.to_string());
            }
            return shapes;
        };
        for shape in room_spawn_map.keys() {
            if shape.trim().is_empty() {
                continue;
            }
            shapes.insert(shape.to_string());
        }
        shapes
    }

}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(Replicated, Prefix::trunc("StructureGenerationSettings"), AssetScoped, SelectedForHotReload)]
pub struct StructureGenerationSettings {
    pub structure_build_timeout_secs: f64,
    pub claimlist_advance_timeout_secs: f32,
    pub region_offer_timeout_secs: f32,
    pub max_used_chunks_per_region_ratio: f32,
}

impl Default for StructureGenerationSettings {
    fn default() -> Self {
        Self {
            structure_build_timeout_secs: 4.0,
            claimlist_advance_timeout_secs: 0.1,
            region_offer_timeout_secs: 2.0,
            max_used_chunks_per_region_ratio: 0.07,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(AssetScoped, Prefix::trunc("SGC"))]
pub struct StructuredGenConfig {
    structure_id: StrId,
    structure_hash_id: HashId,
    pub max_per_region: u32,
    pub max_being_count: Option<u32>,
    pub args: SgcArgsDict,
    pub typed_args: SgcArgsDict,
    pub whitelisted_tags: TagSet,
    pub blacklisted_tags: TagSet,
}

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Region;

impl StructuredGenConfig {
    pub fn new<S: AsRef<str>>(structure_id: S) -> Self {
        Self {
            structure_id: StrId::trunc(structure_id.as_ref()),
            structure_hash_id: HashId::hash(structure_id.as_ref()),
            max_per_region: 1024,
            max_being_count: None,
            args: SgcArgsDict::default(),
            typed_args: SgcArgsDict::default(),
            whitelisted_tags: TagSet::default(),
            blacklisted_tags: TagSet::default(),
        }
    }

    pub fn structure_id(&self) -> &StrId {
        &self.structure_id
    }

    pub fn structure_hash_id(&self) -> HashId {
        self.structure_hash_id
    }

    pub fn tolerates_tags(&self, other_tags: &TagSet) -> bool {
        passes_tag_filters(Some(other_tags), Some(&self.whitelisted_tags), Some(&self.blacklisted_tags))
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
#[relationship(relationship_target = AcceptedFilters)]
pub struct WhitelistedFilterOf {
    #[relationship]
    #[entities]
    pub structured_gen_cfg: Entity,
}

impl WhitelistedFilterOf {
    pub fn new(structured_gen_cfg: Entity) -> Self {
        Self { structured_gen_cfg }
    }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = WhitelistedFilterOf)]
pub struct AcceptedFilters(Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(AssetScoped, Prefix::trunc("SGCsWeightedSampler"))]
pub struct SgcsWeightedSampler;

#[derive(Debug, Clone, Default)]
pub struct SgcCommandSchema {
    pub room_spawn_shapes: HashSet<String>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SgcCommandRegistry(pub HashMap<String, SgcCommandSchema>);

impl SgcCommandRegistry {
    pub fn with_builtins() -> Self {
        let mut registry = Self::default();
        registry.register_room_spawn_shapes("chamberscorridors", ["rectangle", "circle", "triangle", "regular_polygon", "pentacle"]);
        registry.register_room_spawn_shapes("maze", ["square_room", "circle_room", "island_circle", "island_triangle", "island_hexagon", "island_square"]);
        registry.register_room_spawn_shapes("drunkwalk", ["chamber_circle"]);
        registry.register_room_spawn_shapes("spiral", ["center_circle", "arm_inner", "arm_outer"]);
        registry.register_room_spawn_shapes("archi", ["center_spiral"]);
        registry
    }

    pub fn register_room_spawn_shapes<S, I>(&mut self, structure_id: &str, room_shapes: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let schema = self.0.entry(structure_id.to_string()).or_default();
        for room_shape in room_shapes {
            schema.room_spawn_shapes.insert(room_shape.as_ref().to_string());
        }
    }

    pub fn allowed_room_spawn_shapes_for(&self, structure_id: &str) -> Option<&HashSet<String>> {
        self.0.get(structure_id).map(|schema| &schema.room_spawn_shapes)
    }
}

#[derive(Resource, Default)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct PrioritizedSgs(pub Vec<HashId>);

#[derive(Resource, Default)]
pub struct PrioritizedPerRegion(pub HashMap<(DimensionRef, RegionPos), Vec<HashId>>);




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
    pub pending_is: HashSet<usize>,
    pub advance_timer: Timer,
}
impl ClaimList {
    pub fn advance_processed_upto_i(&mut self) {
        self.pending_is.remove(&self.processed_up_to_i);
        self.processed_up_to_i += 1;
        self.advance_timer.reset();
    }

    pub fn mark_pending_i(&mut self, i: usize) {
        self.pending_is.insert(i);
        self.advance_timer.reset();
    }

    pub fn clear_pending_i(&mut self, i: usize) {
        self.pending_is.remove(&i);
    }

    pub fn waiting_for_current_i(&self) -> bool {
        self.pending_is.contains(&self.processed_up_to_i)
    }

    pub fn reached_end(&self) -> bool {
        self.processed_up_to_i >= MAX_CHUNK_CLAIMS_PER_REGION
    }
}
impl Default for ClaimList {
    fn default() -> Self {
        let mut claims = Vec::with_capacity(MAX_CHUNK_CLAIMS_PER_REGION);
        claims.resize(MAX_CHUNK_CLAIMS_PER_REGION, None);
        Self {
            claims,
            processed_up_to_i: 0,
            skipped_is: HashSet::new(),
            pending_is: HashSet::new(),
            advance_timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct CountsOfSgcs (pub EntityHashMap<u32>,);

pub type TilesFromBuilder = Vec<(GlobalTilePos, HashId, Option<DeleteOtherTilesInSamePos>)>;

#[derive(Debug, Clone)]
pub struct PendingBuildOrder {
    pub chunks: Vec<ChunkPos>,
    pub timer: Timer,
}

#[derive(Component, Debug, Default, Clone)]
pub struct RegionPlannedTiles {
    tiles_to_spawn_on_chunk_load_map: HashMap<ChunkPos, TilesFromBuilder>,
    terrgen_disabled_gpos_on_chunk_load: HashMap<ChunkPos, ChunkGposMask>,
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
        chunk_tiles: TilesFromBuilder,
        terrgen_disabled_gpos_for_chunks: TerrGenDisabledGposForChunks,
    ) -> Result<bool, BevyError> {
        let Some(order) = self.pending_build_orders.remove(&order_i) else {
            return Err(BevyError::from(format!(
                "Build order {} is not pending",
                order_i
            )));
        };
        let mut chunk_tiles_by_chunk: HashMap<ChunkPos, TilesFromBuilder> = HashMap::new();
        let selected_chunks: HashSet<ChunkPos> = order.chunks.iter().copied().collect();
        for (tile_pos, tile_ref, delete_others) in chunk_tiles {
            let chunk_pos = tile_pos.to_chunkpos();
            if !selected_chunks.contains(&chunk_pos) {
                return Err(BevyError::from(format!(
                    "Tile position {:?} maps to ChunkPos {:?} which is not part of build order {}",
                    tile_pos, chunk_pos, order_i
                )));
            }
            chunk_pos.is_tilepos_within_chunk(tile_pos)?;
            chunk_tiles_by_chunk.entry(chunk_pos).or_insert_with(Vec::new).push((tile_pos, tile_ref, delete_others));
        }
        let mut dropped_out_of_bounds = 0usize;
        for (chunk_pos, blocked_gpos) in terrgen_disabled_gpos_for_chunks.0 {
            if !selected_chunks.contains(&chunk_pos) {
                dropped_out_of_bounds += blocked_gpos.count_set();
                continue;
            }
            self.terrgen_disabled_gpos_on_chunk_load.insert(chunk_pos, blocked_gpos);
        }
        if dropped_out_of_bounds > 0 {
            debug!(
                target: common::log_targets::REGION_SYSTEM,
                "Dropped {} terrgen-disabled gpos outside selected chunk bounds for build order {}",
                dropped_out_of_bounds,
                order_i
            );
        }
        for chunk_pos in order.chunks {
            let chunk_tiles = chunk_tiles_by_chunk.remove(&chunk_pos).unwrap_or_default();
            self.tiles_to_spawn_on_chunk_load_map.entry(chunk_pos).or_insert_with(Vec::new).extend(chunk_tiles);
            self.pending_chunks.remove(&chunk_pos);
        }
        Ok(self.pending_build_orders.is_empty())
    }

    pub fn get(&self, chunk_pos: &ChunkPos,) -> Option<&TilesFromBuilder> {
        self.tiles_to_spawn_on_chunk_load_map.get(chunk_pos)
    }

    pub fn take_terrgen_disabled_gpos(&mut self, chunk_pos: ChunkPos) -> ChunkGposMask {
        self.terrgen_disabled_gpos_on_chunk_load.remove(&chunk_pos).unwrap_or_default()
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

pub const MAX_CHUNK_CLAIMS_PER_REGION: usize = REGION_SIZE_IN_CHUNKS.area_usize();

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


common::define_entity_map_systems!(
    main_component: StructuredGenConfig,
    with_filters: (),
    abbreviation: Sgc,
    target: "sgc",
    entity_prefix: "SGC",
    despawn_trigger: StructuredGenConfig,
    id_type: common::common_components::StrId,
);
