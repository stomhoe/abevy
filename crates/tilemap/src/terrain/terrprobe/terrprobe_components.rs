use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::terrain::terrprobe::terrprobe_messages::{ProbePattern, TerrProbeJob};
use ::tilemap_shared::*;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ProbePatternSeri {
    #[serde(alias = "sun")]
    Sun,
    #[serde(alias = "spiral")]
    Spiral,
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct TerrProbeTempl {
    pub opfilter_ent: Entity,
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
            opfilter_ent: Entity::PLACEHOLDER,
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
        opfilter_ent: Entity,
        probe_pattern: ProbePatternSeri,
        step_size: u16,
        max_batches: u16,
        iterations_per_batch: u16,
        max_emitted_results: u16,
        min_result_distance: u16,
    ) -> Self {
        Self {
            opfilter_ent,
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
            ..Default::default()
        }
    }
}
