use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use being::{
    being_bundles::BeingBundle,
    being_inst_template::being_inst_template_resources::BitRef,
    pack::pack_components::{Pack, PackBeingSampler, PackInitialSize},
    race::{race_components::Race, race_resources::RaceRef},
};
use being_shared::BeingInstTemplate;
use common::log_targets::WILDLIFE_SYSTEM;
use game_common::{
    game_common_samplers::{BiomePackCountMultiplierStats, MacroChunkBiomeTagDistributionMap},
    game_common_timers::TimerComp,
};
use param_sets::BlockingTileParamSet;
use rand_distr::{Distribution, Normal};
use tilemap::chunking::chunking_components::{Chunk, TerrGenState};
use tilemap::terrain::biome::biome_components::BiomePackSampler;
use tilemap_shared::{
    ChunkPos,
    DimensionRef,
    GlobalTilePos,
    MacroChunkPos,
    MACRO_CHUNK_SIZE_IN_CHUNKS,
};

const PACK_CLUSTER_RADIUS_TILES: i32 = 7;
const PACK_CENTER_MIN_SEPARATION_TILES: i32 = 22;
const PACK_CENTER_SAMPLE_ATTEMPTS: usize = 24;
const PACK_MEMBER_SAMPLE_ATTEMPTS: usize = 24;

