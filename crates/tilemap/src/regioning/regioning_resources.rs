use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};

use ::tilemap_shared::*;
use crate::regioning::regioning_sgc_components::StructuredGenConfig;
use serde::Deserialize;
pub use crate::regioning::regioning_seris::*;

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
