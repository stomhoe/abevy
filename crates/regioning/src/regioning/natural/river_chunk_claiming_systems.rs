use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::{AsyncComputeTaskPool, futures_lite::future},
};
use common::{common_components::HashId, log_targets::RIVER_SYSTEM};
use ::tilemap_shared::*;


use tilemap::terrain::terrprobe::{
    terrprobe_components::TerrProbeTempl,
    terrprobe_messages::{SampledValuesCollected, SearchFailed, TerrProbeJob},
    terrprobe_resources::TerrProbeTemplEntityMap,
};

use super::river_components::*;
use super::river_formation::{
    generate_river_region_plan,
};

const RIVER: HashId = HashId::hash("river");
const RIVER_REGION_PROBE_ID: &str = "river_region_probe";

#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, non_camel_case_types)]
pub struct claim_chunks_for_river_structuresLocals<'w, 's> {
    pending_river_regions: Local<'s, EntityHashSet>,
    claims_to_emit: Local<'s, Vec<ChunksClaim>>,
    skipped_offers: Local<'s, Vec<(Entity, usize)>>,
    pending_plan_tasks: Local<'s, Vec<bevy::tasks::Task<RiverPlanTaskResult>>>,
    claims_writer: MessageWriter<'w, ChunksClaim>,
}

#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, non_camel_case_types)]
pub struct claim_chunks_for_river_structuresQueries<'w, 's> {
    structured_gens: Query<'w, 's, &'static StructuredGenConfig>,
    region_info: Query<'w, 's, (&'static DimensionRef, &'static RegionPos)>,
    region_plans: Query<'w, 's, &'static RiverRegionPlan>,
    pending_offers: Query<'w, 's, &'static mut RiverPendingOffer>,
    settings_q: Query<'w, 's, &'static GlobalGenSettings>,
    terrprobe_query: Query<'w, 's, &'static TerrProbeTempl>,
    probe_requests: Query<'w, 's, &'static RiverProbeRequest>,
    claimlists: Query<'w, 's, &'static mut ClaimList>,
}