pub fn spawn_natural_wildlife_for_chunk(
    mut cmd: Commands,
    launched_chunk_query: Query<
        (&DimensionRef, &ChunkPos, &TerrGenState),
        (With<Chunk>, Changed<TerrGenState>),
    >,
    biome_pack_samplers: Query<&BiomePackSampler>,
    pack_being_samplers: Query<&PackBeingSampler>,
    blocking_tiles: BlockingTileParamSet,
    mut pending_macro_chunks: Local<HashMap<(DimensionRef, MacroChunkPos), TimerComp>>,
    mut ready_macro_chunks: Local<Vec<(DimensionRef, MacroChunkPos)>>,
    mut macro_chunk_biome_tag_dist: ResMut<MacroChunkBiomeTagDistributionMap>,
    spawn_target_query: Query<(Has<BeingInstTemplate>, Has<Race>, Has<Pack>)>,
    spawn_pack_size_query: Query<&PackInitialSize>,
    mut blocking_tiles_to_drain: Local<Vec<Entity>>,
    time: Res<Time>,
) {
    if launched_chunk_query.is_empty() && pending_macro_chunks.is_empty() {
        return;
    }
    for (&dim_ref, &chunk_pos, terrgen_state) in launched_chunk_query.iter() {
        if *terrgen_state != TerrGenState::OpsLaunched {
            continue;
        }
        pending_macro_chunks
            .entry((dim_ref, chunk_pos.to_macrochunk_pos()))
            .or_insert_with(|| TimerComp::new(4.0));
    }
    if pending_macro_chunks.is_empty() {
        return;
    }

    ready_macro_chunks.clear();
    ready_macro_chunks.reserve(pending_macro_chunks.len());
    for (&key, timer_comp) in pending_macro_chunks.iter_mut() {
        timer_comp.0.tick(time.delta());
        if timer_comp.0.is_finished() {
            ready_macro_chunks.push(key);
        }
    }
    if ready_macro_chunks.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    for key in ready_macro_chunks.drain(..) {
        pending_macro_chunks.remove(&key);

        let (dim_ref, macro_chunk_pos) = key;
        let Some(distribution) = macro_chunk_biome_tag_dist.0.get(&key) else {
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn found no biome distribution for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };

        let Some(biome_ent) = distribution.biome_sampler.sample_with_rng(&mut rng) else {
            macro_chunk_biome_tag_dist.0.remove(&key);
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn found no weighted biome for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let Ok(biome_pack_sampler) = biome_pack_samplers.get(biome_ent) else {
            macro_chunk_biome_tag_dist.0.remove(&key);
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn found no pack sampler for biome {:?} in macrochunk {} in {:?}", biome_ent, macro_chunk_pos, dim_ref);
            continue;
        };
        let pack_count = sample_pack_count_multiplier(
            distribution.averaged_pack_count_multiplier_stats(biome_ent),
            &mut rng,
        );
        let mut used_positions: HashSet<GlobalTilePos> = HashSet::default();
        let mut pack_centers: Vec<GlobalTilePos> = Vec::with_capacity(pack_count);
        let mut spawned_packs = 0usize;
        let mut spawned_beings = 0usize;

        for _ in 0..pack_count {
            let Some(pack_target) = biome_pack_sampler.0.sample_with_rng(&mut rng) else {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no further candidate wildlife for biome {:?} in macrochunk {} in {:?}", biome_ent, macro_chunk_pos, dim_ref);
                break;
            };
            let Ok((_, _, is_pack)) = spawn_target_query.get(pack_target) else {
                warn!(target: WILDLIFE_SYSTEM, "Natural spawn target {:?} for macrochunk {} in {:?} is neither bit, race, nor pack", pack_target, macro_chunk_pos, dim_ref);
                continue;
            };
            let pack_size = spawn_pack_size_query
                .get(pack_target)
                .ok()
                .map(|dist| dist.sample_count(&mut rng))
                .unwrap_or(1);
            let sampled_beings = if is_pack {
                let Ok(pack_being_sampler) = pack_being_samplers.get(pack_target) else {
                    trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no being sampler for pack {:?} in macrochunk {} in {:?}", pack_target, macro_chunk_pos, dim_ref);
                    continue;
                };
                let sampled_beings = sample_pack_beings(pack_being_sampler, pack_size, &mut rng);
                if sampled_beings.is_empty() {
                    trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no beings to sample from pack {:?} in macrochunk {} in {:?}", pack_target, macro_chunk_pos, dim_ref);
                    continue;
                }
                sampled_beings
            } else {
                vec![pack_target; pack_size]
            };
            let Some(pack_center) = sample_pack_center(
                &blocking_tiles,
                &mut blocking_tiles_to_drain,
                dim_ref,
                macro_chunk_pos,
                &pack_centers,
                &mut rng,
            ) else {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn ran out of pack centers for biome {:?} in macrochunk {} in {:?}", biome_ent, macro_chunk_pos, dim_ref);
                break;
            };
            let spawn_positions = sample_pack_positions(
                &blocking_tiles,
                &mut blocking_tiles_to_drain,
                dim_ref,
                macro_chunk_pos,
                pack_center,
                sampled_beings.len(),
                &mut used_positions,
                &mut rng,
            );
            if spawn_positions.is_empty() {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no valid positions for target {:?} in macrochunk {} in {:?}", pack_target, macro_chunk_pos, dim_ref);
                continue;
            }

            pack_centers.push(pack_center);
            spawned_packs += 1;
            spawned_beings += spawn_positions.len();
            for (gpos, spawn_target) in spawn_positions.into_iter().zip(sampled_beings.into_iter()) {
                let Ok((is_bit, is_race, _)) = spawn_target_query.get(spawn_target) else {
                    warn!(target: WILDLIFE_SYSTEM, "Natural spawn member target {:?} for macrochunk {} in {:?} is neither bit nor race", spawn_target, macro_chunk_pos, dim_ref);
                    continue;
                };
                let entity = cmd.spawn(BeingBundle::new(dim_ref, gpos)).id();
                if is_bit {
                    cmd.entity(entity).insert(BitRef(spawn_target));
                } else if is_race {
                    cmd.entity(entity).insert(RaceRef(spawn_target));
                }
            }
        }

        if spawned_beings == 0 {
            macro_chunk_biome_tag_dist.0.remove(&key);
            continue;
        }

        macro_chunk_biome_tag_dist.0.remove(&key);
        debug!(target: WILDLIFE_SYSTEM, "Natural spawn seeded macrochunk {} in {:?} with biome {:?}, packs {}, beings {}", macro_chunk_pos, dim_ref, biome_ent, spawned_packs, spawned_beings);
    }
}

fn sample_pack_count_multiplier(
    stats: BiomePackCountMultiplierStats,
    rng: &mut impl rand::Rng,
) -> usize {
    let mean = stats.averaged_mean().max(0.0);
    let std_dev = stats.averaged_std_dev().max(0.0);
    if std_dev <= f32::EPSILON {
        return sample_fractional_pack_count(mean, rng);
    }
    let Ok(dist) = Normal::new(mean, std_dev.max(0.01)) else {
        return sample_fractional_pack_count(mean, rng);
    };
    sample_fractional_pack_count(dist.sample(rng).max(0.0), rng)
}

fn sample_fractional_pack_count(
    sampled_multiplier: f32,
    rng: &mut impl rand::Rng,
) -> usize {
    if sampled_multiplier <= 0.0 {
        return 0;
    }
    let guaranteed_packs = sampled_multiplier.floor() as usize;
    let extra_pack_probability = sampled_multiplier.fract();
    guaranteed_packs + usize::from(rng.random::<f32>() < extra_pack_probability)
}

fn sample_pack_beings(
    pack_being_sampler: &PackBeingSampler,
    sample_count: usize,
    rng: &mut impl rand::Rng,
) -> Vec<Entity> {
    let mut out = Vec::with_capacity(sample_count);
    while out.len() < sample_count {
        let Some(being) = pack_being_sampler.0.sample_with_rng(rng) else {
            break;
        };
        out.push(being);
    }
    out
}

fn sample_pack_center(
    blocking_tiles: &BlockingTileParamSet,
    to_drain: &mut Vec<Entity>,
    dim_ref: DimensionRef,
    macro_chunk_pos: MacroChunkPos,
    existing_centers: &[GlobalTilePos],
    rng: &mut impl rand::Rng,
) -> Option<GlobalTilePos> {
    let min_tile = macro_chunk_pos.to_chunkpos().to_tilepos().0;
    let macro_size = MACRO_CHUNK_SIZE_IN_CHUNKS.0 * ChunkPos::CHUNK_SIZE.as_ivec2();
    let max_tile_excl = min_tile + macro_size;
    for _ in 0..PACK_CENTER_SAMPLE_ATTEMPTS {
        let candidate = GlobalTilePos(IVec2::new(
            rng.random_range(min_tile.x..max_tile_excl.x),
            rng.random_range(min_tile.y..max_tile_excl.y),
        ));
        if blocking_tiles.is_blocked_at(
            to_drain,
            dim_ref,
            candidate,
            Entity::PLACEHOLDER,
        ) {
            continue;
        }
        if existing_centers.iter().all(|center| {
            (center.0 - candidate.0)
                .abs()
                .max_element()
                >= PACK_CENTER_MIN_SEPARATION_TILES
        }) {
            return Some(candidate);
        }
    }
    None
}

fn sample_pack_positions(
    blocking_tiles: &BlockingTileParamSet,
    to_drain: &mut Vec<Entity>,
    dim_ref: DimensionRef,
    macro_chunk_pos: MacroChunkPos,
    center: GlobalTilePos,
    pack_size: usize,
    used_positions: &mut HashSet<GlobalTilePos>,
    rng: &mut impl rand::Rng,
) -> Vec<GlobalTilePos> {
    let mut out = Vec::with_capacity(pack_size);
    if !blocking_tiles.is_blocked_at(
        to_drain,
        dim_ref,
        center,
        Entity::PLACEHOLDER,
    ) && used_positions.insert(center) {
        out.push(center);
    }

    let min_tile = macro_chunk_pos.to_chunkpos().to_tilepos().0;
    let macro_size = MACRO_CHUNK_SIZE_IN_CHUNKS.0 * ChunkPos::CHUNK_SIZE.as_ivec2();
    let max_tile_excl = min_tile + macro_size;

    while out.len() < pack_size {
        let mut placed = false;
        for _ in 0..PACK_MEMBER_SAMPLE_ATTEMPTS {
            let offset = IVec2::new(
                rng.random_range(-PACK_CLUSTER_RADIUS_TILES..=PACK_CLUSTER_RADIUS_TILES),
                rng.random_range(-PACK_CLUSTER_RADIUS_TILES..=PACK_CLUSTER_RADIUS_TILES),
            );
            let candidate = GlobalTilePos(center.0 + offset);
            if candidate.0.x < min_tile.x
                || candidate.0.y < min_tile.y
                || candidate.0.x >= max_tile_excl.x
                || candidate.0.y >= max_tile_excl.y
            {
                continue;
            }
            if blocking_tiles.is_blocked_at(
                to_drain,
                dim_ref,
                candidate,
                Entity::PLACEHOLDER,
            ) || !used_positions.insert(candidate) {
                continue;
            }
            out.push(candidate);
            placed = true;
            break;
        }
        if placed {
            continue;
        }
        let mut fallback = macro_chunk_pos.random_unique_gposes(1, rng);
        let Some(candidate) = fallback.pop() else {
            break;
        };
        if blocking_tiles.is_blocked_at(
            to_drain,
            dim_ref,
            candidate,
            Entity::PLACEHOLDER,
        ) {
            continue;
        }
        if used_positions.insert(candidate) {
            out.push(candidate);
            continue;
        }
        break;
    }
    out
}
