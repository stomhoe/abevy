use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::{common_components::HashId, log_targets::RIVER_SYSTEM};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos, RegionPos};

use crate::{
    regioning::{
        regioning_components::ClaimList,
        regioning_messages::{ChunksClaim, OfferChunk, SgcPrepareTilesOrder, StructureBuildCompliance},
        regioning_sgc_components::StructuredGenConfig,
    },
    terrain::terrprobe::{
        terrprobe_components::TerrProbeTempl,
        terrprobe_messages::{SampledValueMatrixFound, TerrProbeJob},
        terrprobe_resources::TerrProbeTemplEntityMap,
    },
};

const RIVER: HashId = HashId::hash("river");
const RELAND_ALL_PROBE_ID: &str = "river_land_all_probe";

#[derive(Component, Debug, Clone, Copy)]
pub struct RiverProbeRequest {
    region_ent: Entity,
    region_pos: RegionPos,
    start_chunk: ChunkPos,
}

#[derive(Debug, Clone, Default)]
pub struct RiverDebugEvent {
    pub offer_i: u64,
    pub start_chunk: ChunkPos,
    pub is_failure: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct RiverRegionDebugInfo {
    pub active_probe_chunks: HashSet<ChunkPos>,
    pub claimed_chunks: HashSet<ChunkPos>,
    pub failed_chunks: HashSet<ChunkPos>,
    pub river_tiles: HashSet<GlobalTilePos>,
    pub sampled_points: HashMap<GlobalTilePos, f32>,
    pub success_count: u32,
    pub failure_count: u32,
    pub recent_events: Vec<RiverDebugEvent>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct RiverDebugData(pub HashMap<(DimensionRef, RegionPos), RiverRegionDebugInfo>);
impl RiverDebugData {
    fn region_mut(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) -> &mut RiverRegionDebugInfo {
        self.0.entry((dimension_ref, region_pos)).or_default()
    }

    pub fn remove_region(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos) {
        self.0.remove(&(dimension_ref, region_pos));
    }

    fn mark_probe_started(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, start_chunk: ChunkPos) {
        self.region_mut(dimension_ref, region_pos)
            .active_probe_chunks
            .insert(start_chunk);
    }

    fn mark_probe_finished(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, start_chunk: ChunkPos) {
        let Some(info) = self.0.get_mut(&(dimension_ref, region_pos)) else {
            return;
        };
        info.active_probe_chunks.remove(&start_chunk);
    }

    fn mark_sample(&mut self, dimension_ref: DimensionRef, region_pos: RegionPos, pos: GlobalTilePos, val: f32) {
        self.region_mut(dimension_ref, region_pos).sampled_points.insert(pos, val);
    }
}

#[derive(Resource, Default)]
pub struct RiverPlans;

#[derive(bevy::ecs::system::SystemParam)]
pub struct RiverClaimState<'w, 's> {
    completed_probes: Local<'s, EntityHashSet>,
    skipped_offers: Local<'s, Vec<(Entity, usize)>>,
    _claims_writer: MessageWriter<'w, ChunksClaim>,
}

#[allow(unused_parens)]
pub fn claim_chunks_for_river_structures(
    mut cmd: Commands,
    mut offered_chunks: MessageReader<OfferChunk>,
    structured_gens: Query<&StructuredGenConfig>,
    region_dimension: Query<&DimensionRef>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    probe_requests: Query<(Entity, &RiverProbeRequest)>,
    mut terrprobe_writer: MessageWriter<TerrProbeJob>,
    mut sampled_values_reader: MessageReader<SampledValueMatrixFound>,
    mut claimlists: Query<&mut ClaimList>,
    mut river_debug: ResMut<RiverDebugData>,
    mut claim_state: RiverClaimState,
) {
    claim_state.completed_probes.clear();
    claim_state.skipped_offers.clear();
    trace!(target: RIVER_SYSTEM, "claim_chunks_for_river_structures tick");

    let Ok(probe_templ_ent) = terrprobe_entity_map.0.get_cloned(RELAND_ALL_PROBE_ID) else {
        error!(target: RIVER_SYSTEM, "Missing terrprobe template id '{}'", RELAND_ALL_PROBE_ID);
        return;
    };
    let Ok(probe_templ) = terrprobe_query.get(probe_templ_ent) else {
        error!(target: RIVER_SYSTEM, "Terrprobe entity {:?} missing TerrProbeTempl", probe_templ_ent);
        return;
    };

    let mut regions_with_probe: HashSet<(DimensionRef, RegionPos)> = HashSet::default();
    for (_, req) in probe_requests.iter() {
        let Ok(&dimension_ref) = region_dimension.get(req.region_ent) else {
            continue;
        };
        regions_with_probe.insert((dimension_ref, req.region_pos));
    }

    for offer in offered_chunks.read() {
        let Ok(cfg) = structured_gens.get(offer.structured_gen_cfg_ent) else {
            error!(
                target: RIVER_SYSTEM,
                "Offer {}: missing StructuredGenConfig entity {:?}",
                offer.i,
                offer.structured_gen_cfg_ent
            );
            claim_state.skipped_offers.push((offer.region_ent, offer.i as usize));
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }

        claim_state.skipped_offers.push((offer.region_ent, offer.i as usize));
        let Ok(&dimension_ref) = region_dimension.get(offer.region_ent) else {
            error!(
                target: RIVER_SYSTEM,
                "Offer {}: region {:?} missing DimensionRef",
                offer.i,
                offer.region_ent
            );
            continue;
        };

        let region_pos = offer.start_pos.to_region_pos();
        if regions_with_probe.contains(&(dimension_ref, region_pos)) {
            trace!(
                target: RIVER_SYSTEM,
                "Offer {}: skipping region {:?} in dim {:?}, probe already active",
                offer.i,
                region_pos,
                dimension_ref
            );
            continue;
        }

        let center = offer.start_pos.to_tilepos()
            + bevy::math::IVec2::new(
                (ChunkPos::CHUNK_SIZE.x / 2) as i32,
                (ChunkPos::CHUNK_SIZE.y / 2) as i32,
            );

        let probe_ent = cmd
            .spawn(RiverProbeRequest {
                region_ent: offer.region_ent,
                region_pos,
                start_chunk: offer.start_pos,
            })
            .id();

        let mut probe = probe_templ.to_probe(probe_templ_ent, dimension_ref, center);
        probe.requester = probe_ent;
        terrprobe_writer.write(probe);
        info!(
            target: RIVER_SYSTEM,
            "Offer {}: spawned river probe {:?} at center {:?}, region {:?}, dim {:?}",
            offer.i,
            probe_ent,
            center,
            region_pos,
            dimension_ref
        );

        river_debug.mark_probe_started(dimension_ref, region_pos, offer.start_pos);
        regions_with_probe.insert((dimension_ref, region_pos));
    }

    for sampled_values in sampled_values_reader.read() {
        let Ok((_, req)) = probe_requests.get(sampled_values.requester) else {
            error!(
                target: RIVER_SYSTEM,
                "SampledValueMatrixFound for unknown requester {:?}, region_pos <unknown>",
                sampled_values.requester,
            );
            continue;
        };
        let Ok(&dimension_ref) = region_dimension.get(req.region_ent) else {
            error!(
                target: RIVER_SYSTEM,
                "Requester {:?} region {:?} missing DimensionRef",
                sampled_values.requester,
                req.region_ent
            );
            continue;
        };
        let mut matched_samples = 0_u32;
        for (sample_pos, sample_val_opt) in sampled_values.matrix.values.iter() {
            let Some(sample_val) = sample_val_opt else {
                continue;
            };
            river_debug.mark_sample(dimension_ref, req.region_pos, *sample_pos, *sample_val);
            matched_samples = matched_samples.saturating_add(1);
        }
        let info = river_debug.region_mut(dimension_ref, req.region_pos);
        if matched_samples > 0 {
            info.success_count = info.success_count.saturating_add(1);
            info!(
                target: RIVER_SYSTEM,
                "Probe {:?} region {:?} dim {:?}: captured {} sampled points (total cached: {})",
                sampled_values.requester,
                req.region_pos,
                dimension_ref,
                matched_samples,
                info.sampled_points.len()
            );
        } else {
            info.failure_count = info.failure_count.saturating_add(1);
            error!(
                target: RIVER_SYSTEM,
                "Probe {:?} region {:?} dim {:?}: sampled matrix contained 0 matched points",
                sampled_values.requester,
                req.region_pos,
                dimension_ref
            );
        }
        claim_state.completed_probes.insert(sampled_values.requester);
    }

    for ent in claim_state.completed_probes.drain() {
        let Ok((_, req)) = probe_requests.get(ent) else {
            cmd.entity(ent).despawn();
            continue;
        };
        let Ok(&dimension_ref) = region_dimension.get(req.region_ent) else {
            cmd.entity(ent).despawn();
            continue;
        };
        river_debug.mark_probe_finished(dimension_ref, req.region_pos, req.start_chunk);
        cmd.entity(ent).despawn();
    }

    for (region_ent, offer_i) in claim_state.skipped_offers.drain(..) {
        let Ok(mut claimlist) = claimlists.get_mut(region_ent) else {
            continue;
        };
        claimlist.skipped_is.insert(offer_i);
    }
}

#[allow(unused_parens)]
pub fn river_structure_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<&StructuredGenConfig>,
    mut writer: MessageWriter<StructureBuildCompliance>,
) {
    let mut compliances_to_emit = Vec::new();
    for order in reader.read() {
        let Ok(cfg) = structured_gens.get(order.structured_gen_cfg_ent) else {
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }
        compliances_to_emit.push(StructureBuildCompliance {
            i: order.i,
            structure_gen_cfg_ent: order.structured_gen_cfg_ent,
            dimension_ref: order.dimension_ref,
            chunks: Vec::new(),
            terrgen_disabled_for_chunks: Vec::new(),
        });
    }
    writer.write_batch(compliances_to_emit);
}
