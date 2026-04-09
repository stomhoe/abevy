use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::{common_components::HashId, log_targets::RIVER_SYSTEM};
use game_common::game_common_components::TemplEntiRef;
use std::collections::VecDeque;
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
    tile::tile_resources::TileEntityMap,
};
use super::river_components::{
    RiverDebugData, RiverPlans, RiverProbeRequest, RiverRegisteredOffer,
};

const RIVER: HashId = HashId::hash("river");
const RIVER_REGION_PROBE_ID: &str = "river_region_probe";

#[derive(Default)]
struct GeneratedRiverNetwork {
    river_tiles: HashSet<GlobalTilePos>,
    river_source_points: HashSet<GlobalTilePos>,
    river_mouth_points: HashSet<GlobalTilePos>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct RiverClaimState<'w, 's> {
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
    probe_requests: Query<(Entity, &RiverProbeRequest)>,
    mut terrprobe_writer: MessageWriter<TerrProbeJob>,
    mut sampled_values_reader: MessageReader<SampledValuesCollected>,
    mut claimlists: Query<&mut ClaimList>,
    mut river_debug: ResMut<RiverDebugData>,
    mut river_plans: ResMut<RiverPlans>,
    mut claim_state: RiverClaimState,
) {
    claim_state.completed_probes.clear();
    claim_state.completed_with_samples.clear();
    claim_state.claims_to_emit.clear();
    claim_state.skipped_offers.clear();
    trace!(target: RIVER_SYSTEM, "claim_chunks_for_river_structures tick");
    let mut river_offers_seen = 0_u32;
    let mut probes_spawned = 0_u32;
    let mut sampled_events = 0_u32;
    let mut sampled_points_total = 0_u32;
    let mut probes_completed = 0_u32;
    let mut claims_emitted = 0_u32;
    let mut offers_skipped = 0_u32;

    let Ok(probe_templ_ent) = terrprobe_entity_map.0.get_cloned(RIVER_REGION_PROBE_ID) else {
        error!(target: RIVER_SYSTEM, "Missing terrprobe template id '{}'", RIVER_REGION_PROBE_ID);
        return;
    };
    let Ok(probe_templ) = terrprobe_query.get(probe_templ_ent) else {
        error!(target: RIVER_SYSTEM, "Terrprobe entity {:?} missing TerrProbeTempl", probe_templ_ent);
        return;
    };

    let mut regions_with_probe: HashSet<(DimensionRef, RegionPos)> = HashSet::default();
    for (_, req) in probe_requests.iter() {
        regions_with_probe.insert((req.dimension_ref, req.region_pos));
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
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }
        river_offers_seen = river_offers_seen.saturating_add(1);

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
        if let Some(existing_offer) = river_plans.registered_offer(dimension_ref, region_pos)
            && existing_offer.offer_i != offer.i
        {
            trace!(
                target: RIVER_SYSTEM,
                "Offer {}: region {:?} in dim {:?} already has registered river offer {}, skipping duplicate",
                offer.i,
                region_pos,
                dimension_ref,
                existing_offer.offer_i
            );
            claim_state.skipped_offers.push((offer.region_ent, offer.i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
            continue;
        }
        river_plans.register_offer(
            dimension_ref,
            region_pos,
            RiverRegisteredOffer {
                region_ent: offer.region_ent,
                sgc_ent: offer.structured_gen_cfg_ent,
                offer_i: offer.i,
            },
        );

        if river_plans.plan(dimension_ref, region_pos).is_some() {
            if queue_claim_from_plan(
                &mut river_plans,
                &mut claim_state.claims_to_emit,
                dimension_ref,
                region_pos,
            ) {
                claims_emitted = claims_emitted.saturating_add(1);
                info!(
                    target: RIVER_SYSTEM,
                    "Offer {}: reused cached river plan for region {:?} dim {:?}",
                    offer.i,
                    region_pos,
                    dimension_ref
                );
            } else {
                claim_state.skipped_offers.push((offer.region_ent, offer.i as usize));
                offers_skipped = offers_skipped.saturating_add(1);
            }
            continue;
        }

        if regions_with_probe.contains(&(dimension_ref, region_pos)) {
            trace!(
                target: RIVER_SYSTEM,
                "Offer {}: region {:?} in dim {:?} already has active probe",
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
                dimension_ref,
                sgc_ent: offer.structured_gen_cfg_ent,
                offer_i: offer.i,
                region_pos,
                start_chunk: offer.start_pos,
            })
            .id();

        let mut probe = probe_templ.to_probe(probe_templ_ent, dimension_ref, center);
        probe.requester = probe_ent;
        terrprobe_writer.write(probe);
        probes_spawned = probes_spawned.saturating_add(1);
        info!(
            target: RIVER_SYSTEM,
            "Offer {}: spawned river probe {:?} at center {:?}, region {:?}, dim {:?}",
            offer.i,
            probe_ent,
            center,
            region_pos,
            dimension_ref,
        );

        river_debug.mark_probe_started(dimension_ref, region_pos, offer.start_pos);
        regions_with_probe.insert((dimension_ref, region_pos));
    }

    for sampled_values in sampled_values_reader.read() {
        sampled_events = sampled_events.saturating_add(1);
        let Ok((_, req)) = probe_requests.get(sampled_values.requester) else {
            error!(
                target: RIVER_SYSTEM,
                "SampledValueMatrixFound for unknown requester {:?}, region_pos <unknown>",
                sampled_values.requester,
            );
            continue;
        };
        let dimension_ref = req.dimension_ref;
        let info = river_debug.region_mut(dimension_ref, req.region_pos);
        info.sampled_points.clear();
        info.sampled_none_points.clear();
        let mut matched_samples = 0_u32;
        for (sample_pos, sample_val_opt) in sampled_values.matrix.values.iter() {
            if let Some(sample_val) = sample_val_opt {
                info.sampled_points.insert(*sample_pos, *sample_val);
                matched_samples = matched_samples.saturating_add(1);
            } else {
                info.sampled_none_points.insert(*sample_pos);
            }
        }
        if matched_samples > 0 {
            sampled_points_total = sampled_points_total.saturating_add(matched_samples);
            claim_state.completed_with_samples.insert(sampled_values.requester);
            let sampled_points_len = {
                let info = river_debug.region_mut(dimension_ref, req.region_pos);
                info.success_count = info.success_count.saturating_add(1);
                info.failed_chunks.remove(&req.start_chunk);
                info.sampled_points.len()
            };
            info!(
                target: RIVER_SYSTEM,
                "Probe {:?} region {:?} dim {:?}: captured {} sampled points (total cached: {})",
                sampled_values.requester,
                req.region_pos,
                dimension_ref,
                matched_samples,
                sampled_points_len
            );
        } else {
            {
                let info = river_debug.region_mut(dimension_ref, req.region_pos);
                info.failure_count = info.failure_count.saturating_add(1);
                info.failed_chunks.insert(req.start_chunk);
            }
            warn!(
                target: RIVER_SYSTEM,
                "Probe {:?} region {:?} dim {:?}: sampled matrix contained 0 matched points",
                sampled_values.requester,
                req.region_pos,
                dimension_ref
            );
        }
        info!(
            target: RIVER_SYSTEM,
            "Requester {:?} region {:?} dim {:?}: received sampled matrix entries={}, matched_samples={}",
            sampled_values.requester,
            req.region_pos,
            dimension_ref,
            sampled_values.matrix.values.len(),
            matched_samples
        );

        if claim_state.completed_with_samples.contains(&sampled_values.requester) {
            let Ok(cfg) = structured_gens.get(req.sgc_ent) else {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: missing StructuredGenConfig {:?}",
                    sampled_values.requester,
                    req.region_pos,
                    dimension_ref,
                    req.sgc_ent
                );
                claim_state.completed_probes.insert(sampled_values.requester);
                continue;
            };
            let Ok(settings) = settings_q.single() else {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: missing GlobalGenSettings",
                    sampled_values.requester,
                    req.region_pos,
                    dimension_ref
                );
                claim_state.completed_probes.insert(sampled_values.requester);
                continue;
            };
            let sampled_points = river_debug
                .0
                .get(&(dimension_ref, req.region_pos))
                .map(|info| info.sampled_points.clone())
                .unwrap_or_default();
            let network = generate_river_network_for_region(
                &sampled_points,
                cfg,
                settings,
                dimension_ref.0,
                req.region_pos,
            );
            info!(
                target: RIVER_SYSTEM,
                "Probe {:?} region {:?} dim {:?}: generated_tiles={}",
                sampled_values.requester,
                req.region_pos,
                dimension_ref,
                network.river_tiles.len()
            );
            let touched_regions = cache_generated_river_network(
                dimension_ref,
                &network,
                &mut river_plans,
                &mut river_debug,
            );
            for target_region_pos in touched_regions {
                if queue_claim_from_plan(
                    &mut river_plans,
                    &mut claim_state.claims_to_emit,
                    dimension_ref,
                    target_region_pos,
                ) {
                    claims_emitted = claims_emitted.saturating_add(1);
                }
            }
        }
        claim_state.completed_probes.insert(sampled_values.requester);
    }

