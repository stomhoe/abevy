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
    SgcSeri, "ron/tilemap/region/structures", "sgc.ron",
);
#[derive(Resource, Default)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Deserialize, Asset, TypePath, )]
pub struct SgcSeri {
    pub id: String,
    /// village, cave, dungeon, fort, etc
    pub structure_id: String,

    pub tags: Option<Vec<String>>,

    /// extra arguments given to structure generation
    pub args: Option<HashMap<String, Vec<String>>>,
    /// weight in weighted map of structured gens for region. (more weight= likely for this structure to be generated first within the map of valid generations for that region)
    pub weight: f32,

    //expected terr conditions for spawning
    pub whitelisted_filters: Option<HashSet<String>>,

    pub pdisk_mindist_and_tag: Option<Vec<(Option<u8>, String)>>,

    //para evitar adyacencia con chunk fronterizo de otra region, comparar hash del chunkpos con adyacentes y que se quede si es el más grande?
    pub min_dists_from_other_structures: Option<HashMap<String, u8>>,//in chunks

    //if empty, active in all dimensions (but that dimension must have a matcfhing tag)
    pub exclusive_for_dimensions: Option<Vec<String>>,//TODO user tags en vez de ids?

    pub min_used_chunks: Option<u8>,//structure's own minimum chunk usage takes priority over this one
    pub max_used_chunks: Option<u16>,

    pub max_per_region: Option<u32>,

}
