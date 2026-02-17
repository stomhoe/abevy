#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*, tasks::Task};
use common::common_components::{HashId, HashIdMap};

use crate::terrain_gen::{
    terrain_probe::terrain_probe_messages::{SuitablePosFound, TerrainProbe},
    terrgen_components::Terrgen,
    terrgen_messages::PendingOp,
};

use ::tilemap_shared::*;

use serde::{Deserialize, };
use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub struct TerrGenOpTaskResult {
    pub new_pending_ops: Vec<PendingOp>,
    pub sampled_value_events: Vec<SuitablePosFound>,
    pub tile_requests: Vec<TerrGenTileRequest>,
    pub debug_samples: Vec<TerrGenDebugSample>,
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

#[derive(Debug, Clone)]
pub struct TerrGenDebugSample {
    pub dimension_ref: DimensionRef,
    pub gpos: GlobalTilePos,
    pub oplist: Entity,
    pub oplist_id: HashId,
    pub output: f32,
    pub variables: HashIdMap<f32>,
}

#[derive(Debug, Clone)]
pub struct TerrGenTileDebugInfo {
    pub oplist: Entity,
    pub oplist_id: HashId,
    pub output: f32,
    pub variables: HashIdMap<f32>,
}
impl Default for TerrGenTileDebugInfo {
    fn default() -> Self {
        Self {
            oplist: Entity::PLACEHOLDER,
            oplist_id: HashId::default(),
            output: 0.0,
            variables: HashIdMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrGenDebugTileKey {
    pub dimension: Entity,
    pub gpos: IVec2,
    pub oplist: Entity,
}

#[derive(Resource, Debug)]
pub struct TerrGenDebugGrid {
    pub enabled: bool,
    pub selected_metric: HashId,
    pub oplist_filter: Option<HashId>,
    pub max_entries: usize,
    pub bucket_size_tiles: i32,
    pub bucket_radius: i32,
    pub capture_margin_buckets: i32,
    pub tiles: HashMap<TerrGenDebugTileKey, TerrGenTileDebugInfo>,
}

impl Default for TerrGenDebugGrid {
    fn default() -> Self {
        Self {
            enabled: true,
            selected_metric: HashId::from("shore_proximity"),
            oplist_filter: None,
            max_entries: 150_000,
            bucket_size_tiles: 10,
            bucket_radius: 18,
            capture_margin_buckets: 8,
            tiles: HashMap::new(),
        }
    }
}


#[derive(Deserialize, Asset, TypePath, )]
pub struct FnlSeri {
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


#[derive(Deserialize, Asset, TypePath, )]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}

common::define_entity_map_systems!(
    Terrgen,
    FnlSeri, "ron/tilemap/terrgen/noise", "fnl.ron",
);
