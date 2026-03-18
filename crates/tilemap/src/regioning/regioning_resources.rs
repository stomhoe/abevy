use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::*;

use ::tilemap_shared::*;
use crate::regioning::regioning_sgc_components::StructuredGenConfig;
use serde::{Deserialize, Serialize};
use common::common_components::*;
pub use crate::regioning::regioning_seris::*;

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(Replicated, Prefix::trunc("StructureGenerationSettings"), AssetScoped, HotReload)]
pub struct StructureGenerationSettings {
    /// Timeout in seconds to wait for StructureBuildCompliance before giving up
    pub structure_build_timeout_secs: f64,
}

impl Default for StructureGenerationSettings {
    fn default() -> Self {
        Self {
            structure_build_timeout_secs: 4.0,
        }
    }
}

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

#[derive(Resource, Default)]
pub struct Prioritized(pub Vec<Entity>);

#[derive(Resource, Default)]
pub struct PrioritizedPerRegion(pub HashMap<(DimensionRef, RegionPos), Vec<Entity>>);
