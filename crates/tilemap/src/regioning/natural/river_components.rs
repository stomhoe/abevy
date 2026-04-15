use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos, RegionPos};



#[derive(Debug, Clone, Default)]
pub struct RiverRegionDebugInfo {
    pub active_probe_chunks: HashSet<ChunkPos>,
    pub claimed_chunks: HashSet<ChunkPos>,
    pub failed_chunks: HashSet<ChunkPos>,
    pub river_tiles: HashSet<GlobalTilePos>,
    pub river_source_points: HashSet<GlobalTilePos>,
    pub river_mouth_points: HashSet<GlobalTilePos>,
    pub sampled_points: HashMap<GlobalTilePos, f32>,
    pub sampled_none_points: HashSet<GlobalTilePos>,
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

    pub(crate) fn clear_generated_river(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) {
        let Some(info) = self.data.get_mut(&(dimension_ref, region_pos)) else {
            return;
        };
        info.river_tiles.clear();
        info.river_source_points.clear();
        info.river_mouth_points.clear();
        self.bump_revision();
    }
}

