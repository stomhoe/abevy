use bevy::prelude::*;
use game_common::game_common_components::TemplEntiRef;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::terrain::terrprobe::opfilter::opfilter_resources::OpFilterRef;
use crate::terrain::terrprobe::terrprobe_messages::*;
use ::tilemap_shared::*;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ProbePatternSeri {
    #[serde(alias = "conc")]
    #[serde(alias = "co")]
    #[serde(alias = "concentric")]
    Concentric,
    #[serde(alias = "chunk")]
    Chunk,
    #[serde(alias = "region")]
    Region,
}
impl ProbePatternSeri {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "concentric" | "conc" | "co" => Some(Self::Concentric),
            "chunk" => Some(Self::Chunk),
            "region" => Some(Self::Region),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct TerrProbeTempl {
    pub opfilter_ref: OpFilterRef,
    pub sgc_admitted_tiles_as_found_pos: Vec<TemplEntiRef>,
    pub sgc_whitelist: Vec<Entity>,
    pub sgc_blacklist: Vec<Entity>,
    pub sgc_required_tile_tags: HashSet<String>,
    pub probe_pattern: ProbePattern,
    pub step_size: u16,
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub max_emitted_results: u32,
    pub min_result_distance: u16,
    pub collect: bool,
}

impl TerrProbeTempl {
    pub fn from_seri(
        opfilter_ref: OpFilterRef,
        structuregen_whitelist: Vec<Entity>,
        structuregen_blacklist: Vec<Entity>,
        sgc_required_tile_tags: HashSet<String>,
        sgc_admitted_tiles_as_found_pos: Vec<TemplEntiRef>,
        probe_pattern: ProbePatternSeri,
        concentric_sample_spacing: f32,
        step_size: u16,
        region_multiplier: f32,
        max_batches: u16,
        iterations_per_batch: u16,
        max_emitted_results: u32,
        min_result_distance: u16,
        collect: bool,
    ) -> Self {
        Self {
            opfilter_ref,
            sgc_admitted_tiles_as_found_pos,
            sgc_whitelist: structuregen_whitelist,
            sgc_blacklist: structuregen_blacklist,
            sgc_required_tile_tags,
            probe_pattern: match probe_pattern {
                ProbePatternSeri::Concentric => ProbePattern::concentric(step_size.max(1) as f32, concentric_sample_spacing),
                ProbePatternSeri::Chunk => ProbePattern::chunk(ChunkPos::default()),
                ProbePatternSeri::Region => ProbePattern::region(step_size, region_multiplier),
            },
            step_size,
            max_batches,
            iterations_per_batch,
            max_emitted_results,
            min_result_distance,
            collect,
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
            collect_all_successes: self.collect
                && matches!(self.probe_pattern, ProbePattern::Chunk(_) | ProbePattern::Region { .. }),
            ..Default::default()
        }
    }
}


#[derive(Component, Debug, Default, Copy, Clone, )]
pub struct AwaitingStartSearch;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, )]
pub struct SearchingForSuitablePos {
    pub requester: Entity,
    pub collect_all_successes: bool,
}
