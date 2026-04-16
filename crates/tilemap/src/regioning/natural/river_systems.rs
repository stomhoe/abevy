use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::{common_components::HashId, log_targets::RIVER_SYSTEM};
use tilemap_shared::{ChunkPos, DimensionRef, GlobalGenSettings, GlobalTilePos, HashablePosVec, RegionPos, SizeInTiles};

use crate::{
    regioning::{
        regioning_components::ClaimList,
        regioning_messages::{ChunksClaim, OfferChunk, SgcPrepareTilesOrder, StructureBuildCompliance, TerrGenDisabledGposForChunks},
        regioning_resources::LoadedRegions,
        regioning_sgc_components::StructuredGenConfig,
    },
    terrain::terrprobe::{
        terrprobe_components::TerrProbeTempl,
        terrprobe_messages::{SampledValuesCollected, SearchFailed, TerrProbeJob},
        terrprobe_resources::TerrProbeTemplEntityMap,
    },
    terrain::terrgen_async_resources::TerrGenBlockedGposMask,
    tile::tile_resources::{TileEntityMap, TileRef},
};

use super::river_components::{
    RiverDebugData, RiverPendingOffer, RiverProbeKind, RiverProbeRequest, RiverRegionPlan,
};
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

    let mut river_offers_seen = 0_u32;
    let mut probes_spawned = 0_u32;
    let mut sampled_events = 0_u32;
    let mut sampled_points_total = 0_u32;
    let mut probes_completed = 0_u32;
    let mut claims_emitted = 0_u32;
    let mut offers_skipped = 0_u32;

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
        claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
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
        let info = river_debug.region_mut(*dimension_ref, *region_pos);
        if matrix.values.iter().all(|value| value.is_none()) {
            error!(target: RIVER_SYSTEM, "River probe for region {:?} offer {} produced no sampled values", region_ent, pending.offer_i);
            info.failure_count = info.failure_count.saturating_add(1);
            info.failed_chunks.insert(req.start_chunk);
            info.failed_probe_points.insert(req.start_chunk.center_gpos());
            river_debug.mark_probe_finished(*dimension_ref, *region_pos, req.start_chunk);
            let offer_i = pending.offer_i;
            let _ = pending;
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            claim_state.skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        }

        info.success_count = info.success_count.saturating_add(1);
        info.failed_chunks.remove(&req.start_chunk);
        for (sample_pos, sample_val_opt) in matrix.iter() {
            if let Some(sample_val) = sample_val_opt {
                info.sampled_points.insert(sample_pos, sample_val);
            }
        }
        sampled_points_total = sampled_points_total.saturating_add(matrix.values.iter().filter(|value| value.is_some()).count() as u32);

        match req.probe_kind {
            RiverProbeKind::Inlandness => {
                pending.inland_requester = Some(sampled_values.requester);
            }
        }

        river_debug.mark_probe_finished(*dimension_ref, *region_pos, req.start_chunk);
        cmd.entity(sampled_values.requester).try_despawn();
        let sgc_ent = pending.sgc_ent;
        let offer_i = pending.offer_i;
        let _ = pending;

        let Some(cfg) = claim_queries.structured_gens.get(sgc_ent).ok() else {
            error!(target: RIVER_SYSTEM, "Missing StructuredGenConfig {:?} for river offer {}", sgc_ent, offer_i);
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            claim_state.skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };

        let river_mouth_max_inlandness: f32 = cfg.args.parse_arg("river_mouth_max_inlandness", 0.05);
        let mut inland_map = HashMap::default();
        let mut below_max_inlandness_points = HashSet::default();
        for (pos, val_opt) in matrix.iter() {
            let Some(val) = val_opt else {
                continue;
            };
            inland_map.insert(pos, val);
            if val <= river_mouth_max_inlandness {
                below_max_inlandness_points.insert(pos);
            }
        }
        if inland_map.is_empty() {
            error!(target: RIVER_SYSTEM, "River probe for region {:?} offer {} produced inland samples but no usable inland_map entries", region_ent, offer_i);
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            claim_state.skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        }
        sampled_points_total = sampled_points_total.saturating_add(inland_map.len() as u32);
        let mut plan = RiverRegionPlan::default();
        let generated_ok = generate_river_region_plan(
            &inland_map,
            &below_max_inlandness_points,
            cfg,
            settings,
            *dimension_ref,
            *region_pos,
            &mut plan,
            &mut river_debug,
        );
        river_debug.bump_revision();
        if !generated_ok {
            clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
            claim_state.skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            cmd.entity(sampled_values.requester).try_despawn();
            continue;
        }
        let emitted_claim = queue_claim_for_offer_from_plan(
            &plan,
            &mut claim_state.claims_to_emit,
            region_ent,
            sgc_ent,
            offer_i,
        );
        if emitted_claim {
            cmd.entity(region_ent).try_insert(plan);
            cmd.entity(region_ent).try_remove::<RiverPendingOffer>();
            claims_emitted = claims_emitted.saturating_add(1);
        } else {
            error!(target: RIVER_SYSTEM, "River plan for region {:?} offer {} generated tiles but no claim chunks could be emitted", region_ent, offer_i);
            claim_state.skipped_offers.push((region_ent, offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
        }
        clear_claimlist_pending_offer_i(&mut claim_queries.claimlists, region_ent, offer_i);
        probes_completed = probes_completed.saturating_add(1);
    }

    for offer in offered_chunks.read() {
        river_offers_seen = river_offers_seen.saturating_add(1);
        let Ok(cfg) = claim_queries.structured_gens.get(offer.structured_gen_cfg_ent) else {
            claim_state.skipped_offers.push((offer.region_ent, offer.i as usize));
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
            claim_state.skipped_offers.push((offer.region_ent, offer_i));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };
        let region_pos = offer.start_pos.to_region_pos();
        let region_key = (*dimension_ref, region_pos);
        let Some(&loaded_region_ent) = loaded_regions.0.get(&region_key) else {
            claimlist.skipped_is.insert(offer_i);
            claim_state.skipped_offers.push((offer.region_ent, offer_i));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };

        if let Ok(plan) = claim_queries.region_plans.get(loaded_region_ent) {
            if queue_claim_for_offer_from_plan(
                plan,
                &mut claim_state.claims_to_emit,
                offer.region_ent,
                offer.structured_gen_cfg_ent,
                offer.i,
            ) {
                claims_emitted = claims_emitted.saturating_add(1);
            } else {
                claimlist.skipped_is.insert(offer_i);
                claim_state.skipped_offers.push((offer.region_ent, offer_i));
                offers_skipped = offers_skipped.saturating_add(1);
            }
            continue;
        }
        if claim_queries.pending_offers.get(loaded_region_ent).is_ok() || claim_state.pending_river_regions.contains(&loaded_region_ent) {
            claimlist.skipped_is.insert(offer_i);
            claim_state.skipped_offers.push((offer.region_ent, offer_i));
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
                probe_kind: RiverProbeKind::Inlandness,
            },
            ChildOf(loaded_region_ent),
        )).id();
        pending.inland_requester = Some(inland_requester);
        cmd.entity(loaded_region_ent).try_insert(pending);
        claim_state.pending_river_regions.insert(loaded_region_ent);
        claimlist.mark_pending_i(offer_i);

        let center = offer.start_pos.center_gpos();
        let mut probe = region_probe_templ.to_probe(region_probe_templ_ent, *dimension_ref, center);
        probe.requester = inland_requester;
        terrprobe_writer.write(probe);
        river_debug.mark_probe_started(*dimension_ref, region_pos, offer.start_pos);
        probes_spawned = probes_spawned.saturating_add(1);
    }

    for (region_ent, offer_i) in claim_state.skipped_offers.drain(..) {
        let Ok(mut claimlist) = claim_queries.claimlists.get_mut(region_ent) else {
            continue;
        };
        claimlist.clear_pending_i(offer_i);
        claimlist.skipped_is.insert(offer_i);
    }

    for claim in claim_state.claims_to_emit.drain(..) {
        claim_state.claims_writer.write(claim);
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

#[allow(unused_parens)]
pub fn river_structure_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<&StructuredGenConfig>,
    tiles_map: Res<TileEntityMap>,
    loaded_regions: Res<LoadedRegions>,
    region_plans: Query<&RiverRegionPlan>,
    mut writer: MessageWriter<StructureBuildCompliance>,
) {
    let mut compliances_to_emit = Vec::new();
    let mut total_orders = 0_u32;
    let mut river_orders = 0_u32;
    let mut emitted_chunks_total = 0_usize;
    let mut emitted_tiles_total = 0_usize;
    let mut generated_tiles_total = 0_usize;
    let mut blocked_gpos_total = 0_usize;
    for order in reader.read() {
        total_orders = total_orders.saturating_add(1);
        let Ok(cfg) = structured_gens.get(order.structured_gen_cfg_ent) else {
            error!(target: RIVER_SYSTEM, "Order {}: missing StructuredGenConfig {:?}", order.i, order.structured_gen_cfg_ent);
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }
        river_orders = river_orders.saturating_add(1);
        info!(target: RIVER_SYSTEM, "Order {}: river build start for region {:?} dim {:?}, claimed chunks={}", order.i, order.region_pos, order.dimension_ref, order.chunks_pos.len());

        let mut compliance = StructureBuildCompliance {
            i: order.i,
            structure_gen_cfg_ent: order.structured_gen_cfg_ent,
            dimension_ref: order.dimension_ref,
            chunks: Vec::new(),
            terrgen_disabled_gpos_for_chunks: TerrGenDisabledGposForChunks::default(),
            terrgen_disabled_for_chunks: Vec::new(),
            forced_chunk_biomes: Vec::new(),
        };

        let river_tile_id = cfg.args
            .get("river_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("blue"));
        let Ok(_river_tile_ent) = tiles_map.0.get_cloned(river_tile_id) else {
            error!(target: RIVER_SYSTEM, "Missing river_tile_id {:?} for cfg {:?}", river_tile_id, order.structured_gen_cfg_ent);
            compliances_to_emit.push(compliance);
            continue;
        };
        let river_tile_ref = TileRef(river_tile_id);
        let river_tile_size = SizeInTiles::default().inner();

        let Some(&region_ent) = loaded_regions.0.get(&(order.dimension_ref, order.region_pos)) else {
            error!(target: RIVER_SYSTEM, "Order {}: no loaded region entity for region {:?} dim {:?}", order.i, order.region_pos, order.dimension_ref);
            compliances_to_emit.push(compliance);
            continue;
        };
        let Ok(plan) = region_plans.get(region_ent) else {
            error!(target: RIVER_SYSTEM, "Order {}: no cached RiverRegionPlan for region {:?} dim {:?}", order.i, order.region_pos, order.dimension_ref);
            compliances_to_emit.push(compliance);
            continue;
        };
        let generated_tiles = plan.iter_river_tiles_sorted();
        let generated_count = generated_tiles.len();
        generated_tiles_total = generated_tiles_total.saturating_add(generated_count);

        let claimed_chunks: HashSet<ChunkPos> = order.chunks_pos.iter().copied().collect();
        let mut tiles_by_chunk: HashMap<ChunkPos, Vec<(GlobalTilePos, TileRef, Option<tilemap_shared::DeleteOtherTilesInSamePos>)>> = HashMap::default();
        let mut terrgen_disabled_gpos_for_chunks = TerrGenDisabledGposForChunks::default();
        for gpos in generated_tiles.iter().copied() {
            let chunk_pos = gpos.to_chunkpos();
            if !claimed_chunks.contains(&chunk_pos) {
                continue;
            }
            tiles_by_chunk
                .entry(chunk_pos)
                .or_default()
                .push((gpos, river_tile_ref, None));
        }

        let mut chunks: Vec<_> = tiles_by_chunk.into_iter().collect();
        chunks.sort_by_key(|(chunk, _)| (chunk.y(), chunk.x()));
        let emitted_chunk_count = chunks.len();
        let emitted_tile_count = chunks.iter().map(|(_, tiles)| tiles.len()).sum::<usize>();
        let tiles_outside_claimed = generated_count.saturating_sub(emitted_tile_count);
        for (chunk_pos, tiles) in &chunks {
            let mut blocked_gpos = TerrGenBlockedGposMask::default();
            for (tile_pos, _, _) in tiles {
                mark_occupied_gpos(&mut blocked_gpos, *chunk_pos, *tile_pos, river_tile_size);
            }
            blocked_gpos_total = blocked_gpos_total.saturating_add(blocked_gpos.count_set());
            terrgen_disabled_gpos_for_chunks.insert_for_chunk(*chunk_pos, blocked_gpos);
        }
        emitted_chunks_total = emitted_chunks_total.saturating_add(emitted_chunk_count);
        emitted_tiles_total = emitted_tiles_total.saturating_add(emitted_tile_count);
        if emitted_chunk_count == 0 {
            error!(target: RIVER_SYSTEM, "Order {}: generated {} river tiles but emitted 0 chunks (claimed chunks={}, outside_claimed_tiles={})", order.i, generated_count, claimed_chunks.len(), tiles_outside_claimed);
        } else {
            info!(target: RIVER_SYSTEM, "Order {}: emitted {} chunks and {} river tiles (generated before clip={}, outside_claimed_tiles={})", order.i, emitted_chunk_count, emitted_tile_count, generated_count, tiles_outside_claimed);
        }
        compliance.chunks = chunks;
        compliance.terrgen_disabled_gpos_for_chunks = terrgen_disabled_gpos_for_chunks;

        compliances_to_emit.push(compliance);
    }
    if river_orders > 0 {
        info!(target: RIVER_SYSTEM, "river build summary: total_orders={}, river_orders={}, generated_tiles_total={}, emitted_chunks_total={}, emitted_tiles_total={}, blocked_gpos_total={}", total_orders, river_orders, generated_tiles_total, emitted_chunks_total, emitted_tiles_total, blocked_gpos_total);
    }
    writer.write_batch(compliances_to_emit);
}

fn queue_claim_for_offer_from_plan(
    plan: &RiverRegionPlan,
    claims_to_emit: &mut Vec<ChunksClaim>,
    region_ent: Entity,
    sgc_ent: Entity,
    offer_i: u64,
) -> bool {
    let Some(chunks_pos) = sorted_claimed_chunks_from_plan(plan) else {
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

fn sorted_claimed_chunks_from_plan(
    plan: &RiverRegionPlan,
) -> Option<Vec<ChunkPos>> {
    if plan.claimed_chunks.is_empty() {
        return None;
    }
    let mut chunks_pos = plan.claimed_chunks.iter().copied().collect::<Vec<_>>();
    chunks_pos.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
    Some(chunks_pos)
}


fn mark_occupied_gpos(
    blocked_gpos: &mut TerrGenBlockedGposMask,
    chunk_pos: ChunkPos,
    anchor_gpos: GlobalTilePos,
    size: UVec2,
) {
    for y in anchor_gpos.0.y..(anchor_gpos.0.y + size.y as i32) {
        for x in anchor_gpos.0.x..(anchor_gpos.0.x + size.x as i32) {
            blocked_gpos.set_gpos(chunk_pos, GlobalTilePos::new(x, y));
        }
    }
}