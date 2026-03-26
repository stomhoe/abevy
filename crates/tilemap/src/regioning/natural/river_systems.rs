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
        regioning_messages::{ChunksClaim, OfferChunk, SgcPrepareTilesOrder, StructureBuildCompliance},
        regioning_sgc_components::StructuredGenConfig,
    },
    terrain::terrprobe::{
        terrprobe_components::TerrProbeTempl,
        terrprobe_messages::{SampledValuesCollected, TerrProbeJob},
        terrprobe_resources::TerrProbeTemplEntityMap,
    },
    tile::tile_resources::TileEntityMap,
};
use super::river_components::{RiverDebugData, RiverProbeRequest};

const RIVER: HashId = HashId::hash("river");
const RIVER_REGION_PROBE_ID: &str = "river_region_probe";

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
    dimension_hash_q: Query<&HashId>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    probe_requests: Query<(Entity, &RiverProbeRequest)>,
    mut terrprobe_writer: MessageWriter<TerrProbeJob>,
    mut sampled_values_reader: MessageReader<SampledValuesCollected>,
    mut claimlists: Query<&mut ClaimList>,
    mut river_debug: ResMut<RiverDebugData>,
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
        if regions_with_probe.contains(&(dimension_ref, region_pos)) {
            trace!(
                target: RIVER_SYSTEM,
                "Offer {}: skipping region {:?} in dim {:?}, probe already active",
                offer.i,
                region_pos,
                dimension_ref
            );
            claim_state.skipped_offers.push((offer.region_ent, offer.i as usize));
            offers_skipped = offers_skipped.saturating_add(1);
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
            dimension_ref
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
        let info = river_debug.region_mut(dimension_ref, req.region_pos);
        if matched_samples > 0 {
            sampled_points_total = sampled_points_total.saturating_add(matched_samples);
            claim_state.completed_with_samples.insert(sampled_values.requester);
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
        info!(
            target: RIVER_SYSTEM,
            "Requester {:?} region {:?} dim {:?}: received sampled matrix entries={}, matched_samples={}",
            sampled_values.requester,
            req.region_pos,
            dimension_ref,
            sampled_values.matrix.values.len(),
            matched_samples
        );
        claim_state.completed_probes.insert(sampled_values.requester);
    }

    for ent in claim_state.completed_probes.drain() {
        let Ok((_, req)) = probe_requests.get(ent) else {
            cmd.entity(ent).despawn();
            continue;
        };
        let dimension_ref = req.dimension_ref;
        probes_completed = probes_completed.saturating_add(1);
        if claim_state.completed_with_samples.contains(&ent) {
            let Ok(cfg) = structured_gens.get(req.sgc_ent) else {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: missing StructuredGenConfig {:?}",
                    ent,
                    req.region_pos,
                    dimension_ref,
                    req.sgc_ent
                );
                claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
                river_debug.mark_probe_finished(dimension_ref, req.region_pos, req.start_chunk);
                cmd.entity(ent).despawn();
                continue;
            };
            let Ok(settings) = settings_q.single() else {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: missing GlobalGenSettings",
                    ent,
                    req.region_pos,
                    dimension_ref
                );
                claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
                river_debug.mark_probe_finished(dimension_ref, req.region_pos, req.start_chunk);
                cmd.entity(ent).despawn();
                continue;
            };
            let Ok(&dimension_hash) = dimension_hash_q.get(dimension_ref.0) else {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: missing dimension hash",
                    ent,
                    req.region_pos,
                    dimension_ref
                );
                claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
                river_debug.mark_probe_finished(dimension_ref, req.region_pos, req.start_chunk);
                cmd.entity(ent).despawn();
                continue;
            };
            let Some(region_info_ro) = river_debug.0.get(&(dimension_ref, req.region_pos)) else {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: no RiverDebugData entry at completion",
                    ent,
                    req.region_pos,
                    dimension_ref
                );
                claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
                river_debug.mark_probe_finished(dimension_ref, req.region_pos, req.start_chunk);
                cmd.entity(ent).despawn();
                continue;
            };
            let sampled_points = region_info_ro.sampled_points.clone();
            let generated_tiles = generate_river_tiles_for_region(
                &sampled_points,
                cfg,
                settings,
                dimension_hash,
            );
            let mut claimed_chunks: HashSet<ChunkPos> = HashSet::default();
            for gpos in generated_tiles.iter().copied() {
                let chunk_pos = gpos.to_chunkpos();
                if !req.region_pos.contains_chunkpos(chunk_pos) {
                    continue;
                }
                claimed_chunks.insert(chunk_pos);
            }
            let mut chunks_gpos: Vec<ChunkPos> = claimed_chunks.into_iter().collect();
            chunks_gpos.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));

            if chunks_gpos.is_empty() {
                error!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: generated 0 local river chunks, skipping offer {}",
                    ent,
                    req.region_pos,
                    dimension_ref,
                    req.offer_i
                );
                claim_state.skipped_offers.push((req.region_ent, req.offer_i as usize));
                offers_skipped = offers_skipped.saturating_add(1);
            } else {
                let local_chunk_count = chunks_gpos.len();
                let generated_tiles_count = generated_tiles.len();
                claim_state.claims_to_emit.push(ChunksClaim {
                    i: req.offer_i,
                    region_ent: req.region_ent,
                    sgc_ent: req.sgc_ent,
                    chunks_gpos,
                    partition_tolerant: true,
                });
                claims_emitted = claims_emitted.saturating_add(1);
                info!(
                    target: RIVER_SYSTEM,
                    "Probe {:?} region {:?} dim {:?}: emitted ChunksClaim for offer {} (generated_tiles={}, local_chunks={})",
                    ent,
                    req.region_pos,
                    dimension_ref,
                    req.offer_i,
                    generated_tiles_count,
                    local_chunk_count
                );
            }
        } else {
            error!(
                target: RIVER_SYSTEM,
                "Probe {:?} region {:?} dim {:?}: completed with no sampled values, skipping offer {}",
                ent,
                req.region_pos,
                dimension_ref,
                req.offer_i
            );
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
    settings_q: Query<&GlobalGenSettings>,
    dimension_hash_q: Query<&HashId>,
    tiles_map: Res<TileEntityMap>,
    mut river_debug: ResMut<RiverDebugData>,
    mut writer: MessageWriter<StructureBuildCompliance>,
) {
    let Ok(settings) = settings_q.single() else {
        error!(
            target: RIVER_SYSTEM,
            "river_structure_building_system: missing GlobalGenSettings"
        );
        return;
    };
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
            order.chunks_gpos.len()
        );

        let mut compliance = StructureBuildCompliance {
            i: order.i,
            structure_gen_cfg_ent: order.structured_gen_cfg_ent,
            dimension_ref: order.dimension_ref,
            chunks: Vec::new(),
            terrgen_disabled_gpos_for_chunks: Vec::new(),
            terrgen_disabled_for_chunks: Vec::new(),
        };

        let Ok(&dimension_hash) = dimension_hash_q.get(order.dimension_ref.0) else {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: missing dimension hash for {:?}",
                order.i,
                order.dimension_ref
            );
            compliances_to_emit.push(compliance);
            continue;
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

        let Some(region_info_ro) = river_debug.0.get(&(order.dimension_ref, order.region_pos)) else {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: no RiverDebugData entry for region {:?} dim {:?}",
                order.i,
                order.region_pos,
                order.dimension_ref
            );
            compliances_to_emit.push(compliance);
            continue;
        };
        if region_info_ro.sampled_points.is_empty() {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: RiverDebugData for region {:?} dim {:?} has 0 sampled points",
                order.i,
                order.region_pos,
                order.dimension_ref
            );
            compliances_to_emit.push(compliance);
            continue;
        }
        let sampled_points = region_info_ro.sampled_points.clone();

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

        let spacing = estimate_sample_spacing(&sampled_points);
        let (component_of, component_sizes) = build_land_components(&sampled_points, land_threshold, spacing);
        let sample_area_chunks = ((spacing as f32 / ChunkPos::CHUNK_SIZE.x as f32)
            * (spacing as f32 / ChunkPos::CHUNK_SIZE.y as f32))
            .max(0.001);
        let min_component_samples = (min_island_area_chunks / sample_area_chunks).ceil() as usize;

        let sources = pick_river_sources(
            &sampled_points,
            &component_of,
            &component_sizes,
            min_component_samples.max(1),
            source_min_inlandness,
            spacing,
            settings,
            dimension_hash,
            source_stride,
            max_sources,
            None,
        );
        info!(
            target: RIVER_SYSTEM,
            "Order {}: sampled_points={}, spacing={}, min_component_samples={}, sources={}",
            order.i,
            sampled_points.len(),
            spacing,
            min_component_samples.max(1),
            sources.len()
        );
        if sources.is_empty() {
            error!(
                target: RIVER_SYSTEM,
                "Order {}: no river sources selected for region {:?} dim {:?}",
                order.i,
                order.region_pos,
                order.dimension_ref
            );
        }

        let mut generated_tiles: HashSet<GlobalTilePos> = HashSet::default();
        let mut river_sources: HashSet<GlobalTilePos> = HashSet::default();
        let mut river_mouths: HashSet<GlobalTilePos> = HashSet::default();
        let mut attempted_paths = 0_u32;
        let mut accepted_paths = 0_u32;
        for source in sources {
            let Some(&source_component_i) = component_of.get(&source) else {
                continue;
            };
            attempted_paths = attempted_paths.saturating_add(1);
            let path = trace_downhill_path(
                source,
                &sampled_points,
                &component_of,
                source_component_i,
                spacing,
                max_steps,
                mouth_max_inlandness,
                source_mouth_min_distance,
            );
            if path.len() < 2 {
                trace!(
                    target: RIVER_SYSTEM,
                    "Order {}: dropped source {:?} because downhill path len < 2",
                    order.i,
                    source
                );
                continue;
            }
            accepted_paths = accepted_paths.saturating_add(1);
            river_sources.insert(path[0]);
            river_mouths.insert(*path.last().unwrap_or(&path[0]));
            add_path_tiles(&path, half_width_start, half_width_end, &mut generated_tiles);
        }
        let generated_count = generated_tiles.len();
        info!(
            target: RIVER_SYSTEM,
            "Order {}: attempted_paths={}, accepted_paths={}, unique_sources={}, unique_mouths={}, generated_tiles={}",
            order.i,
            attempted_paths,
            accepted_paths,
            river_sources.len(),
            river_mouths.len(),
            generated_count
        );
        generated_tiles_total = generated_tiles_total.saturating_add(generated_count);

        let claimed_chunks: HashSet<ChunkPos> = order.chunks_gpos.iter().copied().collect();
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
        info.claimed_chunks.extend(order.chunks_gpos.iter().copied());
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

