use bevy::{ecs::entity::{EntityHashMap, MapEntities}, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use ::common::*;
use game_common::game_common_samplers::{CappedNormalDist, EntityWeightedSampler};
use tilemap_shared::{DimensionRef, GlobalTilePos, };
use tilemap_shared::{ChunkPos, MacroChunkPos, MACRO_CHUNK_SIZE_IN_CHUNKS};
use serde::{Deserialize, Serialize};

const PACK_CLUSTER_RADIUS_CHUNKS: i32 = 1;
const PACK_CENTER_MIN_SEPARATION_CHUNKS: i32 = 2;
const PACK_CENTER_SAMPLE_ATTEMPTS: usize = 16;

#[derive(Component, serde::Serialize, serde::Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Pack"), AssetScoped, HotReload)]
pub struct Pack;
impl Pack {
    pub fn sample_pack_center_chunk(
        macro_chunk_pos: MacroChunkPos,
        preferred_center_chunk: Option<ChunkPos>,
        allow_adjacent_preferred_center: bool,
        existing_centers: &[ChunkPos],
        rng: &mut impl rand::Rng,
    ) -> Option<ChunkPos> {
        if let Some(candidate) = preferred_center_chunk
            .filter(|candidate| macro_chunk_pos.contains_chunkpos(*candidate))
        {
            let has_required_separation = existing_centers.iter().all(|center| {
                (center.0 - candidate.0)
                    .abs()
                    .max_element()
                    >= PACK_CENTER_MIN_SEPARATION_CHUNKS
            });
            if has_required_separation {
                return Some(candidate);
            }
            if allow_adjacent_preferred_center && !existing_centers.contains(&candidate) {
                return Some(candidate);
            }
        }
        let min_chunk = macro_chunk_pos.to_chunkpos().0;
        let max_chunk_excl = min_chunk + MACRO_CHUNK_SIZE_IN_CHUNKS.0;
        for _ in 0..PACK_CENTER_SAMPLE_ATTEMPTS {
            let candidate = ChunkPos(IVec2::new(
                rng.random_range(min_chunk.x..max_chunk_excl.x),
                rng.random_range(min_chunk.y..max_chunk_excl.y),
            ));
            if existing_centers.iter().all(|center| {
                (center.0 - candidate.0)
                    .abs()
                    .max_element()
                    >= PACK_CENTER_MIN_SEPARATION_CHUNKS
            }) {
                return Some(candidate);
            }
        }
        None
    }

    pub fn sample_pack_member_chunks(
        macro_chunk_pos: MacroChunkPos,
        center_chunk: ChunkPos,
        pack_size: usize,
        rng: &mut impl rand::Rng,
    ) -> Vec<ChunkPos> {
        let mut out = Vec::with_capacity(pack_size);
        if pack_size == 0 {
            return out;
        }
        out.push(center_chunk);

        let min_chunk = macro_chunk_pos.to_chunkpos().0;
        let max_chunk = min_chunk + MACRO_CHUNK_SIZE_IN_CHUNKS.0 - IVec2::ONE;
        while out.len() < pack_size {
            let offset = IVec2::new(
                rng.random_range(-PACK_CLUSTER_RADIUS_CHUNKS..=PACK_CLUSTER_RADIUS_CHUNKS),
                rng.random_range(-PACK_CLUSTER_RADIUS_CHUNKS..=PACK_CLUSTER_RADIUS_CHUNKS),
            );
            let candidate = ChunkPos(IVec2::new(
                (center_chunk.0.x + offset.x).clamp(min_chunk.x, max_chunk.x),
                (center_chunk.0.y + offset.y).clamp(min_chunk.y, max_chunk.y),
            ));
            out.push(candidate);
        }
        out
    }
}

#[derive(Component, Debug, Clone, MapEntities, Default)]
#[component(map_entities)]
pub struct PackBeingSampler(#[entities] pub EntityWeightedSampler);

#[derive(Component, Debug, Clone, Default)]
pub struct PackMemberRankSampler(pub EntityHashMap<CappedNormalDist>);

#[derive(Component, Debug, Clone, Default)]
pub struct CenterRankMultipliers(pub EntityHashMap<f32>);

#[derive(Component, Debug, Copy, Clone)]
pub struct GlobalCenterRankWeightMultiplier(pub f32);

#[derive(Component, Debug, Clone, Default)]
pub struct PackOnAttackBehavior(pub StrId);

#[derive(Component, Debug, Copy, Clone)]
pub struct PackAttackAlertEffectivenessFalloff(pub f32);

#[derive(Component, Debug, Copy, Clone)]
pub struct PackCounterRegroupTightness(pub f32);

#[derive(Component, Debug, Clone, Default)]
pub struct PackMinDistsToPacksOrRaces(pub EntityHashMap<u8>);
impl PackMinDistsToPacksOrRaces {
    pub fn insert(&mut self, entity: Entity, min_inbetween_chunks: u8) {
        self.0.insert(entity, min_inbetween_chunks);
    }

    pub fn min_inbetween_chunks(&self, entity: Entity) -> u8 {
        self.configured_min_inbetween_chunks(entity).unwrap_or(1)
    }

    pub fn configured_min_inbetween_chunks(&self, entity: Entity) -> Option<u8> {
        self.0.get(&entity).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Component, Debug, Clone)]
pub struct PackInitialSize(pub CappedNormalDist);
impl PackInitialSize {
    pub fn sample_count(&self, rng: &mut impl rand::Rng) -> usize {
        self.0.sample(rng).round().max(1.0) as usize
    }
}

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct PackCenterPos(pub HashMap<DimensionRef, GlobalTilePos>);

/*
*/
