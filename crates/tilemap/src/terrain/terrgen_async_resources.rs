use bevy::{prelude::*, tasks::Task};
use bitvec::prelude::*;
use common::common_components::HashId;

use crate::{
    chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk,
    terrain::{
        terrprobe::terrprobe_messages::{SampledValuesCollected, SuitablePosFound, TerrProbeJob},
        terrgen_messages::PendingOp,
        terrgen_resources::TerrGenDebugSample,
    },
};

use ::tilemap_shared::*;

#[derive(Debug, Clone, )]
pub struct TerrGenBlockedGposMask(pub BitArr!(for ChunkPos::CHUNK_AREA));
impl TerrGenBlockedGposMask {
    pub fn is_empty(&self) -> bool {
        self.0.as_bitslice().count_ones() == 0
    }
    pub fn count_blocked(&self) -> usize {
        self.0.as_bitslice().count_ones()
    }
    pub fn is_blocked(&self, bit_idx: usize) -> bool {
        self.0.as_bitslice().get(bit_idx).map_or(false, |bit| *bit)
    }
    pub fn set_blocked(&mut self, bit_idx: usize) {
        self.0.as_mut_bitslice().set(bit_idx, true);
    }
    pub fn set_blocked_gpos(&mut self, chunk_pos: ChunkPos, gpos: GlobalTilePos) {
        let Some(bit_idx) = chunk_pos.bit_index_in_chunk(gpos) else {
            return;
        };
        self.set_blocked(bit_idx);
    }
}
impl Default for TerrGenBlockedGposMask {
    fn default() -> Self {
        Self(BitArray::ZERO)
    }
}

#[derive(Debug, Clone)]
pub struct TerrGenLaunchWork {
    pub chunk_ent: Entity,
    pub chunk_pos: ChunkPos,
    pub dim_ref: DimensionRef,
    pub root_oplist: DimensionRootOplist,
    pub oplist_size: OplistSize,
    pub blocked_gpos: TerrGenBlockedGposMask,
}

#[derive(Resource, Debug, Default)]
pub struct TerrGenLaunchQueue(pub Vec<TerrGenLaunchWork>);

#[derive(Debug, Clone)]
pub struct TerrGenTileRequest {
    pub bif_tiles: Vec<Entity>,
    pub pending: PendingOp,
    pub oplist_size: OplistSize,
    pub dimension_hash: HashId,
}

#[derive(Debug, Clone)]
pub struct TerrGenBiomeTagSample {
    pub macro_chunk_ent: Entity,
    pub sample_chunk_pos: ChunkPos,
    pub biome_tags: Vec<BiomeTagWeightAtMacrochunk>,
}

#[derive(Debug, Default)]
pub struct TerrGenOpTaskResult {
    pub new_pending_ops: Vec<PendingOp>,
    pub completed_chunk_gpos: Vec<(Entity, GlobalTilePos)>,
    pub completed_macro_chunk_biome_samples: Vec<Entity>,
    pub sampled_value_events: Vec<SuitablePosFound>,
    pub sampled_value_matrix_events: Vec<SampledValuesCollected>,
    pub tile_requests: Vec<TerrGenTileRequest>,
    pub biome_tag_samples: Vec<TerrGenBiomeTagSample>,
    pub debug_samples: Vec<TerrGenDebugSample>,
}

#[derive(Debug, Default)]
pub struct TerrGenSearchTaskResult {
    pub new_pending_ops: Vec<PendingOp>,
    pub new_pos_searches: Vec<TerrProbeJob>,
    pub search_failed: Vec<Entity>,
}

#[derive(Resource, Debug, Default)]
pub struct TerrGenAsyncTasks {
    pub launch_tasks: Vec<Task<Vec<PendingOp>>>,
    pub op_tasks: Vec<Task<TerrGenOpTaskResult>>,
    pub search_tasks: Vec<Task<TerrGenSearchTaskResult>>,
}
