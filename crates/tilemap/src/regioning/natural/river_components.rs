use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use ::tilemap_shared::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct RiverProbeRequest {
    pub region_ent: Entity,
    pub sgc_ent: Entity,
    pub offer_i: u64,
    pub start_chunk: ChunkPos,
}

#[derive(Component, Debug, Clone, Default)]
pub struct RiverRegionPlan {
    pub claimed_chunks: HashSet<ChunkPos>,
    pub river_tiles: HashMap<ChunkPos, ChunkGposMask>,
    pub gravel_tiles: HashMap<ChunkPos, ChunkGposMask>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiverMouthRejectReason {
    OutsideRegion,
    NearBorder,
    WrongLandComponent,
    TooCloseToSource,
    TooInland,
    ClosedWaterBodyTooSmall,
}

#[derive(Debug, Clone, Default)]
pub struct RiverMouthRejectStats {
    pub total_rejections: u32,
    pub counts: HashMap<RiverMouthRejectReason, u32>,
}

impl RiverRegionPlan {
    pub fn river_tile_count(&self) -> usize {
        self.river_tiles.values().map(ChunkGposMask::count_set).sum()
    }

    pub fn iter_river_tiles_sorted(&self) -> Vec<GlobalTilePos> {
        self.iter_tiles_from_mask(&self.river_tiles)
    }

    pub fn iter_gravel_tiles_sorted(&self) -> Vec<GlobalTilePos> {
        self.iter_tiles_from_mask(&self.gravel_tiles)
    }

    pub fn sorted_claimed_chunks(&self) -> Option<Vec<ChunkPos>> {
        if self.claimed_chunks.is_empty() {
            return None;
        }
        let mut chunks_pos = self.claimed_chunks.iter().copied().collect::<Vec<_>>();
        chunks_pos.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
        Some(chunks_pos)
    }

    fn iter_tiles_from_mask(&self, mask_map: &HashMap<ChunkPos, ChunkGposMask>) -> Vec<GlobalTilePos> {
        let mut tiles = Vec::with_capacity(mask_map.values().map(ChunkGposMask::count_set).sum());
        let mut chunk_positions = mask_map.keys().copied().collect::<Vec<_>>();
        chunk_positions.sort_unstable_by_key(|chunk| (chunk.0.y, chunk.0.x));
        for chunk_pos in chunk_positions {
            let Some(mask) = mask_map.get(&chunk_pos) else {
                continue;
            };
            let chunk_tile_origin = chunk_pos.to_tilepos();
            for bit_idx in 0..ChunkPos::CHUNK_AREA {
                if !mask.is_set(bit_idx) {
                    continue;
                }
                let x = (bit_idx % ChunkPos::CHUNK_SIZE.x as usize) as i32;
                let y = (bit_idx / ChunkPos::CHUNK_SIZE.x as usize) as i32;
                tiles.push(GlobalTilePos(chunk_tile_origin.0 + IVec2::new(x, y)));
            }
        }
        tiles
    }
}

#[derive(Component, Debug, Clone)]
pub struct RiverPendingOffer {
    pub region_ent: Entity,
    pub sgc_ent: Entity,
    pub offer_i: u64,
    pub start_chunk: ChunkPos,
    pub inland_requester: Option<Entity>,
}

impl RiverPendingOffer {
    pub fn new(
        region_ent: Entity,
        sgc_ent: Entity,
        offer_i: u64,
        start_chunk: ChunkPos,
    ) -> Self {
        Self {
            region_ent,
            sgc_ent,
            offer_i,
            start_chunk,
            inland_requester: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RiverRegionDebugInfo {
    pub active_probe_chunks: HashSet<ChunkPos>,
    pub failed_chunks: HashSet<ChunkPos>,
    pub failed_probe_points: HashSet<GlobalTilePos>,
    pub sampled_points: HashMap<GlobalTilePos, f32>,
    pub river_source_points: HashSet<GlobalTilePos>,
    pub river_mouth_points: HashSet<GlobalTilePos>,
    pub mouth_reject_stats: RiverMouthRejectStats,
    pub success_count: u32,
    pub failure_count: u32,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct RiverDebugData {
    pub data: HashMap<(DimensionRef, RegionPos), RiverRegionDebugInfo>,
    pub revision: u64,
}
impl RiverDebugData {
    pub(crate) fn region_mut(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) -> &mut RiverRegionDebugInfo {
        self.data.entry((dimension_ref, region_pos)).or_default()
    }

    pub(crate) fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn remove_region(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) {
        if self.data.remove(&(dimension_ref, region_pos)).is_some() {
            self.bump_revision();
        }
    }

    pub(crate) fn mark_probe_started(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, start_chunk: ChunkPos) {
        self.region_mut(dimension_ref, region_pos)
            .active_probe_chunks
            .insert(start_chunk);
        self.bump_revision();
    }

    pub(crate) fn mark_probe_finished(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, start_chunk: ChunkPos) {
        let Some(info) = self.data.get_mut(&(dimension_ref, region_pos)) else {
            return;
        };
        info.active_probe_chunks.remove(&start_chunk);
        self.bump_revision();
    }
    #[allow(dead_code)]
    pub(crate) fn mark_sample(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, pos: GlobalTilePos, val: f32) {
        self.region_mut(dimension_ref, region_pos)
            .sampled_points
            .insert(pos, val);
        self.bump_revision();
    }

}