    for ent in claim_state.completed_probes.drain() {
        let Ok((_, req)) = probe_requests.get(ent) else {
            cmd.entity(ent).despawn();
            continue;
        };
        let dimension_ref = req.dimension_ref;
        probes_completed = probes_completed.saturating_add(1);
        if river_plans.plan(dimension_ref, req.region_pos).is_some()
            && queue_claim_from_plan(
                &mut river_plans,
                &mut claim_state.claims_to_emit,
                dimension_ref,
                req.region_pos,
            )
        {
            claims_emitted = claims_emitted.saturating_add(1);
        } else if river_plans.registered_offer(dimension_ref, req.region_pos).is_some() {
            if claim_state.completed_with_samples.contains(&ent) {
                warn!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: completed with no local/imported river plan, skipping offer {}",
                    ent,
                    req.region_pos,
                    dimension_ref,
                    req.offer_i
                );
            } else {
                warn!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: completed with no sampled values, skipping offer {}",
                    ent,
                    req.region_pos,
                    dimension_ref,
                    req.offer_i
                );
            }
            river_debug
                .region_mut(dimension_ref, req.region_pos)
                .failed_chunks
                .insert(req.start_chunk);
            river_plans.registered_offers.remove(&(dimension_ref, req.region_pos));
            claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
        }
        river_debug.mark_probe_finished(dimension_ref, req.region_pos, req.start_chunk);
        cmd.entity(ent).despawn();
    }

    for (region_ent, offer_i) in claim_state.skipped_offers.drain(..) {
        let Ok(mut claimlist) = claimlists.get_mut(region_ent) else {
            continue;
        };
        claimlist.skipped_is.insert(offer_i);
    }
    claim_state.claims_writer.write_batch(claim_state.claims_to_emit.drain(..));

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

