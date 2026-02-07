#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*, tasks::Task};
use common::{common_components::{HashId}, common_types::HashIdToEntityMap};

use crate::{terrain_gen::{terrgen_components::Terrgen, terrgen_messages::{PendingOp, SuitablePosFound, TerrainProbe}}, tile::{tile_components::{KeepDistanceFrom, MinDistancesMap, }, }, tilemap_resources};
use dimension_shared::DimensionRef;

use ::tilemap_shared::*;
use game_common::{game_common_components::*};

use serde::{Deserialize, Serialize};




#[derive(Debug, Clone)]
pub struct TerrGenLaunchWork {
    pub chunk_ent: Entity,
    pub chunk_pos: ChunkPos,
    pub dim_ref: DimensionRef,
    pub root_oplist: Entity,
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

#[derive(Debug, Default)]
pub struct TerrGenOpTaskResult {
    pub new_pending_ops: Vec<PendingOp>,
    pub sampled_value_events: Vec<SuitablePosFound>,
    pub tile_requests: Vec<TerrGenTileRequest>,
}

#[derive(Debug, Default)]
pub struct TerrGenSearchTaskResult {
    pub new_pending_ops: Vec<PendingOp>,
    pub new_pos_searches: Vec<TerrainProbe>,
    pub search_failed: Vec<Entity>,
}

#[derive(Resource, Debug, Default)]
pub struct TerrGenAsyncTasks {
    pub launch_tasks: Vec<Task<Vec<PendingOp>>>,
    pub op_tasks: Vec<Task<TerrGenOpTaskResult>>,
    pub search_tasks: Vec<Task<TerrGenSearchTaskResult>>,
}


#[derive(Deserialize, Asset, Reflect, )]
pub struct NoiseSerialization {
    pub id: String,
    /// Default is 0.01
    pub frequency: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2S, 2: Cellular, 3: Perlin, 4: ValueCubic, 5: Value
    pub noise_type: Option<u32>,
    /// 0: None, 1: FBm, 2: Ridged, 3: PingPong, 4: DomainWarpProgressive, 5: DomainWarpIndependent,
    pub fractal_type: Option<u32>,
    /// Default is 3
    pub octaves: Option<u8>,
    /// Default is 2.0
    pub lacunarity: Option<f32>,
    /// Default is 0.5
    pub gain: Option<f32>,
    /// Default is 0.0
    pub weighted_strength: Option<f32>,
    /// Default is 2.0
    pub ping_pong_strength: Option<f32>,
    /// 0: Euclidean, 1: EuclideanSq, 2: Manhattan, 3: Hybrid
    pub cellular_distance_function: Option<u32>,
    /// 0: CellValue, 1: Distance, 2: Distance2, 3: Distance2Add, 4: Distance2Sub, 5: Distance2Mul, 6: Distance2Div
    pub cellular_return_type: Option<u32>,
    /// Default is 1.0
    pub cellular_jitter: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2Reduced, 2: BasicGrid
    pub domain_warp_type: Option<u32>,
    /// Default is 1.0
    pub domain_warp_amp: Option<f32>,
}

pub type NoiseSeri = NoiseSerialization;

#[derive(Deserialize, Asset, Reflect, )]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}



#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct OpListSerialization {
    pub id: String,
    pub tags: Option<Vec<String>>,
    pub root_in_dimensions: Vec<String>,
    /// input variable index, operation name, operands, ouput variable indexs 
    pub operation_operands: Vec<(String, Vec<String>, u8)>,
    /// oplist id, produced tiles
    pub bifs: Vec<(String, Vec<String>)>,
    pub size: Option<[u32; 2]>
}
impl OpListSerialization {
    pub fn is_root(&self) -> bool {
        self.root_in_dimensions.iter().any(|s| !s.is_empty())
    }
}

pub type OpListSeri = OpListSerialization;



common::define_entity_map_systems!(
    Terrgen,
    NoiseSeri, "ron/tilemap/terrgen/noise", "fnl.ron",
    OpListSeri, "ron/tilemap/terrgen/oplist", "oplist.ron",
);

