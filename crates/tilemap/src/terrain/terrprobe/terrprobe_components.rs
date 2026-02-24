use bevy::prelude::*;
use game_common::game_common_components::EntityZeroRef;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::terrain::terrprobe::opfilter::opfilter_components::OpFilter;
use crate::terrain::terrprobe::terrprobe_messages::{ProbePattern, TerrProbeJob};
use ::tilemap_shared::*;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ProbePatternSeri {
    #[serde(alias = "sun")]
    Sun,
    #[serde(alias = "spiral")]
    Spiral,
}
impl ProbePatternSeri {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "sun" => Some(Self::Sun),
            "spiral" => Some(Self::Spiral),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct TerrProbeTempl {
    pub opfilter_ent: Option<Entity>,
    pub opfilter_override: Option<OpFilter>,
    pub sgc_admitted_tiles_as_found_pos: Vec<EntityZeroRef>,
    pub sgc_whitelist: Vec<Entity>,
    pub sgc_blacklist: Vec<Entity>,
    pub sgc_required_tile_tags: HashSet<String>,
    pub probe_pattern: ProbePattern,
    pub step_size: u16,
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub max_emitted_results: u16,
    pub min_result_distance: u16,
}

impl Default for TerrProbeTempl {
    fn default() -> Self {
        Self {
            opfilter_ent: None,
            opfilter_override: None,
            sgc_admitted_tiles_as_found_pos: Vec::new(),
            sgc_whitelist: Vec::new(),
            sgc_blacklist: Vec::new(),
            sgc_required_tile_tags: HashSet::default(),
            probe_pattern: ProbePattern::spiral(GlobalTilePos::default()),
            step_size: 1,
            max_batches: 1000,
            iterations_per_batch: 10000,
            max_emitted_results: 1,
            min_result_distance: 0,
        }
    }
}

impl TerrProbeTempl {
    pub fn from_seri(
        opfilter_ent: Option<Entity>,
        opfilter_override: Option<OpFilter>,
        structuregen_whitelist: Vec<Entity>,
        structuregen_blacklist: Vec<Entity>,
        sgc_required_tile_tags: HashSet<String>,
        sgc_admitted_tiles_as_found_pos: Vec<EntityZeroRef>,
        probe_pattern: ProbePatternSeri,
        step_size: u16,
        max_batches: u16,
        iterations_per_batch: u16,
        max_emitted_results: u16,
        min_result_distance: u16,
    ) -> Self {
        Self {
            opfilter_ent,
            opfilter_override,
            sgc_admitted_tiles_as_found_pos,
            sgc_whitelist: structuregen_whitelist,
            sgc_blacklist: structuregen_blacklist,
            sgc_required_tile_tags,
            probe_pattern: match probe_pattern {
                ProbePatternSeri::Sun => ProbePattern::sun(),
                ProbePatternSeri::Spiral => ProbePattern::spiral(GlobalTilePos::default()),
            },
            step_size,
            max_batches,
            iterations_per_batch,
            max_emitted_results,
            min_result_distance,
        }
    }

    pub fn to_probe(&self, templ_ent: Entity, dimension_ref: DimensionRef, search_start_pos: GlobalTilePos) -> TerrProbeJob {
        TerrProbeJob {
            templ_ent,
            dimension_ref,
            search_start_pos,
            min_result_distance: self.min_result_distance,
            structuregen_whitelist: self.sgc_whitelist.clone(),
            structuregen_blacklist: self.sgc_blacklist.clone(),
            ..Default::default()
        }
    }
}


#[derive(Component, Debug, Default, Copy, Clone, )]
pub struct AwaitingStartSearch;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct SearchingForSuitablePos { pub requester: Entity, }