#[allow(unused_parens)]
pub fn claim_chunks_for_river_structures(
    mut cmd: Commands,
    mut offered_chunks: MessageReader<OfferChunk>,
    loaded_regions: Res<LoadedRegions>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    mut terrprobe_writer: MessageWriter<TerrProbeJob>,
    mut sampled_values_reader: MessageReader<SampledValuesCollected>,
    mut search_failed_reader: MessageReader<SearchFailed>,
    mut river_debug: ResMut<RiverDebugData>,
    mut claim_queries: claim_chunks_for_river_structuresQueries,
    mut claim_state: claim_chunks_for_river_structuresLocals,
) {
    claim_state.pending_river_regions.clear();
    claim_state.claims_to_emit.clear();
    claim_state.skipped_offers.clear();

    let claim_chunks_for_river_structuresLocals {
        pending_river_regions,
        claims_to_emit,
        skipped_offers,
        pending_plan_tasks,
        claims_writer,
    } = &mut claim_state;

    let mut river_offers_seen = 0_u32;
    let mut probes_spawned = 0_u32;
    let mut sampled_events = 0_u32;
    let mut sampled_points_total = 0_u32;
    let mut probes_completed = 0_u32;
    let mut claims_emitted = 0_u32;
    let mut offers_skipped = 0_u32;

    pending_plan_tasks.retain_mut(|task| {
        let Some(result) = future::block_on(future::poll_once(task)) else {
            return true;
        };

        let RiverPlanTaskResult {
            region_ent,
            sgc_ent,
            offer_i,
            dimension_ref,
            region_pos,
            region_debug,
            plan,
        } = result;

        if plan.river_tiles.is_empty() {
            error!(target: RIVER_SYSTEM, "River plan task for region {:?} offer {} finished without producing any river tiles", region_ent, offer_i);
                if let Ok(mut claimlist) = claim_queries.claimlists.get_mut(region_ent) {
                claimlist.clear_pending_i(offer_i as usize);
                claimlist.skipped_is.insert(offer_i as usize);
            }
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            *river_debug.region_mut(dimension_ref, region_pos) = region_debug;
            river_debug.bump_revision();
            probes_completed = probes_completed.saturating_add(1);
            offers_skipped = offers_skipped.saturating_add(1);
            return false;
        }

        let emitted_claim = queue_claim_for_offer_from_plan(
            &plan,
            claims_to_emit,
            region_ent,
            sgc_ent,
            offer_i,
        );
        if emitted_claim {
            cmd.entity(region_ent)
                .try_insert(plan)
                .try_remove::<RiverPendingOffer>();
            claims_emitted = claims_emitted.saturating_add(1);
        } else {
            error!(target: RIVER_SYSTEM, "River plan for region {:?} offer {} generated tiles but no claim chunks could be emitted", region_ent, offer_i);
            if let Ok(mut claimlist) = claim_queries.claimlists.get_mut(region_ent) {
                claimlist.clear_pending_i(offer_i as usize);
                claimlist.skipped_is.insert(offer_i as usize);
            }
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            offers_skipped = offers_skipped.saturating_add(1);
        }
        *river_debug.region_mut(dimension_ref, region_pos) = region_debug;
        river_debug.bump_revision();
            if let Ok(mut claimlist) = claim_queries.claimlists.get_mut(region_ent) {
            claimlist.clear_pending_i(offer_i as usize);
        }
        probes_completed = probes_completed.saturating_add(1);
        false
    });

    if offered_chunks.is_empty() && sampled_values_reader.is_empty() && search_failed_reader.is_empty() {
        for claim in claims_to_emit.drain(..) {
            claims_writer.write(claim);
        }
        return;
    }

    let Some(settings) = claim_queries.settings_q.single().ok() else {
        error!(target: RIVER_SYSTEM, "Missing GlobalGenSettings for river claim pass");
        return;
    };
    let Ok(region_probe_templ_ent) = terrprobe_entity_map.0.get_cloned(RIVER_REGION_PROBE_ID) else {
        error!(target: RIVER_SYSTEM, "Missing terrprobe template '{}'", RIVER_REGION_PROBE_ID);
        return;
    };
    let Ok(region_probe_templ) = claim_queries.terrprobe_query.get(region_probe_templ_ent) else {
        error!(target: RIVER_SYSTEM, "Terrprobe '{}' missing TerrProbeTempl", RIVER_REGION_PROBE_ID);
        return;
    };

    for failed in search_failed_reader.read() {
        let Ok(req) = claim_queries.probe_requests.get(failed.0) else {
            continue;
        };
        let Ok((dimension_ref, region_pos)) = claim_queries.region_info.get(req.region_ent) else {
            error!(target: RIVER_SYSTEM, "Probe failure for unknown region entity {:?}", req.region_ent);
            cmd.entity(req.region_ent).try_remove::<RiverPendingOffer>();
            cmd.entity(failed.0).try_despawn();
            continue;
        };
        clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, req.region_ent, req.offer_i);
        river_debug.region_mut(*dimension_ref, *region_pos).failure_count = river_debug
            .data
            .get(&(*dimension_ref, *region_pos))
            .map(|info| info.failure_count.saturating_add(1))
            .unwrap_or(1);
        let region_debug = river_debug.region_mut(*dimension_ref, *region_pos);
        region_debug.failed_chunks.insert(req.start_chunk);
        region_debug.failed_probe_points.insert(req.start_chunk.center_gpos());
        river_debug.mark_probe_finished(*dimension_ref, *region_pos, req.start_chunk);
            skipped_offers.push((req.region_ent, req.offer_i as usize));
        offers_skipped = offers_skipped.saturating_add(1);
        cmd.entity(req.region_ent).try_remove::<RiverPendingOffer>();
        cmd.entity(failed.0).try_despawn();
    }

    for sampled_values in sampled_values_reader.read() {
        sampled_events = sampled_events.saturating_add(1);
        let Ok(req) = claim_queries.probe_requests.get(sampled_values.requester) else {
            error!(target: RIVER_SYSTEM, "SampledValuesCollected for unknown requester {:?}", sampled_values.requester);
            continue;
        };
        let Ok(mut pending) = claim_queries.pending_offers.get_mut(req.region_ent) else {
            error!(target: RIVER_SYSTEM, "SampledValuesCollected for {:?} had no pending RiverPendingOffer on region {:?}", sampled_values.requester, req.region_ent);
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        };
        let region_ent = pending.region_ent;
        let Ok((dimension_ref, region_pos)) = claim_queries.region_info.get(region_ent) else {
            error!(target: RIVER_SYSTEM, "SampledValuesCollected for request with unknown region {:?}", region_ent);
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        };

        let matrix = &sampled_values.matrix;
        if matrix.values.iter().all(|value| value.is_none()) {
            {
                let info = river_debug.region_mut(*dimension_ref, *region_pos);
                error!(target: RIVER_SYSTEM, "River probe for region {:?} offer {} produced no sampled values", region_ent, pending.offer_i);
                info.failure_count = info.failure_count.saturating_add(1);
                info.failed_chunks.insert(req.start_chunk);
                info.failed_probe_points.insert(req.start_chunk.center_gpos());
            }
            river_debug.mark_probe_finished(*dimension_ref, *region_pos, req.start_chunk);
            let offer_i = pending.offer_i;
            let _ = pending;
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        }

        {
            let info = river_debug.region_mut(*dimension_ref, *region_pos);
            info.success_count = info.success_count.saturating_add(1);
            info.failed_chunks.remove(&req.start_chunk);
            for (sample_pos, sample_val_opt) in matrix.iter() {
                if let Some(sample_val) = sample_val_opt {
                    info.sampled_points.insert(sample_pos, sample_val);
                } else {
                    info.failed_probe_points.insert(sample_pos);
                }
            }
        }
        sampled_points_total = sampled_points_total.saturating_add(matrix.values.iter().filter(|value| value.is_some()).count() as u32);

        pending.inland_requester = Some(sampled_values.requester);

        river_debug.mark_probe_finished(*dimension_ref, *region_pos, req.start_chunk);
        cmd.entity(sampled_values.requester).try_despawn();
        let sgc_ent = pending.sgc_ent;
        let offer_i = pending.offer_i;
        let _ = pending;

        let Some(cfg) = claim_queries.structured_gens.get(sgc_ent).ok() else {
            error!(target: RIVER_SYSTEM, "Missing StructuredGenConfig {:?} for river offer {}", sgc_ent, offer_i);
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };

        let mut inland_map = HashMap::default();
        let mut coast_points = HashSet::default();
        for (pos, val_opt) in matrix.iter() {
            let Some(val) = val_opt else {
                coast_points.insert(pos);
                continue;
            };
            inland_map.insert(pos, val);
        }
        if inland_map.is_empty() {
            error!(target: RIVER_SYSTEM, "River probe for region {:?} offer {} produced inland samples but no usable inland_map entries", region_ent, offer_i);
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        }
        sampled_points_total = sampled_points_total.saturating_add(inland_map.len() as u32);
        let dimension_ref = *dimension_ref;
        let region_pos = *region_pos;
        let mut region_debug_snapshot = river_debug.region_mut(dimension_ref, region_pos).clone();
        let cfg = cfg.clone();
        let settings = settings.clone();
        let task_pool = AsyncComputeTaskPool::get();
        pending_plan_tasks.push(task_pool.spawn(async move {
            let mut plan = RiverRegionPlan::default();
            let _ = generate_river_region_plan(
                &inland_map,
                &coast_points,
                &cfg,
                &settings,
                dimension_ref,
                region_pos,
                &mut plan,
                &mut region_debug_snapshot,
            );
            RiverPlanTaskResult {
                region_ent,
                sgc_ent,
                offer_i,
                dimension_ref,
                region_pos,
                region_debug: region_debug_snapshot,
                plan,
            }
        }));
    }

    for offer in offered_chunks.read() {
        river_offers_seen = river_offers_seen.saturating_add(1);
        let Ok(cfg) = claim_queries.structured_gens.get(offer.structured_gen_cfg_ent) else {
            skipped_offers.push((offer.region_ent, offer.i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }

        let Ok(mut claimlist) = claim_queries.claimlists.get_mut(offer.region_ent) else {
            trace!(target: RIVER_SYSTEM, "Offer {}: missing ClaimList on region {:?}, skipping", offer.i, offer.region_ent);
            continue;
        };
        let offer_i = offer.i as usize;
        if claimlist.skipped_is.contains(&offer_i) || claimlist.pending_is.contains(&offer_i) {
            continue;
        }

        let Ok((dimension_ref, _)) = claim_queries.region_info.get(offer.region_ent) else {
            error!(target: RIVER_SYSTEM, "Offer {} missing region metadata on entity {:?}", offer.i, offer.region_ent);
            claimlist.skipped_is.insert(offer_i);
            skipped_offers.push((offer.region_ent, offer_i));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };
        let region_pos = offer.start_pos.to_region_pos();
        let region_key = (*dimension_ref, region_pos);
        let Some(&loaded_region_ent) = loaded_regions.0.get(&region_key) else {
            claimlist.skipped_is.insert(offer_i);
            skipped_offers.push((offer.region_ent, offer_i));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };

        if let Ok(plan) = claim_queries.region_plans.get(loaded_region_ent) {
            if queue_claim_for_offer_from_plan(
                plan,
                claims_to_emit,
                offer.region_ent,
                offer.structured_gen_cfg_ent,
                offer.i,
            ) {
                claims_emitted = claims_emitted.saturating_add(1);
            } else {
                claimlist.skipped_is.insert(offer_i);
                skipped_offers.push((offer.region_ent, offer_i));
                offers_skipped = offers_skipped.saturating_add(1);
            }
            continue;
        }
        if claim_queries.pending_offers.get(loaded_region_ent).is_ok() || pending_river_regions.contains(&loaded_region_ent) {
            claimlist.skipped_is.insert(offer_i);
            skipped_offers.push((offer.region_ent, offer_i));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        }

        let mut pending = RiverPendingOffer::new(loaded_region_ent, offer.structured_gen_cfg_ent, offer.i, offer.start_pos);
        let inland_requester = cmd.spawn((
            RiverProbeRequest {
                region_ent: loaded_region_ent,
                sgc_ent: offer.structured_gen_cfg_ent,
                offer_i: offer.i,
                start_chunk: offer.start_pos,
            },
            ChildOf(loaded_region_ent),
        )).id();
        pending.inland_requester = Some(inland_requester);
        cmd.entity(loaded_region_ent).try_insert(pending);
        pending_river_regions.insert(loaded_region_ent);
        claimlist.mark_pending_i(offer_i);

        let center = offer.start_pos.center_gpos();
        let mut probe = region_probe_templ.to_probe(region_probe_templ_ent, *dimension_ref, center);
        probe.requester = inland_requester;
        terrprobe_writer.write(probe);
        river_debug.mark_probe_started(*dimension_ref, region_pos, offer.start_pos);
        probes_spawned = probes_spawned.saturating_add(1);
    }

    for (region_ent, offer_i) in skipped_offers.drain(..) {
        let Ok(mut claimlist) = claim_queries.claimlists.get_mut(region_ent) else {
            continue;
        };
        claimlist.clear_pending_i(offer_i);
        claimlist.skipped_is.insert(offer_i);
    }

    for claim in claims_to_emit.drain(..) {
        claims_writer.write(claim);
    }

    if river_offers_seen > 0 || probes_spawned > 0 || sampled_events > 0 || probes_completed > 0 {
        info!(
            target: RIVER_SYSTEM,
            "claim pass summary: river_offers_seen={}, probes_spawned={}, sampled_events={}, sampled_points_total={}, probes_completed={}, claims_emitted={}, offers_skipped={}",
            river_offers_seen,
            probes_spawned,
            sampled_events,
            sampled_points_total,
            probes_completed,
            claims_emitted,
            offers_skipped
        );
    }
}

fn clear_claimlist_pending_offer_i(
    claimlists: &mut Query<&mut ClaimList>,
    region_ent: Entity,
    offer_i: u64,
) {
    let Ok(mut claimlist) = claimlists.get_mut(region_ent) else {
        return;
    };
    claimlist.clear_pending_i(offer_i as usize);
}



fn queue_claim_for_offer_from_plan(
    plan: &RiverRegionPlan,
    claims_to_emit: &mut Vec<ChunksClaim>,
    region_ent: Entity,
    sgc_ent: Entity,
    offer_i: u64,
) -> bool {
    let Some(chunks_pos) = plan.sorted_claimed_chunks() else {
        return false;
    };
    claims_to_emit.push(ChunksClaim {
        i: offer_i,
        region_ent,
        sgc_ent,
        chunks_pos,
        partition_tolerant: true,
    });
    true
}


