use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath)]
pub struct SgcSeri {
    pub id: String,
    pub structure_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub args: HashMap<String, Vec<String>>,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub priority: f32,
    #[serde(default)]
    pub pdisk_mindist_and_tag: Vec<(Option<u8>, String)>,
    #[serde(default)]
    pub min_dists_from_other_structures: HashMap<String, u8>,
    #[serde(default)]
    pub exclusive_for_dimensions: Vec<String>,
    #[serde(default)]
    pub run_before_sgcs_with_tags: HashSet<String>,
    #[serde(default)]
    pub run_after_sgcs_with_tags: HashSet<String>,
    #[serde(default)]
    pub whitelisted_tags: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tags: HashSet<String>,
    #[serde(default = "default_max_per_region")]
    pub max_per_region: u32,
}
fn default_max_per_region() -> u32 { 1024 }
fn default_weight() -> f32 { f32::NEG_INFINITY }

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct StructureGenerationSettingsSeri {
    #[serde(default = "default_structure_build_timeout_secs")]
    pub structure_build_timeout_secs: f64,
}

impl StructureGenerationSettingsSeri {
    pub fn to_structure_generation_settings(&self) -> super::regioning_resources::StructureGenerationSettings {
        super::regioning_resources::StructureGenerationSettings {
            structure_build_timeout_secs: self.structure_build_timeout_secs,
        }
    }
}

fn default_structure_build_timeout_secs() -> f64 { 4.0 }

pub fn load_structure_generation_settings_seri_defs() -> Vec<StructureGenerationSettingsSeri> {
    let db = match common::def_db::DefDatabase::<StructureGenerationSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(StructureGenerationSettingsSeri),
        &["structure_generation.settings.ron"],
        |_| "structure_generation_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(
                target: common::log_targets::TERRGEN_INIT,
                "Failed loading StructureGenerationSettingsSeri defs: {err:#}"
            );
            return Vec::new();
        }
    };
    db.into_records().into_iter().map(|r| r.value).collect()
}
