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
    #[serde(alias = "curved_sun")]
    Swirl,
    #[serde(alias = "spiral")]
    Spiral,
    #[serde(alias = "conc")]
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
            "sun" => Some(Self::Sun),
            "curved_sun" | "swirl" => Some(Self::Swirl),
            "spiral" => Some(Self::Spiral),
            "concentric" | "conc" => Some(Self::Concentric),
            "chunk" => Some(Self::Chunk),
            "region" => Some(Self::Region),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct TerrProbeTempl {
    pub opfilter: OpFilter,
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
            opfilter: OpFilter {
                tags: Default::default(),
                var_name_hash: None,
                min_val: f32::NEG_INFINITY,
                max_val: f32::INFINITY,
            },
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
        opfilter: OpFilter,
        structuregen_whitelist: Vec<Entity>,
        structuregen_blacklist: Vec<Entity>,
        sgc_required_tile_tags: HashSet<String>,
        sgc_admitted_tiles_as_found_pos: Vec<EntityZeroRef>,
        probe_pattern: ProbePatternSeri,
        ray_curve_per_distance: f32,
        concentric_radius_step: f32,
        concentric_sample_spacing: f32,
        step_size: u16,
        max_batches: u16,
        iterations_per_batch: u16,
        max_emitted_results: u16,
        min_result_distance: u16,
    ) -> Self {
        Self {
            opfilter,
            sgc_admitted_tiles_as_found_pos,
            sgc_whitelist: structuregen_whitelist,
            sgc_blacklist: structuregen_blacklist,
            sgc_required_tile_tags,
            probe_pattern: match probe_pattern {
                ProbePatternSeri::Sun => ProbePattern::sun(ray_curve_per_distance),
                ProbePatternSeri::Swirl => ProbePattern::sun(if ray_curve_per_distance < 0.0 {
                    0.015
                } else {
                    ray_curve_per_distance
                }),
                ProbePatternSeri::Spiral => ProbePattern::spiral(GlobalTilePos::default()),
                ProbePatternSeri::Concentric => ProbePattern::concentric(concentric_radius_step, concentric_sample_spacing),
                ProbePatternSeri::Chunk => ProbePattern::chunk(ChunkPos::default()),
                ProbePatternSeri::Region => ProbePattern::region(step_size),
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