#[allow(unused_parens)]
pub fn river_structure_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<&StructuredGenConfig>,
    tiles_map: Res<TileEntityMap>,
    river_plans: Res<RiverPlans>,
    mut river_debug: ResMut<RiverDebugData>,
    mut writer: MessageWriter<StructureBuildCompliance>,
) {
    let mut compliances_to_emit = Vec::new();
    let mut total_orders = 0_u32;
    let mut river_orders = 0_u32;
    let mut emitted_chunks_total = 0_usize;
    let mut emitted_tiles_total = 0_usize;
    let mut generated_tiles_total = 0_usize;
    for order in reader.read() {
        total_orders = total_orders.saturating_add(1);
        let Ok(cfg) = structured_gens.get(order.structured_gen_cfg_ent) else {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: missing StructuredGenConfig {:?}",
                order.i,
                order.structured_gen_cfg_ent
            );
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }
        river_orders = river_orders.saturating_add(1);
        info!(
            target: RIVER_SYSTEM,
            "Order {}: river build start for region {:?} dim {:?}, claimed chunks={}",
            order.i,
            order.region_pos,
            order.dimension_ref,
            order.chunks_pos.len()
        );

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
        let Ok(river_tile_ent) = tiles_map.0.get_cloned(river_tile_id) else {
            error!(
                target: RIVER_SYSTEM,
                "Missing river_tile_id {:?} for cfg {:?}",
                river_tile_id,
                order.structured_gen_cfg_ent
            );
            compliances_to_emit.push(compliance);
            continue;
        };
        let river_tile_ref = TemplEntiRef(river_tile_ent);

        let Some(plan) = river_plans.plan(order.dimension_ref, order.region_pos) else {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: no cached RiverRegionPlan for region {:?} dim {:?}",
                order.i,
                order.region_pos,
                order.dimension_ref
            );
            compliances_to_emit.push(compliance);
            continue;
        };
        let generated_tiles = plan.river_tiles.clone();
        let river_sources = plan.river_source_points.clone();
        let river_mouths = plan.river_mouth_points.clone();
        let generated_count = generated_tiles.len();
        generated_tiles_total = generated_tiles_total.saturating_add(generated_count);

        let claimed_chunks: HashSet<ChunkPos> = order.chunks_pos.iter().copied().collect();
        let mut tiles_by_chunk: HashMap<ChunkPos, Vec<(GlobalTilePos, TemplEntiRef, Option<tilemap_shared::DeleteOtherTilesInSamePos>)>> =
            HashMap::default();
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
        emitted_chunks_total = emitted_chunks_total.saturating_add(emitted_chunk_count);
        emitted_tiles_total = emitted_tiles_total.saturating_add(emitted_tile_count);
        if emitted_chunk_count == 0 {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: generated {} river tiles but emitted 0 chunks (claimed chunks={}, outside_claimed_tiles={})",
                order.i,
                generated_count,
                claimed_chunks.len(),
                tiles_outside_claimed
            );
        } else {
            info!(
                target: RIVER_SYSTEM,
                "Order {}: emitted {} chunks and {} river tiles (generated before clip={}, outside_claimed_tiles={})",
                order.i,
                emitted_chunk_count,
                emitted_tile_count,
                generated_count,
                tiles_outside_claimed
            );
        }
        compliance.chunks = chunks;

        river_debug.clear_generated_river(order.dimension_ref, order.region_pos);
        let info = river_debug.region_mut(order.dimension_ref, order.region_pos);
        info.claimed_chunks.extend(order.chunks_pos.iter().copied());
        info.river_tiles.extend(generated_tiles.into_iter().filter(|p| {
            claimed_chunks.contains(&p.to_chunkpos())
        }));
        info.river_source_points.extend(river_sources);
        info.river_mouth_points.extend(river_mouths);

        compliances_to_emit.push(compliance);
    }
    if river_orders > 0 {
        info!(
            target: RIVER_SYSTEM,
            "river build summary: total_orders={}, river_orders={}, generated_tiles_total={}, emitted_chunks_total={}, emitted_tiles_total={}",
            total_orders,
            river_orders,
            generated_tiles_total,
            emitted_chunks_total,
            emitted_tiles_total
        );
    }
    writer.write_batch(compliances_to_emit);
}

