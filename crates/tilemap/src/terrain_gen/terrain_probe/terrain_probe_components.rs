use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::terrain_gen::terrain_probe::terrain_probe_messages::{ProbePattern, TerrainProbe};
use ::tilemap_shared::*;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbePatternSeri {
    Sun,
    Spiral,
}

#[derive(Debug, Clone, Component, Deserialize, Serialize)]
pub struct TerrainProbeTemplate {
    pub opfilter_ent: Entity,
    pub probe_pattern: ProbePatternSeri,
    pub step_size: u16,
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub max_emitted_results: u16,
}

impl TerrainProbeTemplate {
    pub fn to_probe(&self, dimension_ref: DimensionRef, search_start_pos: GlobalTilePos) -> TerrainProbe {
        self.to_probe_with_filter(dimension_ref, search_start_pos, self.opfilter_ent)
    }

    pub fn to_probe_with_filter(
        &self,
        dimension_ref: DimensionRef,
        search_start_pos: GlobalTilePos,
        opfilter_ent: Entity,
    ) -> TerrainProbe {
        TerrainProbe {
            dimension_ref,
            search_start_pos,
            opfilter_ent,
            probe_pattern: match self.probe_pattern {
                ProbePatternSeri::Sun => ProbePattern::sun(),
                ProbePatternSeri::Spiral => ProbePattern::spiral(search_start_pos),
            },
            step_size: self.step_size,
            max_batches: self.max_batches,
            iterations_per_batch: self.iterations_per_batch,
            max_emitted_results: self.max_emitted_results,
            ..Default::default()
        }
    }
}
