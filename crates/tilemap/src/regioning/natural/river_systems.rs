use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::{common_components::HashId, log_targets::RIVER_SYSTEM};
use std::{alloc, collections::VecDeque};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalGenSettings, GlobalTilePos, HashablePosVec, RegionPos};

use crate::{
    regioning::{
        regioning_components::ClaimList,
        regioning_messages::{ChunksClaim, OfferChunk, SgcPrepareTilesOrder, StructureBuildCompliance, TerrGenDisabledGposForChunks},
        regioning_sgc_components::StructuredGenConfig,
    },
    terrain::terrprobe::{
        terrprobe_components::TerrProbeTempl,
        terrprobe_messages::{SampledValuesCollected, TerrProbeJob},
        terrprobe_resources::TerrProbeTemplEntityMap,
    },
    tile::tile_resources::{TileEntityMap, TileRef},
};
use super::river_components::*;




#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, non_camel_case_types)]
pub struct claim_chunks_for_river_structuresLocals<'w, 's> {
    completed_probes: Local<'s, EntityHashSet>,
    completed_with_samples: Local<'s, EntityHashSet>,
    claims_to_emit: Local<'s, Vec<ChunksClaim>>,
    skipped_offers: Local<'s, Vec<(Entity, usize)>>,
    claims_writer: MessageWriter<'w, ChunksClaim>,
}

#[allow(unused_parens)]
pub fn claim_chunks_for_river_structures(
    mut cmd: Commands,
    mut offered_chunks: MessageReader<OfferChunk>,
    structured_gens: Query<&StructuredGenConfig>,
    region_dimension: Query<&DimensionRef>,
    settings_q: Query<&GlobalGenSettings>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    mut terrprobe_writer: MessageWriter<TerrProbeJob>,
    mut sampled_values_reader: MessageReader<SampledValuesCollected>,
    mut river_debug: ResMut<RiverDebugData>,
    mut claim_state: claim_chunks_for_river_structuresLocals,
) {
    const RIVER_REGION_PROBE_ID: HashId = HashId::hash("river_region_probe");

    
}

#[allow(unused_parens)]
pub fn river_structure_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<&StructuredGenConfig>,
    tiles_map: Res<TileEntityMap>,
    mut river_debug: ResMut<RiverDebugData>,
    mut writer: MessageWriter<StructureBuildCompliance>,
) {
    const RIVER: HashId = HashId::hash("river");

}