fn queue_claim_from_plan(
    river_plans: &mut RiverPlans,
    claims_to_emit: &mut Vec<ChunksClaim>,
    dimension_ref: DimensionRef,
    region_pos: RegionPos,
) -> bool {
    let mut chunks_pos = {
        let Some(plan) = river_plans.plan(dimension_ref, region_pos) else {
            return false;
        };
        if plan.claimed_chunks.is_empty() {
            return false;
        }
        plan.claimed_chunks.iter().copied().collect::<Vec<_>>()
    };
    let Some(offer) = river_plans.registered_offers.remove(&(dimension_ref, region_pos)) else {
        return false;
    };
    chunks_pos.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
    claims_to_emit.push(ChunksClaim {
        i: offer.offer_i,
        region_ent: offer.region_ent,
        sgc_ent: offer.sgc_ent,
        chunks_pos,
        partition_tolerant: true,
    });
    true
}

fn cache_generated_river_network(
    dimension_ref: DimensionRef,
    network: &GeneratedRiverNetwork,
    river_plans: &mut RiverPlans,
    river_debug: &mut RiverDebugData,
) -> Vec<RegionPos> {
    let mut tiles_by_region: HashMap<RegionPos, HashSet<GlobalTilePos>> = HashMap::default();
    let mut sources_by_region: HashMap<RegionPos, HashSet<GlobalTilePos>> = HashMap::default();
    let mut mouths_by_region: HashMap<RegionPos, HashSet<GlobalTilePos>> = HashMap::default();

    for &tile in network.river_tiles.iter() {
        tiles_by_region
            .entry(tile.to_chunkpos().to_region_pos())
            .or_default()
            .insert(tile);
    }
    for &source in network.river_source_points.iter() {
        sources_by_region
            .entry(source.to_chunkpos().to_region_pos())
            .or_default()
            .insert(source);
    }
    for &mouth in network.river_mouth_points.iter() {
        mouths_by_region
            .entry(mouth.to_chunkpos().to_region_pos())
            .or_default()
            .insert(mouth);
    }

    let mut touched_regions: HashSet<RegionPos> = HashSet::default();
    touched_regions.extend(tiles_by_region.keys().copied());
    touched_regions.extend(sources_by_region.keys().copied());
    touched_regions.extend(mouths_by_region.keys().copied());

    let mut touched_regions_vec = Vec::with_capacity(touched_regions.len());
    for region_pos in touched_regions {
        let region_tiles = tiles_by_region.remove(&region_pos).unwrap_or_default();
        let region_sources = sources_by_region.remove(&region_pos).unwrap_or_default();
        let region_mouths = mouths_by_region.remove(&region_pos).unwrap_or_default();

        let plan = river_plans
            .plans_by_region
            .entry((dimension_ref, region_pos))
            .or_default();
        plan.river_tiles.extend(region_tiles.iter().copied());
        plan.river_source_points.extend(region_sources.iter().copied());
        plan.river_mouth_points.extend(region_mouths.iter().copied());
        for tile in region_tiles.iter().copied() {
            plan.claimed_chunks.insert(tile.to_chunkpos());
        }

        let info = river_debug.region_mut(dimension_ref, region_pos);
        info.river_tiles.extend(region_tiles);
        info.river_source_points.extend(region_sources);
        info.river_mouth_points.extend(region_mouths);

        touched_regions_vec.push(region_pos);
    }
    touched_regions_vec
}

