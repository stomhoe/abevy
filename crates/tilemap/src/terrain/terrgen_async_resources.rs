use bevy::{prelude::*, tasks::Task};
use common::common_components::HashId;

use crate::terrain::{
    terrprobe::terrprobe_messages::{SampledValuesCollected, SuitablePosFound, TerrProbeJob},
    terrgen_messages::PendingOp,
    terrgen_resources::TerrGenDebugSample,
};

use ::tilemap_shared::*;

#[derive(Debug, Clone)]
pub struct TerrGenLaunchWork {
    pub chunk_ent: Entity,
    pub chunk_pos: ChunkPos,
    pub dim_ref: DimensionRef,
    pub root_oplist: DimensionRootOplist,
    pub oplist_size: OplistSize,
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
    pub dimension_ref: DimensionRef,
    pub chunk_pos: ChunkPos,
    pub biome_tags: Vec<(HashId, f32)>,
}

#[derive(Debug, Default)]
pub struct TerrGenOpTaskResult {
    pub new_pending_ops: Vec<PendingOp>,
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
