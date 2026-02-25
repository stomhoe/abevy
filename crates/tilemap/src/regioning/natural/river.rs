use std::collections::VecDeque;

use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::{
    common_components::{HashId, Tag},
    log_targets::{SGC_CHUNK_CLAIM, STRUCTURE_SPAWN},
};
use game_common::game_common_components::{ArgsDict, EntityZeroRef};
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{
    regioning::{
        regioning_components::ClaimList,
        regioning_messages::{ChunksClaim, OfferChunk, SgcPrepareTilesOrder, StructureBuildCompliance},
        regioning_sgc_components::StructuredGenConfig,
    },
    run_suitable_pos_search_logic,
    terrain::terrprobe::{
        terrprobe_components::{AwaitingStartSearch, SearchingForSuitablePos, TerrProbeTempl},
        terrprobe_messages::TerrProbeJob,
        terrprobe_resources::TerrProbeTemplEntityMap,
        terrprobe_systems::SearchParams,
    },
    tile::{tile_components::DeleteOtherTiles, tile_resources::TileEntityMap},
};

const RIVER: HashId = HashId::hash("river");
const DEFAULT_PROBE_TEMPLATE_ID: &str = "reland_all_probe";
const DEFAULT_RIVER_TILE_ID: &str = "blue";

#[derive(Component, Debug, Clone, Copy)]
pub struct RiverProbeRequest {
    pub region_ent: Entity,
    pub sgc_ent: Entity,
    pub offer_i: u64,
    pub start_chunk: ChunkPos,
    pub probe_templ_ent: Entity,
    pub sample_step: u16,
}