fn generate_river_network_for_region(
    sampled_points: &HashMap<GlobalTilePos, f32>,
    cfg: &StructuredGenConfig,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source_region_pos: RegionPos,
) -> GeneratedRiverNetwork {
    if sampled_points.is_empty() {
        return GeneratedRiverNetwork::default();
    }
    let land_threshold: f32 = cfg.args.parse_arg("river_land_threshold", -0.05);
    let source_min_inlandness: f32 = cfg
        .args
        .parse_arg("river_source_min_inlandness", (land_threshold + 0.2).max(land_threshold));
    let mouth_max_inlandness: f32 = cfg.args.parse_arg("river_mouth_max_inlandness", 0.05);
    let max_sources: usize = cfg.args.parse_arg("river_max_sources", 5_usize).max(1);
    let max_steps: usize = cfg.args.parse_arg("river_worm_length", 240_usize).max(8);
    let source_stride: u64 = cfg.args.parse_arg("river_source_hash_stride", 19_u64).max(1);
    let source_mouth_min_distance: usize = cfg.args.parse_arg("river_source_mouth_min_distance", 8_usize).max(1);
    let half_width_start: i32 = cfg.args.parse_arg("river_main_half_width_start", 1_i32).max(0);
    let half_width_end: i32 = cfg.args.parse_arg("river_main_half_width_end", 3_i32).max(0);
    let min_island_area_chunks: f32 = cfg.args.parse_arg("river_min_island_area_chunks", 225.0_f32).max(1.0);

    let spacing = estimate_sample_spacing(sampled_points);
    let (component_of, component_sizes) = build_land_components(sampled_points, land_threshold, spacing);
    let sample_area_chunks = ((spacing as f32 / ChunkPos::CHUNK_SIZE.x as f32)
        * (spacing as f32 / ChunkPos::CHUNK_SIZE.y as f32))
        .max(0.001);
    let min_component_samples = (min_island_area_chunks / sample_area_chunks).ceil() as usize;
    let source_min_separation_steps: usize = cfg
        .args
        .parse_arg("river_source_min_separation_steps", 10_usize)
        .max(1);
    let min_source_distance_tiles = (spacing * source_min_separation_steps as i32).max(spacing);
    let sources = pick_river_sources(
        sampled_points,
        &component_of,
        &component_sizes,
        min_component_samples.max(1),
        source_min_inlandness,
        spacing,
        settings,
        dimension_hash,
        source_stride,
        max_sources,
        min_source_distance_tiles,
        Some(source_region_pos),
    );
    let trace_params = RiverTraceParams {
        neighbor_radius: cfg.args.parse_arg("river_trace_neighbor_radius", 2_i32).clamp(1, 4),
        mouth_max_inlandness,
        min_steps_before_mouth: source_mouth_min_distance,
        directional_inertia: cfg
            .args
            .parse_arg("river_directional_inertia", 0.45_f32)
            .clamp(0.0, 2.0),
        meander_strength: cfg
            .args
            .parse_arg("river_meander_strength", 0.35_f32)
            .clamp(0.0, 2.0),
        downhill_weight: cfg.args.parse_arg("river_downhill_weight", 3.0_f32).max(0.1),
        uphill_penalty: cfg.args.parse_arg("river_uphill_penalty", 2.5_f32).max(0.0),
        coast_avoid_inlandness: cfg
            .args
            .parse_arg("river_coast_avoid_inlandness", (mouth_max_inlandness + 0.18).max(0.1))
            .max(0.0001),
        coast_parallel_penalty: cfg
            .args
            .parse_arg("river_coast_parallel_penalty", 1.3_f32)
            .max(0.0),
    };
    let curve_iterations: usize = cfg.args.parse_arg("river_curve_iterations", 2_usize).min(4);
    let curve_jitter_tiles: f32 = cfg.args.parse_arg(
        "river_curve_jitter_tiles",
        (spacing as f32 * 0.30).clamp(1.0, 8.0),
    );

    let mut generated = GeneratedRiverNetwork::default();
    for source in sources {
        let Some(&source_component_i) = component_of.get(&source) else {
            continue;
        };
        let path = trace_downhill_path(
            source,
            sampled_points,
            &component_of,
            source_component_i,
            spacing,
            max_steps,
            &trace_params,
            settings,
            dimension_hash,
        );
        if path.len() < 2 {
            continue;
        }
        let smoothed_path = smooth_river_path(
            &path,
            curve_iterations,
            curve_jitter_tiles,
            settings,
            dimension_hash,
            source,
        );
        if smoothed_path.len() < 2 {
            continue;
        }
        generated.river_source_points.insert(path[0]);
        generated
            .river_mouth_points
            .insert(*path.last().unwrap_or(&path[0]));
        add_path_tiles(
            &smoothed_path,
            half_width_start,
            half_width_end,
            &mut generated.river_tiles,
        );
    }
    generated
}

fn estimate_sample_spacing(sampled_points: &HashMap<GlobalTilePos, f32>) -> i32 {
    let mut xs: Vec<i32> = sampled_points.keys().map(|p| p.0.x).collect();
    let mut ys: Vec<i32> = sampled_points.keys().map(|p| p.0.y).collect();
    xs.sort_unstable();
    ys.sort_unstable();
    xs.dedup();
    ys.dedup();
    let min_dx = xs
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(64);
    let min_dy = ys
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(64);
    min_dx.min(min_dy).max(1)
}

fn build_land_components(
    sampled_points: &HashMap<GlobalTilePos, f32>,
    land_threshold: f32,
    spacing: i32,
) -> (HashMap<GlobalTilePos, usize>, Vec<usize>) {
    let mut component_of: HashMap<GlobalTilePos, usize> = HashMap::default();
    let mut component_sizes: Vec<usize> = Vec::new();
    for (&start, &val) in sampled_points.iter() {
        if val < land_threshold || component_of.contains_key(&start) {
            continue;
        }
        let component_i = component_sizes.len();
        component_sizes.push(0);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        component_of.insert(start, component_i);
        while let Some(curr) = queue.pop_front() {
            component_sizes[component_i] = component_sizes[component_i].saturating_add(1);
            for oy in -1..=1 {
                for ox in -1..=1 {
                    if ox == 0 && oy == 0 {
                        continue;
                    }
                    let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
                    if component_of.contains_key(&next) {
                        continue;
                    }
                    let Some(&next_val) = sampled_points.get(&next) else {
                        continue;
                    };
                    if next_val < land_threshold {
                        continue;
                    }
                    component_of.insert(next, component_i);
                    queue.push_back(next);
                }
            }
        }
    }
    (component_of, component_sizes)
}

