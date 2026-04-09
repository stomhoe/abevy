#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, platform::collections::HashMap, prelude::*};
use common::common_components::{HashId, HashIdMap};
use common::common_tag_components::HashedTagsVec;
use std::sync::Arc;

use crate::terrain::{
    terrgen_components::Terrgen,
    terrgen_async_resources::TerrGenBlockedGposMask,
    terrgen_components::FnlNoiseComp,
    operation_list::operation_list_components::OperationList,
    terrprobe::opfilter::opfilter_components::OpFilter,
    terrgen_seris::*,
};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos, OplistSize};

#[derive(Debug, Clone)]
pub struct TerrGenDebugSample {
    pub dimension_ref: DimensionRef,
    pub gpos: GlobalTilePos,
    pub oplist: HashId,
    pub oplist_id: HashId,
    pub output: f32,
    pub variables: HashIdMap<f32>,
}

#[derive(Debug, Clone)]
pub struct TerrGenTileDebugInfo {
    pub oplist: HashId,
    pub oplist_id: HashId,
    pub output: f32,
    pub variables: HashIdMap<f32>,
}
impl Default for TerrGenTileDebugInfo {
    fn default() -> Self {
        Self {
            oplist: HashId::default(),
            oplist_id: HashId::default(),
            output: 0.0,
            variables: HashIdMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrGenDebugTileKey {
    pub dimension: HashId,
    pub gpos: IVec2,
    pub oplist: HashId,
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
pub struct TerrGenDisabledGposByChunk(pub HashMap<(DimensionRef, ChunkPos), TerrGenBlockedGposMask>);

impl TerrGenDisabledGposByChunk {
    pub fn insert_for_chunk(
        &mut self,
        dim_ref: DimensionRef,
        chunk_pos: ChunkPos,
        blocked_gpos: TerrGenBlockedGposMask,
    ) {
        if blocked_gpos.is_empty() {
            return;
        }
        self.0.insert((dim_ref, chunk_pos), blocked_gpos);
    }

    pub fn get_for_chunk(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> TerrGenBlockedGposMask {
        self.0.get(&(dim_ref, chunk_pos)).cloned().unwrap_or_default()
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

#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct TerrGenSharedTaskData {
    pub(crate) shared: Option<Arc<TerrGenSharedTaskDataInner>>,
}

#[derive(Debug)]
pub(crate) struct TerrGenSharedTaskDataInner {
    pub(crate) oplists: HashIdMap<OperationList>,
    pub(crate) oplist_debug_var_ids: HashIdMap<Vec<HashId>>,
    pub(crate) oplist_sizes: HashIdMap<OplistSize>,
    pub(crate) oplist_tags: HashIdMap<HashedTagsVec>,
    pub(crate) noises: HashIdMap<FnlNoiseComp>,
    pub(crate) filters: HashIdMap<OpFilter>,
}
common::define_entity_map_systems!(
    main_component: Terrgen,
    with_filters: (),
    abbreviation: Terrgen,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "terrgen noise",
    despawn_trigger: Terrgen,
    id_type: common::common_components::StrId,
    assets: [(FnlSeri, "seri.tilemap.terrgen.noise", "fnl.ron")],
);
