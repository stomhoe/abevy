use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos, RegionPos};

#[derive(Component, Debug, Clone, Copy)]
pub struct RiverProbeRequest {
    pub region_ent: Entity,
    pub dimension_ref: DimensionRef,
    pub sgc_ent: Entity,
    pub offer_i: u64,
    pub region_pos: RegionPos,
    pub start_chunk: ChunkPos,
}

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
pub struct RiverDebugData(pub HashMap<(DimensionRef, RegionPos), RiverRegionDebugInfo>);
impl RiverDebugData {
    pub(crate) fn region_mut(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) -> &mut RiverRegionDebugInfo {
        self.0.entry((dimension_ref, region_pos)).or_default()
    }

    pub fn remove_region(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) {
        self.0.remove(&(dimension_ref, region_pos));
    }

    pub(crate) fn mark_probe_started(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, start_chunk: ChunkPos) {
        self.region_mut(dimension_ref, region_pos)
            .active_probe_chunks
            .insert(start_chunk);
    }

    pub(crate) fn mark_probe_finished(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, start_chunk: ChunkPos) {
        let Some(info) = self.0.get_mut(&(dimension_ref, region_pos)) else {
            return;
        };
        info.active_probe_chunks.remove(&start_chunk);
    }
    #[allow(dead_code)]
    pub(crate) fn mark_sample(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, pos: GlobalTilePos, val: f32) {
        self.region_mut(dimension_ref, region_pos)
            .sampled_points
            .insert(pos, val);
    }

    pub(crate) fn clear_generated_river(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) {
        let Some(info) = self.0.get_mut(&(dimension_ref, region_pos)) else {
            return;
        };
        info.river_tiles.clear();
        info.river_source_points.clear();
        info.river_mouth_points.clear();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RiverRegisteredOffer {
    pub region_ent: Entity,
    pub sgc_ent: Entity,
    pub offer_i: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RiverRegionPlan {
    pub claimed_chunks: HashSet<ChunkPos>,
    pub river_tiles: HashSet<GlobalTilePos>,
    pub river_source_points: HashSet<GlobalTilePos>,
    pub river_mouth_points: HashSet<GlobalTilePos>,
}

#[derive(Resource, Debug, Default)]
pub struct RiverPlans {
    pub registered_offers: HashMap<(DimensionRef, RegionPos), RiverRegisteredOffer>,
    pub plans_by_region: HashMap<(DimensionRef, RegionPos), RiverRegionPlan>,
}

impl RiverPlans {
    pub fn plan(
        &self,
        dimension_ref: DimensionRef,
        region_pos: RegionPos,
    ) -> Option<&RiverRegionPlan> {
        self.plans_by_region.get(&(dimension_ref, region_pos))
    }

    pub fn register_offer(
        &mut self,
        dimension_ref: DimensionRef,
        region_pos: RegionPos,
        offer: RiverRegisteredOffer,
    ) -> Option<RiverRegisteredOffer> {
        self.registered_offers
            .insert((dimension_ref, region_pos), offer)
    }

    pub fn registered_offer(
        &self,
        dimension_ref: DimensionRef,
        region_pos: RegionPos,
    ) -> Option<RiverRegisteredOffer> {
        self.registered_offers
            .get(&(dimension_ref, region_pos))
            .copied()
    }

    pub fn remove_region(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) {
        self.registered_offers.remove(&(dimension_ref, region_pos));
        self.plans_by_region.remove(&(dimension_ref, region_pos));
    }
}