fn is_local_maximum(
    pos: GlobalTilePos,
    val: f32,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    spacing: i32,
) -> bool {
    for oy in -1..=1 {
        for ox in -1..=1 {
            if ox == 0 && oy == 0 {
                continue;
            }
            let next = GlobalTilePos(pos.0 + IVec2::new(ox * spacing, oy * spacing));
            if let Some(&next_val) = sampled_points.get(&next)
                && next_val > val
            {
                return false;
            }
        }
    }
    true
}

fn pick_river_sources(
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    component_sizes: &[usize],
    min_component_samples: usize,
    source_min_inlandness: f32,
    spacing: i32,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source_stride: u64,
    max_sources: usize,
    min_source_distance_tiles: i32,
    local_region_pos: Option<RegionPos>,
) -> Vec<GlobalTilePos> {
    let mut candidates: Vec<(GlobalTilePos, f32)> = Vec::new();
    for (&pos, &val) in sampled_points.iter() {
        if let Some(region_pos) = local_region_pos
            && !region_pos.contains_chunkpos(pos.to_chunkpos())
        {
            continue;
        }
        if val < source_min_inlandness {
            continue;
        }
        let Some(&component_i) = component_of.get(&pos) else {
            continue;
        };
        if component_sizes
            .get(component_i)
            .copied()
            .unwrap_or_default()
            < min_component_samples
        {
            continue;
        }
        if !is_local_maximum(pos, val, sampled_points, spacing) {
            continue;
        }
        if source_stride > 1 && pos.hash_value(settings, dimension_hash, 91) % source_stride != 0 {
            continue;
        }
        candidates.push((pos, val));
    }
    candidates.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.0.y.cmp(&b.0.0.y))
            .then_with(|| a.0.0.x.cmp(&b.0.0.x))
    });

    if candidates.is_empty() {
        let mut fallback: Vec<(GlobalTilePos, f32)> = sampled_points
            .iter()
            .filter_map(|(&pos, &val)| {
                if let Some(region_pos) = local_region_pos
                    && !region_pos.contains_chunkpos(pos.to_chunkpos())
                {
                    return None;
                }
                let component_i = component_of.get(&pos)?;
                if component_sizes.get(*component_i).copied().unwrap_or_default() < min_component_samples {
                    return None;
                }
                Some((pos, val))
            })
            .collect();
        if fallback.is_empty() {
            fallback = sampled_points
                .iter()
                .filter_map(|(&pos, &val)| {
                    if let Some(region_pos) = local_region_pos
                        && !region_pos.contains_chunkpos(pos.to_chunkpos())
                    {
                        return None;
                    }
                    Some((pos, val))
                })
                .collect();
        }
        fallback.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let fallback_sources = fallback.into_iter().map(|(p, _)| p).collect::<Vec<_>>();
        return select_spread_sources(
            fallback_sources,
            max_sources.max(1),
            min_source_distance_tiles.max(1),
        );
    }

    let ordered_sources = candidates.into_iter().map(|(p, _)| p).collect::<Vec<_>>();
    select_spread_sources(
        ordered_sources,
        max_sources.max(1),
        min_source_distance_tiles.max(1),
    )
}

#[derive(Clone, Copy)]
struct RiverTraceParams {
    neighbor_radius: i32,
    mouth_max_inlandness: f32,
    min_steps_before_mouth: usize,
    directional_inertia: f32,
    meander_strength: f32,
    downhill_weight: f32,
    uphill_penalty: f32,
    coast_avoid_inlandness: f32,
    coast_parallel_penalty: f32,
}

