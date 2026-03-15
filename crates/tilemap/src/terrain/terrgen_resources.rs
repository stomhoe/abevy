#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_components::{HashId, HashIdMap};

use crate::terrain::{
    terrgen_components::Terrgen,
    terrgen_seris::*,
};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos};

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

#[derive(Resource, Debug, Default)]
pub struct TerrGenDisabledGposByChunk(pub HashMap<(DimensionRef, ChunkPos), HashSet<GlobalTilePos>>);

impl TerrGenDisabledGposByChunk {
    pub fn insert_for_chunk(
        &mut self,
        dim_ref: DimensionRef,
        chunk_pos: ChunkPos,
        blocked_gpos: HashSet<GlobalTilePos>,
    ) {
        if blocked_gpos.is_empty() {
            return;
        }
        self.0
            .entry((dim_ref, chunk_pos))
            .or_default()
            .extend(blocked_gpos);
    }

    pub fn take_for_chunk(&mut self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> HashSet<GlobalTilePos> {
        self.0.remove(&(dim_ref, chunk_pos)).unwrap_or_default()
    }
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
common::define_entity_map_systems!(
    Terrgen,
    FnlSeri, "seri.tilemap.terrgen.noise", "fnl.ron",
);
