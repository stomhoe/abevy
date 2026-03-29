#[allow(unused_imports)] use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use game_common::game_common_components::TemplEntiRef;
use tilemap_shared::DimensionRef;
use tilemap_shared::{ChunkPos, GlobalTilePos, RegionPos};
use crate::chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk;

use ::tilemap_shared::DeleteOtherTilesInSamePos;
use crate::terrain::terrgen_async_resources::TerrGenBlockedGposMask;



#[derive(Message, Debug, Clone, )]
pub struct OfferChunk {
    // for determinism
    pub i: u64,
    pub region_ent: Entity,// extraer region pos de una query
    pub structured_gen_cfg_ent: Entity,
    pub start_pos: ChunkPos,
}

#[derive(Message, Debug, Clone, Hash, PartialEq, Eq, )]
pub struct ChunksClaim {
    pub i: u64,
    pub region_ent: Entity,
    pub sgc_ent: Entity,
    //TODO: chequear que cada chunkpos no se salga de su respectiva region
    pub chunks_pos: Vec<ChunkPos>,
    pub partition_tolerant: bool,
}
impl Default for ChunksClaim {
    fn default() -> Self {
        Self { i: 0, region_ent: Entity::PLACEHOLDER, sgc_ent: Entity::PLACEHOLDER, chunks_pos: Vec::new(), partition_tolerant: false }
    }
}


#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SgcPrepareTilesOrder {
    pub i: u64,
    pub structured_gen_cfg_ent: Entity,
    pub region_pos: RegionPos,
    pub dimension_ref: DimensionRef,
    //sucessfully claimed global chunk positions
    pub chunks_pos: Vec<ChunkPos>,
}


pub type StructureTilesForChunk = Vec<(GlobalTilePos, TemplEntiRef, Option<DeleteOtherTilesInSamePos>)>;

#[derive(Debug, Default, Clone)]
pub struct TerrGenDisabledGposForChunks(pub HashMap<ChunkPos, TerrGenBlockedGposMask>);

impl TerrGenDisabledGposForChunks {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn count_blocked(&self) -> usize {
        self.0.values().map(TerrGenBlockedGposMask::count_blocked).sum()
    }

    pub fn insert_for_chunk(&mut self, chunk_pos: ChunkPos, blocked_gpos: TerrGenBlockedGposMask) {
        if blocked_gpos.is_empty() {
            return;
        }
        self.0.insert(chunk_pos, blocked_gpos);
    }

    pub fn first_chunk_pos(&self) -> Option<ChunkPos> {
        self.0.keys().next().copied()
    }

    pub fn take_for_chunk(&mut self, chunk_pos: ChunkPos) -> TerrGenBlockedGposMask {
        self.0.remove(&chunk_pos).unwrap_or_default()
    }
}

#[derive(Message, Debug, )]
pub struct StructureBuildCompliance {
    pub i: u64,
    pub structure_gen_cfg_ent: Entity,
    pub dimension_ref: DimensionRef,
    pub chunks: Vec<(ChunkPos, StructureTilesForChunk)>,
    pub terrgen_disabled_gpos_for_chunks: TerrGenDisabledGposForChunks,
    pub terrgen_disabled_for_chunks: Vec<ChunkPos>,
    pub forced_chunk_biomes: Vec<ForcedChunkBiomeConfig>,
}

#[derive(Debug, Clone, )]
pub struct ForcedChunkBiomeConfig {
    pub chunk_pos: ChunkPos,
    pub biome_tags: Vec<BiomeTagWeightAtMacrochunk>,
}


#[derive(Message, Debug, )]
pub struct RecheckRegion(pub Entity);