fn trace_downhill_path(
    source: GlobalTilePos,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    source_component_i: usize,
    spacing: i32,
    max_steps: usize,
    trace_params: &RiverTraceParams,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> Vec<GlobalTilePos> {
    let mut path = vec![source];
    let mut visited: HashSet<GlobalTilePos> = HashSet::default();
    visited.insert(source);

    for step in 0..max_steps {
        let curr = *path.last().unwrap_or(&source);
        let curr_val = sampled_points.get(&curr).copied().unwrap_or(0.0);
        if step >= trace_params.min_steps_before_mouth
            && curr_val <= trace_params.mouth_max_inlandness
        {
            break;
        }
        let prev_dir = if path.len() >= 2 {
            let prev = path[path.len() - 2];
            (curr.0 - prev.0).as_vec2().normalize_or_zero()
        } else {
            Vec2::ZERO
        };
        let gradient_dir = estimate_inlandness_gradient(curr, sampled_points, spacing)
            .try_normalize()
            .unwrap_or(Vec2::ZERO);

        let mut best_scored: Option<(GlobalTilePos, f32)> = None;
        for oy in -trace_params.neighbor_radius..=trace_params.neighbor_radius {
            for ox in -trace_params.neighbor_radius..=trace_params.neighbor_radius {
                if ox == 0 && oy == 0 {
                    continue;
                }
                let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
                if visited.contains(&next) {
                    continue;
                }
                if component_of.get(&next).copied() != Some(source_component_i) {
                    continue;
                }
                let Some(&next_val) = sampled_points.get(&next) else {
                    continue;
                };
                if step < trace_params.min_steps_before_mouth
                    && next_val <= trace_params.mouth_max_inlandness
                {
                    continue;
                }

                let delta = curr_val - next_val;
                let move_dir = IVec2::new(ox, oy).as_vec2().normalize_or_zero();
                let mut score = delta * trace_params.downhill_weight;
                if delta < 0.0 {
                    score -= (-delta) * trace_params.uphill_penalty;
                }
                if prev_dir.length_squared() > 0.0 {
                    score += prev_dir.dot(move_dir) * trace_params.directional_inertia;
                }
                score += river_noise_signed(
                    source,
                    curr,
                    next,
                    step,
                    settings,
                    dimension_hash,
                ) * trace_params.meander_strength;

                if gradient_dir.length_squared() > 0.0 {
                    let along_contour = 1.0 - move_dir.dot(gradient_dir).abs();
                    let coast_factor = ((trace_params.coast_avoid_inlandness - next_val)
                        / trace_params.coast_avoid_inlandness)
                        .clamp(0.0, 1.0);
                    score -= along_contour
                        * along_contour
                        * coast_factor
                        * trace_params.coast_parallel_penalty;
                }
                score -= (ox.abs().max(oy.abs()) as f32 - 1.0).max(0.0) * 0.08;

                if best_scored
                    .as_ref()
                    .map_or(true, |(_, best_score)| score > *best_score)
                {
                    best_scored = Some((next, score));
                }
            }
        }
        let Some((next, _)) = best_scored.or_else(|| {
            find_fallback_flow_target(
                curr,
                curr_val,
                sampled_points,
                component_of,
                source_component_i,
                &visited,
                spacing,
                (trace_params.neighbor_radius * 3).max(2),
            )
        }) else {
            break;
        };
        visited.insert(next);
        path.push(next);
    }

    path
}

fn find_fallback_flow_target(
    curr: GlobalTilePos,
    curr_val: f32,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    source_component_i: usize,
    visited: &HashSet<GlobalTilePos>,
    spacing: i32,
    max_radius_steps: i32,
) -> Option<(GlobalTilePos, f32)> {
    let mut best_lower: Option<(GlobalTilePos, f32, i32)> = None;
    let mut best_any: Option<(GlobalTilePos, f32, i32)> = None;
    for oy in -max_radius_steps..=max_radius_steps {
        for ox in -max_radius_steps..=max_radius_steps {
            if ox == 0 && oy == 0 {
                continue;
            }
            let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
            if visited.contains(&next) {
                continue;
            }
            if component_of.get(&next).copied() != Some(source_component_i) {
                continue;
            }
            let Some(&next_val) = sampled_points.get(&next) else {
                continue;
            };
            let d = ox.abs().max(oy.abs());
            if next_val < curr_val
                && best_lower
                    .as_ref()
                    .map_or(true, |(_, best_val, best_d)| d < *best_d || (d == *best_d && next_val < *best_val))
            {
                best_lower = Some((next, next_val, d));
            }
            if best_any
                .as_ref()
                .map_or(true, |(_, best_val, best_d)| d < *best_d || (d == *best_d && next_val < *best_val))
            {
                best_any = Some((next, next_val, d));
            }
        }
    }
    best_lower
        .or(best_any)
        .map(|(next, next_val, _)| (next, next_val))
}

fn select_spread_sources(
    ordered_sources: Vec<GlobalTilePos>,
    max_sources: usize,
    min_source_distance_tiles: i32,
) -> Vec<GlobalTilePos> {
    if ordered_sources.is_empty() {
        return Vec::new();
    }
    let max_sources = max_sources.max(1);
    let min_dist_sq = (min_source_distance_tiles.max(1) * min_source_distance_tiles.max(1)) as i64;
    let mut selected: Vec<GlobalTilePos> = Vec::with_capacity(max_sources);
    for &candidate in ordered_sources.iter() {
        let too_close = selected
            .iter()
            .any(|&picked| picked.distance_squared(&candidate) as i64 <= min_dist_sq);
        if too_close {
            continue;
        }
        selected.push(candidate);
        if selected.len() >= max_sources {
            return selected;
        }
    }
    for &candidate in ordered_sources.iter() {
        if selected.contains(&candidate) {
            continue;
        }
        selected.push(candidate);
        if selected.len() >= max_sources {
            break;
        }
    }
    selected
}

fn estimate_inlandness_gradient(
    pos: GlobalTilePos,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    spacing: i32,
) -> Vec2 {
    let center = sampled_points.get(&pos).copied().unwrap_or(0.0);
    let x_pos = sampled_points
        .get(&GlobalTilePos(pos.0 + IVec2::new(spacing, 0)))
        .copied()
        .unwrap_or(center);
    let x_neg = sampled_points
        .get(&GlobalTilePos(pos.0 - IVec2::new(spacing, 0)))
        .copied()
        .unwrap_or(center);
    let y_pos = sampled_points
        .get(&GlobalTilePos(pos.0 + IVec2::new(0, spacing)))
        .copied()
        .unwrap_or(center);
    let y_neg = sampled_points
        .get(&GlobalTilePos(pos.0 - IVec2::new(0, spacing)))
        .copied()
        .unwrap_or(center);
    Vec2::new((x_pos - x_neg) * 0.5, (y_pos - y_neg) * 0.5)
}

fn river_noise_signed(
    source: GlobalTilePos,
    curr: GlobalTilePos,
    next: GlobalTilePos,
    step: usize,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> f32 {
    let a = source.hash_value(settings, dimension_hash, 191 + step as u64);
    let b = curr.hash_value(settings, dimension_hash, 277 + (step as u64 * 3));
    let c = next.hash_value(settings, dimension_hash, 383 + (step as u64 * 5));
    let mixed = a ^ b.rotate_left(19) ^ c.rotate_left(37);
    ((mixed & 0xFFFF) as f32 / 65535.0) * 2.0 - 1.0
}

fn smooth_river_path(
    path: &[GlobalTilePos],
    curve_iterations: usize,
    curve_jitter_tiles: f32,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source: GlobalTilePos,
) -> Vec<GlobalTilePos> {
    if path.len() < 2 {
        return path.to_vec();
    }
    let mut points = path.iter().map(|point| point.0.as_vec2()).collect::<Vec<_>>();
    for iter in 0..curve_iterations {
        let mut next_points = Vec::with_capacity(points.len() * 2);
        next_points.push(points[0]);
        for window in points.windows(2) {
            let p0 = window[0];
            let p1 = window[1];
            next_points.push(p0 * 0.75 + p1 * 0.25);
            next_points.push(p0 * 0.25 + p1 * 0.75);
        }
        next_points.push(*points.last().unwrap_or(&points[0]));
        points = next_points;
        if curve_jitter_tiles > 0.0 && points.len() > 2 {
            for i in 1..points.len() - 1 {
                let tangent = (points[i + 1] - points[i - 1]).normalize_or_zero();
                if tangent.length_squared() <= f32::EPSILON {
                    continue;
                }
                let normal = Vec2::new(-tangent.y, tangent.x);
                let jitter = river_noise_signed(
                    source,
                    GlobalTilePos(points[i].round().as_ivec2()),
                    GlobalTilePos(points[i + 1].round().as_ivec2()),
                    iter * 4096 + i,
                    settings,
                    dimension_hash,
                ) * curve_jitter_tiles;
                points[i] += normal * jitter;
            }
        }
    }

    let mut smoothed = Vec::with_capacity(points.len());
    for point in points {
        let tile = GlobalTilePos(point.round().as_ivec2());
        if smoothed.last().copied() == Some(tile) {
            continue;
        }
        smoothed.push(tile);
    }
    if smoothed.len() < 2 {
        return path.to_vec();
    }
    smoothed
}

fn add_path_tiles(
    path: &[GlobalTilePos],
    half_width_start: i32,
    half_width_end: i32,
    out_tiles: &mut HashSet<GlobalTilePos>,
) {
    if path.len() < 2 {
        return;
    }
    let seg_count = (path.len() - 1) as f32;
    for i in 0..path.len() - 1 {
        let t = (i as f32 / seg_count).clamp(0.0, 1.0);
        let half_width = lerp_i32(half_width_start, half_width_end, t).max(0);
        for point in bresenham_line(path[i], path[i + 1]) {
            for oy in -half_width..=half_width {
                for ox in -half_width..=half_width {
                    if ox * ox + oy * oy > half_width * half_width + 1 {
                        continue;
                    }
                    out_tiles.insert(GlobalTilePos(point.0 + IVec2::new(ox, oy)));
                }
            }
        }
    }
}

fn bresenham_line(from: GlobalTilePos, to: GlobalTilePos) -> Vec<GlobalTilePos> {
    let mut points = Vec::new();
    let mut x0 = from.0.x;
    let mut y0 = from.0.y;
    let x1 = to.0.x;
    let y1 = to.0.y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(GlobalTilePos(IVec2::new(x0, y0)));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    points
}

fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    ((a as f32) + ((b - a) as f32) * t).round() as i32
}
