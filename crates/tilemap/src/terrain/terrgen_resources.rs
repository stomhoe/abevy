#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*, tasks::Task};
use common::common_components::{HashId, HashIdMap};

use crate::terrain::{
    terrprobe::terrprobe_messages::{SampledValueMatrixFound, SuitablePosFound, TerrProbeJob},
    terrgen_components::Terrgen,
    terrgen_messages::PendingOp,
};

use ::tilemap_shared::*;

use serde::{Deserialize, };
use std::{collections::HashMap, fs, path::Path};

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
    pub sampled_value_matrix_events: Vec<SampledValueMatrixFound>,
    pub tile_requests: Vec<TerrGenTileRequest>,
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
    #[serde(default = "default_frequency")]
    pub frequency: f32,
    #[serde(default = "default_noise_type")]
    pub noise_type: String,
    #[serde(default = "default_fractal_type")]
    pub fractal_type: String,
    #[serde(default = "default_octaves")]
    pub octaves: u8,
    #[serde(default = "default_lacunarity")]
    pub lacunarity: f32,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default)]
    pub weighted_strength: f32,
    #[serde(default = "default_ping_pong_strength")]
    pub ping_pong_strength: f32,
    #[serde(default = "default_cellular_distance_function")]
    pub cellular_distance_function: String,
    #[serde(default = "default_cellular_return_type")]
    pub cellular_return_type: String,
    #[serde(default = "default_cellular_jitter")]
    pub cellular_jitter: f32,
    #[serde(default = "default_domain_warp_type")]
    pub domain_warp_type: String,
    #[serde(default = "default_domain_warp_amp")]
    pub domain_warp_amp: f32,
}


#[derive(Deserialize, Asset, TypePath, )]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct GlobalGenSettingsSeri {
    #[serde(default)]
    pub seed: i32,
    #[serde(default = "default_global_world_freq")]
    pub world_freq: f32,
    #[serde(default)]
    pub hot_reload_window_open_on_start: bool,
    #[serde(default = "default_global_structure_build_timeout_secs")]
    pub structure_build_timeout_secs: f64,
    #[serde(default = "default_players_spawn_probe_id")]
    pub players_spawn_probe_id: String,
}
impl GlobalGenSettingsSeri {
    pub fn to_global_gen_settings(&self) -> GlobalGenSettings {
        GlobalGenSettings {
            seed: self.seed,
            world_freq: self.world_freq,
            hot_reload_window_open_on_start: self.hot_reload_window_open_on_start,
            structure_build_timeout_secs: self.structure_build_timeout_secs,
            players_spawn_probe_id: common::common_components::StrId::trunc(&self.players_spawn_probe_id),
        }
    }
}

fn default_global_world_freq() -> f32 { 0.02 }
fn default_global_structure_build_timeout_secs() -> f64 { 4.0 }
fn default_players_spawn_probe_id() -> String { "suland".to_string() }
fn default_frequency() -> f32 { 0.01 }
fn default_noise_type() -> String { "OpenSimplex2".to_string() }
fn default_fractal_type() -> String { "None".to_string() }
fn default_octaves() -> u8 { 3 }
fn default_lacunarity() -> f32 { 2.0 }
fn default_gain() -> f32 { 0.5 }
fn default_ping_pong_strength() -> f32 { 2.0 }
fn default_cellular_distance_function() -> String { "EuclideanSq".to_string() }
fn default_cellular_return_type() -> String { "Distance".to_string() }
fn default_cellular_jitter() -> f32 { 1.0 }
fn default_domain_warp_type() -> String { "OpenSimplex2".to_string() }
fn default_domain_warp_amp() -> f32 { 1.0 }

pub fn load_global_gen_settings_seri_defs() -> Vec<GlobalGenSettingsSeri> {
    let db = match common::def_db::DefDatabase::<GlobalGenSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(GlobalGenSettingsSeri),
        &["world_gen.settings.ron"],
        |_| "global_gen_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(
                target: "terrgen_init",
                "Failed loading GlobalGenSettingsSeri defs: {err:#}"
            );
            return Vec::new();
        }
    };
    for ov in db.overrides() {
        info!(
            target: "terrgen_init",
            "GlobalGenSettingsSeri overridden: '{}' -> '{}'",
            ov.previous_source.rel_path,
            ov.replacement_source.rel_path
        );
    }
    let defs: Vec<_> = db.into_records().into_iter().map(|r| r.value).collect();
    if !defs.is_empty() {
        return defs;
    }
    load_global_gen_settings_from_file()
        .into_iter()
        .collect()
}

fn load_global_gen_settings_from_file() -> Option<GlobalGenSettingsSeri> {
    let path = Path::new("assets/ron/tilemap/world_gen.settings.ron");
    let Ok(contents) = fs::read_to_string(path) else {
        return None;
    };
    match ron::from_str::<GlobalGenSettingsSeri>(&contents) {
        Ok(def) => Some(def),
        Err(err) => {
            error!(
                target: "terrgen_init",
                "Failed parsing '{}': {err}",
                path.display()
            );
            None
        }
    }
}

common::define_entity_map_systems!(
    Terrgen,
    FnlSeri, "seri.tilemap.terrgen.noise", "fnl.ron",
);