#[derive(Debug, Clone, Copy)]
struct RiverSample {
    pos: GlobalTilePos,
    inlandness: f32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct RiverPlanKey {
    dimension_ref: DimensionRef,
    region_pos: RegionPos,
    order_i: u64,
    sgc_ent: Entity,
}
impl RiverPlanKey {
    fn from_order(order: &SgcPrepareTilesOrder) -> Self {
        Self {
            dimension_ref: order.dimension_ref,
            region_pos: order.region_pos,
            order_i: order.i,
            sgc_ent: order.structured_gen_cfg_ent,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct RiverPlan {
    claimed_chunks: Vec<ChunkPos>,
    tiles_by_chunk: HashMap<ChunkPos, Vec<GlobalTilePos>>,
}

#[derive(Resource, Default)]
pub struct RiverPlans(HashMap<RiverPlanKey, RiverPlan>);

#[derive(Debug, Clone)]
struct RiverClaimArgs {
    probe_template_id: String,
    land_threshold: f32,
    min_landmass_width: i32,
    min_landmass_height: i32,
    source_mouth_min_distance: u64,
    main_half_width_start: i32,
    main_half_width_end: i32,
    meander_amplitude: f32,
    meander_frequency: f32,
    jitter: f32,
    branch_count: usize,
    branch_source_min_dist: i32,
    branch_source_max_dist: i32,
    tributary_min_inlandness_delta: f32,
    delta_branch_count: usize,
    delta_length: i32,
    delta_spread_deg: f32,
    uphill_penalty: f32,
}
impl RiverClaimArgs {
    fn from_cfg(cfg: &StructuredGenConfig) -> Self {
        let min_dist = cfg
            .args
            .parse_arg("river_source_mouth_min_distance", 90_i32)
            .max(0) as u64;
        Self {
            probe_template_id: parse_arg_string(&cfg.args, "probe_template_id", DEFAULT_PROBE_TEMPLATE_ID),
            land_threshold: cfg.args.parse_arg("river_land_threshold", 0.0_f32),
            min_landmass_width: cfg.args.parse_arg("river_min_landmass_width", 20_i32).max(1),
            min_landmass_height: cfg.args.parse_arg("river_min_landmass_height", 20_i32).max(1),
            source_mouth_min_distance: min_dist * min_dist,
            main_half_width_start: cfg.args.parse_arg("river_main_half_width_start", 1_i32).clamp(1, 8),
            main_half_width_end: cfg.args.parse_arg("river_main_half_width_end", 3_i32).clamp(1, 10),
            meander_amplitude: cfg.args.parse_arg("river_meander_amplitude", 5.0_f32).clamp(0.0, 64.0),
            meander_frequency: cfg.args.parse_arg("river_meander_frequency", 0.035_f32).clamp(0.001, 8.0),
            jitter: cfg.args.parse_arg("river_jitter", 1.25_f32).clamp(0.0, 24.0),
            branch_count: cfg.args.parse_arg("river_branch_count", 2_i32).clamp(0, 8) as usize,
            branch_source_min_dist: cfg.args.parse_arg("river_branch_source_min_dist", 32_i32).clamp(8, 512),
            branch_source_max_dist: cfg.args.parse_arg("river_branch_source_max_dist", 420_i32).clamp(16, 2048),
            tributary_min_inlandness_delta: cfg
                .args
                .parse_arg("river_tributary_min_inlandness_delta", 0.05_f32)
                .clamp(0.0, 10.0),
            delta_branch_count: cfg.args.parse_arg("river_delta_branch_count", 3_i32).clamp(0, 8) as usize,
            delta_length: cfg.args.parse_arg("river_delta_length", 80_i32).clamp(8, 512),
            delta_spread_deg: cfg.args.parse_arg("river_delta_spread_deg", 42.0_f32).clamp(0.0, 180.0),
            uphill_penalty: cfg.args.parse_arg("river_uphill_penalty", 5.0_f32).clamp(0.1, 64.0),
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct RiverClaimState<'w, 's> {
    claims_writer: MessageWriter<'w, ChunksClaim>,
    samples_by_probe: Local<'s, EntityHashMap<Vec<RiverSample>>>,
    claims_to_emit: Local<'s, Vec<ChunksClaim>>,
    skips_to_mark: Local<'s, Vec<(Entity, u64)>>,
    completed_probe_ents: Local<'s, EntityHashSet>,
    failed_probe_ents: Local<'s, Vec<Entity>>,
    probe_ents_to_despawn: Local<'s, Vec<Entity>>,
}

fn parse_arg_string(args: &ArgsDict, key: &str, default: &str) -> String {
    args.get(key)
        .and_then(|vals| vals.first())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[allow(unused_parens)]
pub fn claim_chunks_for_river_structures(
    mut cmd: Commands,
    mut offered_chunks: MessageReader<OfferChunk>,
    region_dimension: Query<&DimensionRef>,
    structured_gens: Query<&StructuredGenConfig>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    searching_entities: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            &EntityZeroRef,
            Has<AwaitingStartSearch>,
            Option<&SearchingForSuitablePos>,
        ),
        With<RiverProbeRequest>,
    >,
    probe_requests: Query<&RiverProbeRequest>,
    probe_states: Query<(Entity, &RiverProbeRequest, Has<AwaitingStartSearch>, Has<SearchingForSuitablePos>)>,
    mut claimlists: Query<&mut ClaimList>,
    settings: Query<&GlobalGenSettings>,
    dimension_hashes: Query<&HashId>,
    mut search_params: SearchParams,
    mut river_plans: ResMut<RiverPlans>,
    mut claim_state: RiverClaimState,
) {
    claim_state.claims_to_emit.clear();
    claim_state.skips_to_mark.clear();
    claim_state.failed_probe_ents.clear();
    let mut regions_with_active_probe = EntityHashSet::default();
    for req in probe_requests.iter() {
        regions_with_active_probe.insert(req.region_ent);
    }
    let mut regions_with_new_probe = EntityHashSet::default();

    for offer in offered_chunks.read() {
        let Ok(cfg) = structured_gens.get(offer.structured_gen_cfg_ent) else {
            claim_state.skips_to_mark.push((offer.region_ent, offer.i));
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }
        if regions_with_active_probe.contains(&offer.region_ent)
            || regions_with_new_probe.contains(&offer.region_ent)
        {
            claim_state.skips_to_mark.push((offer.region_ent, offer.i));
            continue;
        }

        let Ok(&dimension_ref) = region_dimension.get(offer.region_ent) else {
            error!(
                target: SGC_CHUNK_CLAIM,
                "River claim offer {:?} has missing DimensionRef on region {:?}",
                offer.i,
                offer.region_ent
            );
            claim_state.skips_to_mark.push((offer.region_ent, offer.i));
            continue;
        };

        let args = RiverClaimArgs::from_cfg(cfg);
        let Ok(probe_templ_ent) = terrprobe_entity_map.0.get_cloned(args.probe_template_id.as_str()) else {
            error!(
                target: SGC_CHUNK_CLAIM,
                "River SGC {:?} references missing terrain probe template '{}'",
                offer.structured_gen_cfg_ent,
                args.probe_template_id
            );
            claim_state.skips_to_mark.push((offer.region_ent, offer.i));
            continue;
        };
        let Ok(probe_templ) = terrprobe_query.get(probe_templ_ent) else {
            error!(
                target: SGC_CHUNK_CLAIM,
                "River SGC {:?} probe template entity {:?} is missing TerrProbeTempl",
                offer.structured_gen_cfg_ent,
                probe_templ_ent
            );
            claim_state.skips_to_mark.push((offer.region_ent, offer.i));
            continue;
        };

        let chunk_center = offer.start_pos.to_tilepos()
            + IVec2::new(
                (ChunkPos::CHUNK_SIZE.x / 2) as i32,
                (ChunkPos::CHUNK_SIZE.y / 2) as i32,
            );
        cmd.spawn((
            RiverProbeRequest {
                region_ent: offer.region_ent,
                sgc_ent: offer.structured_gen_cfg_ent,
                offer_i: offer.i,
                start_chunk: offer.start_pos,
                probe_templ_ent,
                sample_step: probe_templ.step_size.max(1),
            },
            dimension_ref,
            chunk_center,
            EntityZeroRef(Entity::PLACEHOLDER),
            AwaitingStartSearch,
        ));
        regions_with_new_probe.insert(offer.region_ent);
    }

    for (_, req, is_awaiting_start, is_searching) in probe_states.iter() {
        if !is_awaiting_start && !is_searching {
            continue;
        }
        let Ok(mut claimlist) = claimlists.get_mut(req.region_ent) else {
            continue;
        };
        claimlist.advance_timer.reset();
    }

    let Ok(settings) = settings.single() else {
        error!(target: SGC_CHUNK_CLAIM, "Failed to get GlobalGenSettings for river claims");
        return;
    };

        let make_search_request = |_cmd: &mut Commands,
                               search_ent: Entity,
                               search_pos: GlobalTilePos,
                               _ezero_ref: EntityZeroRef|
     -> Option<TerrProbeJob> {
        let Ok(req) = probe_requests.get(search_ent) else {
            return None;
        };
        let Ok(&dimension_ref) = region_dimension.get(req.region_ent) else {
            return None;
        };
        let Ok(templ) = terrprobe_query.get(req.probe_templ_ent) else {
            return None;
        };
        let mut probe = templ.to_probe(req.probe_templ_ent, dimension_ref, search_pos);
        probe.requester = search_ent;
        probe.collect_all_successes = true;
        Some(probe)
    };

    let mut handle_success_event = |_: &mut Commands,
                                    search_ent: Entity,
                                    _my_pos: GlobalTilePos,
                                    dim_ref: DimensionRef,
                                    _ezero_ref: EntityZeroRef,
                                    found_pos: GlobalTilePos,
                                    sampled_val: f32,
                                    is_last: bool|
     -> bool {
        claim_state.samples_by_probe.entry(search_ent).or_default().push(RiverSample {
            pos: found_pos,
            inlandness: sampled_val,
        });
        if !is_last {
            return false;
        }

        let Ok(req) = probe_requests.get(search_ent) else {
            claim_state.completed_probe_ents.insert(search_ent);
            return true;
        };
        let Ok(cfg) = structured_gens.get(req.sgc_ent) else {
            claim_state.skips_to_mark.push((req.region_ent, req.offer_i));
            claim_state.completed_probe_ents.insert(search_ent);
            claim_state.samples_by_probe.remove(&search_ent);
            return true;
        };
        let args = RiverClaimArgs::from_cfg(cfg);
        let dimension_hash = dimension_hashes
            .get(dim_ref.0)
            .copied()
            .unwrap_or_default();

        let samples = claim_state.samples_by_probe.remove(&search_ent).unwrap_or_default();
        let Some(plan) = build_river_plan(
            req.start_chunk.to_region_pos(),
            req.start_chunk,
            req.sample_step,
            &samples,
            &args,
            settings,
            dimension_hash,
            req.offer_i,
        ) else {
            trace!(
                target: SGC_CHUNK_CLAIM,
                "River offer {} skipped due to unsuitable sampled landmass",
                req.offer_i
            );
            claim_state.skips_to_mark.push((req.region_ent, req.offer_i));
            claim_state.completed_probe_ents.insert(search_ent);
            return true;
        };

        let key = RiverPlanKey {
            dimension_ref: dim_ref,
            region_pos: req.start_chunk.to_region_pos(),
            order_i: req.offer_i,
            sgc_ent: req.sgc_ent,
        };
        let chunks_gpos = plan.claimed_chunks.clone();
        river_plans.0.insert(key, plan);
        claim_state.claims_to_emit.push(ChunksClaim {
            i: req.offer_i,
            region_ent: req.region_ent,
            sgc_ent: req.sgc_ent,
            chunks_gpos,
            partition_tolerant: false,
        });
        claim_state.completed_probe_ents.insert(search_ent);
        true
    };

    let mut handle_pending_failure = |search_ent: Entity,
                                      _global_pos: GlobalTilePos,
                                      _dim_ref: DimensionRef,
                                      _ezero_ref: EntityZeroRef,
                                      _failed_filter_ent: Entity| {
        claim_state.failed_probe_ents.push(search_ent);
    };

    run_suitable_pos_search_logic!(
        target: SGC_CHUNK_CLAIM,
        searched_entity_label: "river",
        cmd: cmd,
        searching_entities: searching_entities,
        search_params: search_params,
        make_search_request: make_search_request,
        handle_success_event: handle_success_event,
        handle_pending_failure: handle_pending_failure,
    );

    for search_ent in claim_state.failed_probe_ents.drain(..) {
        let Ok(req) = probe_requests.get(search_ent) else {
            continue;
        };
        claim_state.skips_to_mark.push((req.region_ent, req.offer_i));
        claim_state.samples_by_probe.remove(&search_ent);
        claim_state.completed_probe_ents.insert(search_ent);
    }

    for (region_ent, i) in claim_state.skips_to_mark.drain(..) {
        let Ok(mut claimlist) = claimlists.get_mut(region_ent) else {
            continue;
        };
        claimlist.skipped_is.insert(i as usize);
    }

    claim_state
        .claims_writer
        .write_batch(claim_state.claims_to_emit.drain(..));

    claim_state.probe_ents_to_despawn.clear();
    for (ent, req, is_awaiting_start, is_searching) in probe_states.iter() {
        if !claim_state.completed_probe_ents.contains(&ent) {
            continue;
        }
        if is_awaiting_start || is_searching {
            let Ok(mut claimlist) = claimlists.get_mut(req.region_ent) else {
                continue;
            };
            claimlist.advance_timer.reset();
            continue;
        }
        claim_state.probe_ents_to_despawn.push(ent);
    }
    for ent in claim_state.probe_ents_to_despawn.drain(..) {
        cmd.entity(ent).despawn();
        claim_state.samples_by_probe.remove(&ent);
        claim_state.completed_probe_ents.remove(&ent);
    }
}

#[allow(unused_parens)]
pub fn river_structure_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<&StructuredGenConfig>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEntityMap>,
    mut river_plans: ResMut<RiverPlans>,
) {
    let mut compliances_to_emit = Vec::new();

    for build_order in reader.read() {
        let Ok(cfg) = structured_gens.get(build_order.structured_gen_cfg_ent) else {
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }

        let river_tile_id = parse_arg_string(&cfg.args, "river_tile_id", DEFAULT_RIVER_TILE_ID);
        let Ok(river_tile_ent) = ezeros_map.0.get_cloned(HashId::hash(river_tile_id.as_str())) else {
            error!(
                target: STRUCTURE_SPAWN,
                "River build order {} references unknown tile '{}'",
                build_order.i,
                river_tile_id
            );
            continue;
        };
        let river_tile_ref = EntityZeroRef(river_tile_ent);

        let mut delete_template = DeleteOtherTiles::default();
        delete_template.priority = cfg.args.parse_arg("river_delete_priority", 1000.0_f32);
        if let Some(spared_tags) = cfg.args.get("river_delete_spared_tags") {
            for tag in spared_tags {
                if tag.trim().is_empty() {
                    continue;
                }
                delete_template.spared_tags.insert(Tag::trunc(tag));
            }
        }

        let plan_key = RiverPlanKey::from_order(build_order);
        let plan_opt = river_plans.0.remove(&plan_key);
        if plan_opt.is_none() {
            warn!(
                target: STRUCTURE_SPAWN,
                "Missing river plan for build order {} in region {:?}; emitting empty compliance",
                build_order.i,
                build_order.region_pos
            );
        }

        let mut chunk_tiles = Vec::with_capacity(build_order.chunks_gpos.len());
        for &chunk_pos in &build_order.chunks_gpos {
            let mut tiles4chunk = Vec::new();
            if let Some(plan) = plan_opt.as_ref()
                && let Some(tiles) = plan.tiles_by_chunk.get(&chunk_pos)
            {
                for &tile_pos in tiles {
                    tiles4chunk.push((tile_pos, river_tile_ref, Some(delete_template.clone())));
                }
            }
            chunk_tiles.push((chunk_pos, tiles4chunk));
        }

        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_for_chunks: Vec::new(),
        });
    }

    writer.write_batch(compliances_to_emit);
}

fn build_river_plan(
    region_pos: RegionPos,
    start_chunk: ChunkPos,
    sample_step: u16,
    samples: &[RiverSample],
    args: &RiverClaimArgs,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    offer_i: u64,
) -> Option<RiverPlan> {
    let step = sample_step.max(1) as i32;
    if samples.len() < 4 {
        return None;
    }

    let mut land_nodes_by_cell: HashMap<IVec2, RiverSample> = HashMap::new();
    for &sample in samples {
        if sample.inlandness < args.land_threshold {
            continue;
        }
        let cell = sample_cell(sample.pos, step);
        if let Some(prev) = land_nodes_by_cell.get(&cell) {
            if sample.inlandness <= prev.inlandness {
                continue;
            }
        }
        land_nodes_by_cell.insert(cell, sample);
    }
    if land_nodes_by_cell.len() < 4 {
        return None;
    }

    let largest_cluster = largest_land_cluster(&land_nodes_by_cell, 1);
    if largest_cluster.len() < 4 {
        return None;
    }

    let (cluster_w, cluster_h) = cluster_size_tiles(&largest_cluster, &land_nodes_by_cell);
    let min_cells = ((args.min_landmass_width as i64 * args.min_landmass_height as i64)
        / (step as i64 * step as i64))
        .max(4) as usize;
    if cluster_w < args.min_landmass_width
        || cluster_h < args.min_landmass_height
        || largest_cluster.len() < min_cells
    {
        return None;
    }

    let (source_cell, mouth_cell) = pick_source_and_mouth(
        &largest_cluster,
        &land_nodes_by_cell,
        args.source_mouth_min_distance,
    )?;

    let seed = start_chunk.hash_value(settings, dimension_hash, offer_i ^ 0xA94F_E13D_17CB_2913);
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

    let mut main_cells = build_greedy_path(
        source_cell,
        mouth_cell,
        &land_nodes_by_cell,
        &mut rng,
        args.uphill_penalty,
    );
    if main_cells.len() < 2 {
        return None;
    }
    if *main_cells.last().unwrap_or(&source_cell) != mouth_cell {
        main_cells.push(mouth_cell);
    }
    dedup_cells_in_place(&mut main_cells);

    let mut main_points = Vec::with_capacity(main_cells.len());
    for cell in &main_cells {
        let Some(sample) = land_nodes_by_cell.get(cell) else {
            continue;
        };
        main_points.push(sample.pos);
    }
    if main_points.len() < 2 {
        return None;
    }

    let mut water_tiles: HashSet<GlobalTilePos> = HashSet::default();
    let mut main_meander_points: Vec<GlobalTilePos> = Vec::new();
    paint_meandering_path(
        &main_points,
        args.main_half_width_start,
        args.main_half_width_end,
        args.meander_amplitude,
        args.meander_frequency,
        args.jitter,
        &mut rng,
        &mut water_tiles,
        &mut main_meander_points,
    );

    if args.branch_count > 0 && main_cells.len() > 4 {
        for _ in 0..args.branch_count {
            let attach_i = rng.random_range(1..main_cells.len() - 1);
            let attach_cell = main_cells[attach_i];
            let Some(attach_sample) = land_nodes_by_cell.get(&attach_cell) else {
                continue;
            };
            let mut source_options: Vec<(IVec2, f32)> = Vec::new();
            for &cell in &largest_cluster {
                let Some(sample) = land_nodes_by_cell.get(&cell) else {
                    continue;
                };
                let dist_sq = sample.pos.distance_squared(&attach_sample.pos);
                if dist_sq < (args.branch_source_min_dist as u64).pow(2)
                    || dist_sq > (args.branch_source_max_dist as u64).pow(2)
                {
                    continue;
                }
                if sample.inlandness <= attach_sample.inlandness + args.tributary_min_inlandness_delta {
                    continue;
                }
                source_options.push((cell, sample.inlandness));
            }
            if source_options.is_empty() {
                continue;
            }
            source_options.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
            let top = source_options.len().min(8);
            let source_cell = source_options[rng.random_range(0..top)].0;
            let mut branch_cells = build_greedy_path(
                source_cell,
                attach_cell,
                &land_nodes_by_cell,
                &mut rng,
                args.uphill_penalty,
            );
            if branch_cells.len() < 2 {
                continue;
            }
            if *branch_cells.last().unwrap_or(&source_cell) != attach_cell {
                branch_cells.push(attach_cell);
            }
            dedup_cells_in_place(&mut branch_cells);

            let mut branch_points = Vec::with_capacity(branch_cells.len());
            for cell in &branch_cells {
                let Some(sample) = land_nodes_by_cell.get(cell) else {
                    continue;
                };
                branch_points.push(sample.pos);
            }
            if branch_points.len() < 2 {
                continue;
            }

            paint_meandering_path(
                &branch_points,
                1,
                args.main_half_width_start.max(1),
                (args.meander_amplitude * 0.45).max(0.5),
                (args.meander_frequency * 1.4).max(0.001),
                args.jitter * 0.8,
                &mut rng,
                &mut water_tiles,
                &mut Vec::new(),
            );
        }
    }

    if args.delta_branch_count > 0 && main_meander_points.len() >= 2 {
        let mouth = *main_meander_points.last().unwrap_or(&main_points[main_points.len() - 1]);
        let prev = main_meander_points[main_meander_points.len() - 2];
        let mut base_dir = (mouth.0 - prev.0).as_vec2();
        if base_dir.length_squared() < 0.01 {
            base_dir = (main_points[main_points.len() - 1].0 - main_points[0].0).as_vec2();
        }
        if base_dir.length_squared() < 0.01 {
            base_dir = Vec2::Y;
        }
        base_dir = base_dir.normalize();

        let spread_rad = args.delta_spread_deg.to_radians();
        for i in 0..args.delta_branch_count {
            let branch_t = if args.delta_branch_count <= 1 {
                0.0
            } else {
                (i as f32 / (args.delta_branch_count - 1) as f32) * 2.0 - 1.0
            };
            let random_ang = if spread_rad > 0.0 {
                rng.random_range(-spread_rad * 0.2..spread_rad * 0.2)
            } else {
                0.0
            };
            let angle = branch_t * spread_rad + random_ang;
            let dir = rotate_vec2(base_dir, angle).normalize_or_zero();
            if dir.length_squared() < 0.01 {
                continue;
            }
            let length = ((args.delta_length as f32) * rng.random_range(0.75..1.25)).round() as i32;
            let steps = length.max(8);
            let mut pos = mouth.0.as_vec2();
            for s in 0..steps {
                let t = s as f32 / steps as f32;
                pos += dir * 1.0;
                let p = GlobalTilePos::new(pos.x.round() as i32, pos.y.round() as i32);
                let w = lerp_i32(args.main_half_width_end.max(1), 1, t).max(1);
                paint_disk(p, w, &mut water_tiles);
            }
        }
    }

    let (min_chunk, max_chunk_excl) = region_pos.chunk_bounds();
    let min_tile = min_chunk.to_tilepos();
    let max_tile_excl = max_chunk_excl.to_tilepos();
    water_tiles.retain(|pos| {
        pos.0.x >= min_tile.0.x
            && pos.0.x < max_tile_excl.0.x
            && pos.0.y >= min_tile.0.y
            && pos.0.y < max_tile_excl.0.y
    });
    if water_tiles.is_empty() {
        return None;
    }

    let mut tiles_by_chunk: HashMap<ChunkPos, Vec<GlobalTilePos>> = HashMap::default();
    for pos in water_tiles {
        tiles_by_chunk.entry(pos.to_chunkpos()).or_default().push(pos);
    }
    for chunk_tiles in tiles_by_chunk.values_mut() {
        chunk_tiles.sort_unstable_by_key(|pos| (pos.0.y, pos.0.x));
    }

    let source_chunk = land_nodes_by_cell
        .get(&source_cell)
        .map(|sample| sample.pos.to_chunkpos())
        .unwrap_or(start_chunk);
    let mut claimed_chunks = tiles_by_chunk.keys().copied().collect::<Vec<_>>();
    claimed_chunks.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
    if let Some(source_i) = claimed_chunks.iter().position(|chunk| *chunk == source_chunk) {
        claimed_chunks.swap(0, source_i);
    }
    if claimed_chunks.is_empty() {
        return None;
    }

    Some(RiverPlan {
        claimed_chunks,
        tiles_by_chunk,
    })
}

fn sample_cell(pos: GlobalTilePos, step: i32) -> IVec2 {
    IVec2::new(pos.0.x.div_euclid(step), pos.0.y.div_euclid(step))
}

fn largest_land_cluster(
    land_nodes_by_cell: &HashMap<IVec2, RiverSample>,
    neighbor_radius_cells: i32,
) -> Vec<IVec2> {
    let mut visited: HashSet<IVec2> = HashSet::default();
    let mut largest: Vec<IVec2> = Vec::new();

    for &cell in land_nodes_by_cell.keys() {
        if visited.contains(&cell) {
            continue;
        }
        let mut cluster = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(cell);
        visited.insert(cell);

        while let Some(curr) = queue.pop_front() {
            cluster.push(curr);
            for ny in -neighbor_radius_cells..=neighbor_radius_cells {
                for nx in -neighbor_radius_cells..=neighbor_radius_cells {
                    if nx == 0 && ny == 0 {
                        continue;
                    }
                    let next = curr + IVec2::new(nx, ny);
                    if visited.contains(&next) {
                        continue;
                    }
                    if !land_nodes_by_cell.contains_key(&next) {
                        continue;
                    }
                    visited.insert(next);
                    queue.push_back(next);
                }
            }
        }

        if cluster.len() > largest.len() {
            largest = cluster;
        }
    }

    largest
}

fn cluster_size_tiles(cluster: &[IVec2], land_nodes_by_cell: &HashMap<IVec2, RiverSample>) -> (i32, i32) {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for cell in cluster {
        let Some(sample) = land_nodes_by_cell.get(cell) else {
            continue;
        };
        min_x = min_x.min(sample.pos.0.x);
        min_y = min_y.min(sample.pos.0.y);
        max_x = max_x.max(sample.pos.0.x);
        max_y = max_y.max(sample.pos.0.y);
    }
    if min_x > max_x || min_y > max_y {
        return (0, 0);
    }
    (max_x - min_x + 1, max_y - min_y + 1)
}

fn pick_source_and_mouth(
    cluster: &[IVec2],
    land_nodes_by_cell: &HashMap<IVec2, RiverSample>,
    min_distance_sq: u64,
) -> Option<(IVec2, IVec2)> {
    let mut by_height_desc = cluster
        .iter()
        .copied()
        .collect::<Vec<_>>();
    by_height_desc.sort_unstable_by(|a, b| {
        let av = land_nodes_by_cell.get(a).map(|s| s.inlandness).unwrap_or(f32::NEG_INFINITY);
        let bv = land_nodes_by_cell.get(b).map(|s| s.inlandness).unwrap_or(f32::NEG_INFINITY);
        bv.total_cmp(&av)
    });
    let mut by_height_asc = by_height_desc.clone();
    by_height_asc.reverse();
    if by_height_desc.is_empty() || by_height_asc.is_empty() {
        return None;
    }

    let source = by_height_desc[0];
    let mouth = by_height_asc[0];
    let source_pos = land_nodes_by_cell.get(&source)?.pos;
    let mouth_pos = land_nodes_by_cell.get(&mouth)?.pos;
    if source_pos.distance_squared(&mouth_pos) >= min_distance_sq {
        return Some((source, mouth));
    }

    let highs = by_height_desc.into_iter().take(12).collect::<Vec<_>>();
    let lows = by_height_asc.into_iter().take(12).collect::<Vec<_>>();
    let mut best: Option<(IVec2, IVec2, u64)> = None;
    for high in highs {
        let Some(high_sample) = land_nodes_by_cell.get(&high) else {
            continue;
        };
        for low in &lows {
            let Some(low_sample) = land_nodes_by_cell.get(low) else {
                continue;
            };
            let dist_sq = high_sample.pos.distance_squared(&low_sample.pos);
            if best.map(|(_, _, best_dist)| dist_sq > best_dist).unwrap_or(true) {
                best = Some((high, *low, dist_sq));
            }
        }
    }

    best.map(|(s, m, _)| (s, m))
}

fn build_greedy_path(
    source_cell: IVec2,
    target_cell: IVec2,
    land_nodes_by_cell: &HashMap<IVec2, RiverSample>,
    rng: &mut rand_pcg::Pcg64Mcg,
    uphill_penalty: f32,
) -> Vec<IVec2> {
    let mut path = Vec::new();
    let mut visited: HashSet<IVec2> = HashSet::default();
    let mut current = source_cell;
    path.push(current);
    visited.insert(current);

    let max_steps = land_nodes_by_cell.len().min(6000).max(64);
    for _ in 0..max_steps {
        if current == target_cell {
            break;
        }
        let Some(current_sample) = land_nodes_by_cell.get(&current) else {
            break;
        };

        let mut candidate: Option<(IVec2, f32)> = None;
        let mut fallback: Option<(IVec2, i32)> = None;
        for ny in -1..=1 {
            for nx in -1..=1 {
                if nx == 0 && ny == 0 {
                    continue;
                }
                let next = current + IVec2::new(nx, ny);
                let Some(next_sample) = land_nodes_by_cell.get(&next) else {
                    continue;
                };

                let dist_score = (target_cell - next).abs().max_element() as f32 * 0.02;
                let uphill = (next_sample.inlandness - current_sample.inlandness).max(0.0);
                let downhill = (current_sample.inlandness - next_sample.inlandness).max(0.0);
                let score = uphill * uphill_penalty
                    + dist_score
                    - downhill * 0.1
                    + rng.random_range(0.0..0.1);

                if !visited.contains(&next)
                    && candidate
                        .as_ref()
                        .map(|(_, best_score)| score < *best_score)
                        .unwrap_or(true)
                {
                    candidate = Some((next, score));
                }

                let manhattan = (target_cell - next).abs().x + (target_cell - next).abs().y;
                if fallback
                    .as_ref()
                    .map(|(_, best_dist)| manhattan < *best_dist)
                    .unwrap_or(true)
                {
                    fallback = Some((next, manhattan));
                }
            }
        }

        let next = candidate
            .map(|(cell, _)| cell)
            .or_else(|| fallback.map(|(cell, _)| cell));
        let Some(next) = next else {
            break;
        };
        if next == current {
            break;
        }
        current = next;
        path.push(current);
        visited.insert(current);
    }

    if *path.last().unwrap_or(&source_cell) != target_cell {
        let mut current = *path.last().unwrap_or(&source_cell);
        for _ in 0..512 {
            if current == target_cell {
                break;
            }
            let delta = target_cell - current;
            let direct = current + IVec2::new(delta.x.signum(), delta.y.signum());
            let next = if land_nodes_by_cell.contains_key(&direct) {
                direct
            } else {
                let mut best_neighbor: Option<(IVec2, i32)> = None;
                for ny in -1..=1 {
                    for nx in -1..=1 {
                        if nx == 0 && ny == 0 {
                            continue;
                        }
                        let cand = current + IVec2::new(nx, ny);
                        if !land_nodes_by_cell.contains_key(&cand) {
                            continue;
                        }
                        let dist = (target_cell - cand).abs().x + (target_cell - cand).abs().y;
                        if best_neighbor
                            .as_ref()
                            .map(|(_, best)| dist < *best)
                            .unwrap_or(true)
                        {
                            best_neighbor = Some((cand, dist));
                        }
                    }
                }
                best_neighbor.map(|(cand, _)| cand).unwrap_or(current)
            };
            if next == current {
                break;
            }
            current = next;
            path.push(current);
        }
    }

    dedup_cells_in_place(&mut path);
    path
}

fn dedup_cells_in_place(cells: &mut Vec<IVec2>) {
    if cells.len() < 2 {
        return;
    }
    let mut write = 1usize;
    for read in 1..cells.len() {
        if cells[read] == cells[write - 1] {
            continue;
        }
        cells[write] = cells[read];
        write += 1;
    }
    cells.truncate(write);
}

fn bresenham_segment(start: GlobalTilePos, end: GlobalTilePos, out: &mut Vec<GlobalTilePos>) {
    let mut x0 = start.0.x;
    let mut y0 = start.0.y;
    let x1 = end.0.x;
    let y1 = end.0.y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        out.push(GlobalTilePos::new(x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn paint_meandering_path(
    coarse_points: &[GlobalTilePos],
    half_width_start: i32,
    half_width_end: i32,
    meander_amplitude: f32,
    meander_frequency: f32,
    jitter: f32,
    rng: &mut rand_pcg::Pcg64Mcg,
    out_tiles: &mut HashSet<GlobalTilePos>,
    out_path_points: &mut Vec<GlobalTilePos>,
) {
    if coarse_points.len() < 2 {
        return;
    }
    let phase = rng.random_range(0.0..std::f32::consts::TAU);
    let seg_count = (coarse_points.len() - 1) as f32;

    for seg_i in 0..coarse_points.len() - 1 {
        let a = coarse_points[seg_i];
        let b = coarse_points[seg_i + 1];
        let mut segment_points = Vec::new();
        bresenham_segment(a, b, &mut segment_points);
        if segment_points.is_empty() {
            continue;
        }
        let mut dir = (b.0 - a.0).as_vec2();
        if dir.length_squared() < 0.01 {
            dir = Vec2::X;
        } else {
            dir = dir.normalize();
        }
        let perp = Vec2::new(-dir.y, dir.x);
        let seg_len_f = segment_points.len().max(1) as f32;

        for (point_i, point) in segment_points.into_iter().enumerate() {
            let t = (seg_i as f32 + point_i as f32 / seg_len_f) / seg_count.max(1.0);
            let amp = meander_amplitude * (0.3 + 0.7 * t);
            let wobble = ((t * std::f32::consts::TAU * meander_frequency) + phase).sin() * amp;
            let jitter_val = if jitter > 0.0 {
                rng.random_range(-jitter..jitter)
            } else {
                0.0
            };
            let shifted = point.0.as_vec2() + perp * (wobble + jitter_val);
            let meandered = GlobalTilePos::new(
                shifted.x.round() as i32,
                shifted.y.round() as i32,
            );

            let half_width = lerp_i32(half_width_start, half_width_end, t).max(1);
            paint_disk(meandered, half_width, out_tiles);

            if out_path_points.last().copied() != Some(meandered) {
                out_path_points.push(meandered);
            }
        }
    }
}

fn paint_disk(center: GlobalTilePos, half_width: i32, out_tiles: &mut HashSet<GlobalTilePos>) {
    let radius_sq = half_width * half_width;
    for dy in -half_width..=half_width {
        for dx in -half_width..=half_width {
            if dx * dx + dy * dy > radius_sq {
                continue;
            }
            out_tiles.insert(center + IVec2::new(dx, dy));
        }
    }
}

fn lerp_i32(start: i32, end: i32, t: f32) -> i32 {
    let t = t.clamp(0.0, 1.0);
    (start as f32 + (end as f32 - start as f32) * t).round() as i32
}

fn rotate_vec2(v: Vec2, radians: f32) -> Vec2 {
    let sin = radians.sin();
    let cos = radians.cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}
