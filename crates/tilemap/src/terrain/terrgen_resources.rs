#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::common_components::{HashId, HashIdMap};

use crate::terrain::{
    terrgen_components::Terrgen,
    terrgen_seris::*,
};
use tilemap_shared::{DimensionRef, GlobalTilePos};
use std::collections::HashMap;

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
common::define_entity_map_systems!(
    Terrgen,
    FnlSeri, "seri.tilemap.terrgen.noise", "fnl.ron",
);