fn generate_river_tiles_for_region(
    sampled_points: &HashMap<GlobalTilePos, f32>,
    cfg: &StructuredGenConfig,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> HashSet<GlobalTilePos> {
    if sampled_points.is_empty() {
        return HashSet::default();
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
        None,
    );

    let mut generated_tiles: HashSet<GlobalTilePos> = HashSet::default();
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
            mouth_max_inlandness,
            source_mouth_min_distance,
        );
        if path.len() < 2 {
            continue;
        }
        add_path_tiles(&path, half_width_start, half_width_end, &mut generated_tiles);
    }
    generated_tiles
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
        return fallback.into_iter().take(max_sources.max(1)).map(|(p, _)| p).collect();
    }

    candidates
        .into_iter()
        .take(max_sources.max(1))
        .map(|(p, _)| p)
        .collect()
}

fn trace_downhill_path(
    source: GlobalTilePos,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    source_component_i: usize,
    spacing: i32,
    max_steps: usize,
    mouth_max_inlandness: f32,
    min_steps_before_mouth: usize,
) -> Vec<GlobalTilePos> {
    let mut path = vec![source];
    let mut visited: HashSet<GlobalTilePos> = HashSet::default();
    visited.insert(source);

    for step in 0..max_steps {
        let curr = *path.last().unwrap_or(&source);
        let curr_val = sampled_points.get(&curr).copied().unwrap_or(0.0);
        if step >= min_steps_before_mouth && curr_val <= mouth_max_inlandness {
            break;
        }

        let mut best_downhill: Option<(GlobalTilePos, f32)> = None;
        let mut best_any: Option<(GlobalTilePos, f32)> = None;
        for oy in -1..=1 {
            for ox in -1..=1 {
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
                if next_val < curr_val
                    && best_downhill.as_ref().map_or(true, |(_, best_val)| next_val < *best_val)
                {
                    best_downhill = Some((next, next_val));
                }
                if best_any.as_ref().map_or(true, |(_, best_val)| next_val < *best_val) {
                    best_any = Some((next, next_val));
                }
            }
        }
        let Some((next, _)) = best_downhill.or(best_any).or_else(|| {
            find_fallback_flow_target(curr, curr_val, sampled_points, component_of, source_component_i)
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
) -> Option<(GlobalTilePos, f32)> {
    let mut best_lower: Option<(GlobalTilePos, f32, i64)> = None;
    let mut best_any: Option<(GlobalTilePos, f32, i64)> = None;
    for (&next, &next_val) in sampled_points.iter() {
        if next == curr {
            continue;
        }
        if component_of.get(&next).copied() != Some(source_component_i) {
            continue;
        }
        let d = curr.distance_squared(&next) as i64;
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
    best_lower
        .or(best_any)
        .map(|(next, next_val, _)| (next, next_val))
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
