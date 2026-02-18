use bevy::{platform::collections::{HashSet, HashMap}, prelude::*};

use ::tilemap_shared::*;
use crate::{regioning::regioning_sgc_components::{StructuredGenConfig}, };
use serde::Deserialize;

common::define_entity_map_systems!(
    StructuredGenConfig,
    (),
    Sgc,
    "sgc",
    "SGC",
    StructuredGenConfig,
    common::common_components::StrId,
    SgcSeri, "seri.tilemap.region.sgc", "sgc.ron",
);
#[derive(Resource, Default)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Deserialize, Asset, TypePath, )]
pub struct SgcSeri {
    pub id: String,
    /// village, cave, dungeon, fort, etc
    pub structure_id: String,

    #[serde(default)]
    pub tags: Vec<String>,

    /// extra arguments given to structure generation
    #[serde(default)]
    pub args: HashMap<String, Vec<String>>,
    /// weight in weighted map of structured gens for region. (more weight= likely for this structure to be generated first within the map of valid generations for that region)
    pub weight: f32,

    //expected terr conditions for spawning
    #[serde(default)]
    pub whitelisted_filters: HashSet<String>,

    #[serde(default)]
    pub pdisk_mindist_and_tag: Vec<(Option<u8>, String)>,

    //para evitar adyacencia con chunk fronterizo de otra region, comparar hash del chunkpos con adyacentes y que se quede si es el más grande?
    #[serde(default)]
    pub min_dists_from_other_structures: HashMap<String, u8>,//in chunks

    //if empty, active in all dimensions (but that dimension must have a matcfhing tag)
    #[serde(default)]
    pub exclusive_for_dimensions: Vec<String>,//TODO user tags en vez de ids?


    #[serde(default = "default_max_per_region")]
    pub max_per_region: u32,

}
fn default_max_per_region() -> u32 { 1024 }
