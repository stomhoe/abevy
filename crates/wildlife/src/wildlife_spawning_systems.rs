use bevy::ecs::entity_disabling::Disabled;
use bevy::{
    ecs::{entity::{EntityHashMap, EntityHashSet}, system::SystemParam},
    platform::collections::HashSet,
    prelude::*,
};
use being::{
    being_bundles::BeingBundle,
    being_inst_template::being_inst_template_resources::BitRef,
    pack::pack_components::*,
    race::{race_components::Race, race_resources::RaceRef},
};
use ::being_shared::*;
use common::file_logging::file_log;
use common::common_components::StrId;
use common::log_targets::WILDLIFE_SYSTEM;
use ::game_common::*;
use movement::movement_components::GridLockedMovement;
use param_sets::BlockingTileParamSet;
use tilemap::terrain::biome::biome_components::CreatureSampler;
use tilemap::{
    chunking::{
        chunking_components::{Chunk, MacroChunk},
        macro_chunk_components::{BiomeDistribution, MacroChunkBiomePendingSampleState},
    },
    terrain::terrgen_messages::*,
};

use ::tilemap_shared::*;

use crate::wildlife_resources::*;

const DEFAULT_MIN_INBETWEEN_PACK_CENTER_CHUNKS: u8 = 1;
const NATURAL_PACK_CENTER_SAMPLE_ATTEMPTS: usize = 16;

pub const WATCHED_CHUNK_OFFSETS: [IVec2; 9] = [
    IVec2::new(0, 0),
    IVec2::new(0, 1),
    IVec2::new(0, -1),
    IVec2::new(1, 0),
    IVec2::new(-1, 0),
    IVec2::new(1, 1),
    IVec2::new(-1, 1),
    IVec2::new(1, -1),
    IVec2::new(-1, -1),
];

#[derive(Clone, Copy)]
struct SampledNaturalPackCenter {
    pack_target: Entity,
    relation_target: Entity,
    center_chunk: ChunkPos,
}

#[derive(SystemParam)]
pub struct NaturalWildlifeSpawnQueries<'w, 's> {
    macro_chunk_query: Query<'w, 's, (&'static DimensionRef, &'static MacroChunkPos, &'static BiomeDistribution), With<MacroChunk>>,
    macro_chunk_biome_sampling_states: Query<'w, 's, &'static MacroChunkBiomePendingSampleState>,
    biome_pack_samplers: Query<'w, 's, &'static CreatureSampler>,
    pack_being_samplers: Query<'w, 's, &'static PackBeingSampler>,
    pack_min_dists_query: Query<'w, 's, &'static PackMinDistsToPacksOrRaces>,
    bit_race_query: Query<'w, 's, &'static RaceRef>,
    spawn_target_query: Query<'w, 's, (Has<BeingInstTemplate>, Has<Race>, Has<Pack>, Has<NoSpawnGroup>), With<TemplEnti>>,
    spawn_pack_size_query: Query<'w, 's, &'static PackInitialSize>,
}

#[derive(SystemParam)]
pub struct NaturalWildlifeSpawnRes<'w> {
    pending_wildlife_by_chunk: ResMut<'w, NaturalSpawnReservationIndex>,
    seeded_macrochunks: ResMut<'w, SeededNaturalWildlifeMacroChunks>,
    request_writer: MessageWriter<'w, RequestMacroChunkBiomeSampling>,
}

#[derive(SystemParam)]
pub struct NaturalWildlifeSpawnLocals<'s> {
    macrochunks_to_seed: Local<'s, Vec<Entity>>,
    macrochunks_seen: Local<'s, EntityHashSet>,
    requests: Local<'s, Vec<RequestMacroChunkBiomeSampling>>,
    sampled_beings: Local<'s, Vec<Entity>>,
    pack_centers: Local<'s, Vec<SampledNaturalPackCenter>>,
}

fn choose_prioritized_pack_center_chunk(
    distribution: &BiomeDistribution,
    biome_ent: Entity,
    current_target: Entity,
    current_relation_target: Entity,
    macro_chunk_pos: MacroChunkPos,
    existing_centers: &[SampledNaturalPackCenter],
    pack_min_dists: Option<&PackMinDistsToPacksOrRaces>,
    pack_min_dists_query: &Query<&PackMinDistsToPacksOrRaces>,
    rng: &mut impl rand::Rng,
) -> Option<ChunkPos> {
    let mut seen_candidates = HashSet::<ChunkPos>::default();
    let sorted_candidates = distribution.sorted_chunk_candidates_for_biome(biome_ent);
    for preferred_chunk in sorted_candidates {
        for offset in WATCHED_CHUNK_OFFSETS {
            let candidate = ChunkPos(preferred_chunk.0 + offset);
            if !macro_chunk_pos.contains_chunkpos(candidate) || !seen_candidates.insert(candidate) {
                continue;
            }
            if is_pack_center_candidate_valid(
                candidate,
                current_target,
                current_relation_target,
                existing_centers,
                pack_min_dists,
                pack_min_dists_query,
            ) {
                return Some(candidate);
            }
        }
    }

    let min_chunk = macro_chunk_pos.to_chunkpos().0;
    let max_chunk_excl = min_chunk + MacroChunkPos::SIZE_IN_CHUNKS.0;
    for _ in 0..NATURAL_PACK_CENTER_SAMPLE_ATTEMPTS {
        let candidate = ChunkPos(IVec2::new(
            rng.random_range(min_chunk.x..max_chunk_excl.x),
            rng.random_range(min_chunk.y..max_chunk_excl.y),
        ));
        if is_pack_center_candidate_valid(
            candidate,
            current_target,
            current_relation_target,
            existing_centers,
            pack_min_dists,
            pack_min_dists_query,
        ) {
            return Some(candidate);
        }
    }

    for y in min_chunk.y..max_chunk_excl.y {
        for x in min_chunk.x..max_chunk_excl.x {
            let candidate = ChunkPos(IVec2::new(x, y));
            if is_pack_center_candidate_valid(
                candidate,
                current_target,
                current_relation_target,
                existing_centers,
                pack_min_dists,
                pack_min_dists_query,
            ) {
                return Some(candidate);
            }
        }
    }
    None
}

fn is_pack_center_candidate_valid(
    candidate: ChunkPos,
    current_target: Entity,
    current_relation_target: Entity,
    existing_centers: &[SampledNaturalPackCenter],
    pack_min_dists: Option<&PackMinDistsToPacksOrRaces>,
    pack_min_dists_query: &Query<&PackMinDistsToPacksOrRaces>,
) -> bool {
    existing_centers.iter().all(|existing_center| {
        let mut required_inbetween_chunks = pack_min_dists
            .map(|pack_min_dists| pack_min_dists.min_inbetween_chunks(existing_center.relation_target))
            .unwrap_or(DEFAULT_MIN_INBETWEEN_PACK_CENTER_CHUNKS);
        if let Ok(existing_pack_min_dists) = pack_min_dists_query.get(existing_center.pack_target) {
            if let Some(reciprocal_required_inbetween_chunks) = existing_pack_min_dists
                .configured_min_inbetween_chunks(current_relation_target)
            {
                required_inbetween_chunks = required_inbetween_chunks.max(reciprocal_required_inbetween_chunks);
            }
        }
        if existing_center.pack_target == current_target {
            required_inbetween_chunks = required_inbetween_chunks.max(1);
        }
        (existing_center.center_chunk.0 - candidate.0)
            .abs()
            .max_element()
            >= i32::from(required_inbetween_chunks) + 1
    })
}

fn spawn_relation_target(
    spawn_target: Entity,
    spawns_pack_entity: bool,
    is_race: bool,
    bit_race_query: &Query<&RaceRef>,
) -> Entity {
    if spawns_pack_entity || is_race {
        return spawn_target;
    }
    bit_race_query.get(spawn_target).map(|race_ref| race_ref.0).unwrap_or(spawn_target)
}

fn collect_macrochunk_pack_scores(
    distribution: &BiomeDistribution,
    biome_pack_samplers: &Query<&CreatureSampler>,
) -> Vec<(Entity, f32)> {
    let mut scores = EntityHashMap::default();
    for &(biome_ent, biome_weight) in distribution.produced_biome_sampler.iter() {
        let Ok(biome_pack_sampler) = biome_pack_samplers.get(biome_ent) else {
            continue;
        };
        for (&pack_ent, affinity) in biome_pack_sampler.0.iter() {
            let score = biome_weight * *affinity;
            if score == 0.0 {
                continue;
            }
            *scores.entry(pack_ent).or_insert(0.0) += score;
        }
    }
    scores
        .into_iter()
        .filter_map(|(pack_ent, score)| (score > 0.0).then_some((pack_ent, score)))
        .collect()
}

fn sample_macrochunk_pack_target(
    distribution: &BiomeDistribution,
    biome_pack_samplers: &Query<&CreatureSampler>,
    rng: &mut impl rand::Rng,
) -> Option<Entity> {
    let pack_scores = collect_macrochunk_pack_scores(distribution, biome_pack_samplers);
    if pack_scores.is_empty() {
        return None;
    }
    tilemap_shared::EntityWeightedSampler::new(&pack_scores).sample_with_rng(rng)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PendingNaturalSpawnUnfreeze;

#[derive(Component, Debug, Clone, Copy)]
pub struct NaturalSpawnOrigin(pub ChunkPos);

pub fn cleanup_pending_natural_wildlife_index(
    mut removed_reservations: RemovedComponents<PendingNaturalSpawnUnfreeze>,
    mut pending_wildlife_by_chunk: ResMut<NaturalSpawnReservationIndex>,
) {
    for being_ent in removed_reservations.read() {
        pending_wildlife_by_chunk.remove_being(being_ent);
    }
}

#[allow(unused_parens, )]
pub fn spawn_natural_wildlife_for_macro_chunk(
    mut cmd: Commands,
    mut loaded_macrochunks: MessageReader<MacroChunkLoaded>,
    mut removed_macro_chunk_biome_pending_sample_states: RemovedComponents<MacroChunkBiomePendingSampleState>,
    spawn_queries: NaturalWildlifeSpawnQueries,
    mut spawn_res: NaturalWildlifeSpawnRes,
    mut spawn_locals: NaturalWildlifeSpawnLocals,
) {
    spawn_locals.macrochunks_seen.clear();
    spawn_locals.requests.clear();

    for loaded_macrochunk in loaded_macrochunks.read() {
        let Ok((&dim_ref, &macro_chunk_pos, _)) = spawn_queries.macro_chunk_query.get(loaded_macrochunk.macro_chunk_ent) else {
            continue;
        };
        if spawn_res.seeded_macrochunks.0.contains(&(dim_ref, macro_chunk_pos)) {
            continue;
        }
        match spawn_queries.macro_chunk_biome_sampling_states.get(loaded_macrochunk.macro_chunk_ent) {
            Ok(biome_sampling_state) => {
                if matches!(*biome_sampling_state, MacroChunkBiomePendingSampleState::Unsampled) {
                    spawn_locals.requests.push(RequestMacroChunkBiomeSampling {
                        macro_chunk_ent: loaded_macrochunk.macro_chunk_ent,
                    });
                }
            }
            Err(_) => {
                if spawn_locals.macrochunks_seen.insert(loaded_macrochunk.macro_chunk_ent) {
                    spawn_locals.macrochunks_to_seed.push(loaded_macrochunk.macro_chunk_ent);
                }
            }
        }
    }
    spawn_res.request_writer.write_batch(spawn_locals.requests.drain(..));

    for removed_macrochunk_ent in removed_macro_chunk_biome_pending_sample_states.read() {
        if spawn_locals.macrochunks_seen.insert(removed_macrochunk_ent) {
            spawn_locals.macrochunks_to_seed.push(removed_macrochunk_ent);
        }
    }
    let mut rng = rand::rng();
    for macro_chunk_ent in spawn_locals.macrochunks_to_seed.drain(..) {
        let Ok((&dim_ref, &macro_chunk_pos, distribution)) = spawn_queries.macro_chunk_query.get(macro_chunk_ent) else {
            continue;
        };
        if spawn_res.seeded_macrochunks.0.contains(&(dim_ref, macro_chunk_pos)) {
            trace!(target: WILDLIFE_SYSTEM, "Skipping natural spawn reseed for already-seeded macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        }
        let Some(biome_ent) = distribution.produced_biome_sampler.sample_with_rng(&mut rng) else {
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn found no weighted biome for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let pack_count = distribution
            .averaged_pack_count_multiplier_stats(biome_ent)
            .sample_pack_count_multiplier(&mut rng);
        let strongest_center_chunk = distribution.predominant_chunk_for_biome(biome_ent);
        file_log(
            "wildlife",
            "host",
            &format!(
                "seed macrochunk={macro_chunk_pos} dim={dim_ref:?} biome={biome_ent:?} pack_count={pack_count} strongest_center={strongest_center_chunk:?}"
            ),
        );
        spawn_locals.pack_centers.clear();
        spawn_locals.pack_centers.reserve(pack_count);
        let mut spawned_packs = 0usize;
        let mut spawned_beings = 0usize;

        for _ in 0..pack_count {
            let Some(pack_target) = sample_macrochunk_pack_target(distribution, &spawn_queries.biome_pack_samplers, &mut rng) else {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no candidate wildlife for macrochunk {} in {:?} after affinity filtering", macro_chunk_pos, dim_ref);
                break;
            };
            let Ok((_, is_race, is_pack, has_no_spawn_group)) = spawn_queries.spawn_target_query.get(pack_target) else {
                warn!(target: WILDLIFE_SYSTEM, "Natural spawn target {:?} for macrochunk {} in {:?} is neither bit, race, nor pack", pack_target, macro_chunk_pos, dim_ref);
                continue;
            };
            let pack_size = spawn_queries.spawn_pack_size_query
                .get(pack_target)
                .ok()
                .map(|dist| dist.sample_count(&mut rng))
                .unwrap_or(1);
            let spawns_pack_entity = pack_size > 1 && !has_no_spawn_group;
            spawn_locals.sampled_beings.clear();
            if is_pack {
                let Ok(pack_being_sampler) = spawn_queries.pack_being_samplers.get(pack_target) else {
                    trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no being sampler for pack {:?} in macrochunk {} in {:?}", pack_target, macro_chunk_pos, dim_ref);
                    continue;
                };
                pack_being_sampler.0.sample_n_with_rng(pack_size, &mut rng, &mut *spawn_locals.sampled_beings);
                if spawn_locals.sampled_beings.is_empty() {
                    trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no beings to sample from pack {:?} in macrochunk {} in {:?}", pack_target, macro_chunk_pos, dim_ref);
                    continue;
                }
            } else {
                spawn_locals.sampled_beings.resize(pack_size, pack_target);
            }
            let relation_target = spawn_relation_target(pack_target, spawns_pack_entity, is_race, &spawn_queries.bit_race_query);
            let pack_min_dists = spawn_queries.pack_min_dists_query.get(pack_target).ok();
            let Some(pack_center_chunk) = choose_prioritized_pack_center_chunk(
                distribution,
                biome_ent,
                pack_target,
                relation_target,
                macro_chunk_pos,
                &spawn_locals.pack_centers,
                pack_min_dists,
                &spawn_queries.pack_min_dists_query,
                &mut rng,
            ) else {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn ran out of pack centers for biome {:?} in macrochunk {} in {:?}", biome_ent, macro_chunk_pos, dim_ref);
                break;
            };
            let spawn_chunks = Pack::sample_pack_member_chunks(
                macro_chunk_pos,
                pack_center_chunk,
                spawn_locals.sampled_beings.len(),
                &mut rng,
            );
            if spawn_chunks.is_empty() {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no target chunks for {:?} in macrochunk {} in {:?}", pack_target, macro_chunk_pos, dim_ref);
                continue;
            }

            spawn_locals.pack_centers.push(SampledNaturalPackCenter {
                pack_target,
                relation_target,
                center_chunk: pack_center_chunk,
            });
            spawned_packs += 1;
            let pack_entity = spawns_pack_entity.then(|| {
                let pack_entity = cmd.spawn((Pack, )).id();
                if is_pack {
                    cmd.entity(pack_entity).insert(TemplEntiRef(pack_target));
                }
                pack_entity
            });
            for (spawn_chunk, &spawn_target) in spawn_chunks.into_iter().zip(spawn_locals.sampled_beings.iter()) {
                let Ok((is_bit, is_race, _, _)) = spawn_queries.spawn_target_query.get(spawn_target) else {
                    warn!(target: WILDLIFE_SYSTEM, "Natural spawn member target {:?} for macrochunk {} in {:?} is neither bit nor race", spawn_target, macro_chunk_pos, dim_ref);
                    continue;
                };
                let gpos = spawn_chunk.random_gpos_within(&mut rng);
                let entity = cmd
                    .spawn((
                        BeingBundle::new(dim_ref, gpos),
                        Disabled,
                        NaturalSpawnOrigin(spawn_chunk),
                        PendingNaturalSpawnUnfreeze,
                    ))
                    .id();
                if let Some(pack_entity) = pack_entity {
                    cmd.entity(entity).insert(SquadMemberOf::single(pack_entity));
                }
                spawn_res.pending_wildlife_by_chunk.insert(entity, dim_ref, spawn_chunk);
                if is_bit {
                    cmd.entity(entity).insert(BitRef(spawn_target));
                } else if is_race {
                    cmd.entity(entity).insert(RaceRef(spawn_target));
                }
                spawned_beings += 1;
            }
            if let Some(pack_entity) = pack_entity {
                debug!(target: WILDLIFE_SYSTEM, "Natural spawn created pack {:?} for target {:?} in macrochunk {} in {:?} with {} members", pack_entity, pack_target, macro_chunk_pos, dim_ref, spawn_locals.sampled_beings.len());
            }
        }

        if spawned_beings > 0 {
            spawn_res.seeded_macrochunks.0.insert((dim_ref, macro_chunk_pos));
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn seeded macrochunk {} in {:?} with biome {:?}, packs {}, beings {}", macro_chunk_pos, dim_ref, biome_ent, spawned_packs, spawned_beings);
            file_log(
                "wildlife",
                "host",
                &format!(
                    "seed macrochunk={macro_chunk_pos} dim={dim_ref:?} biome={biome_ent:?} packs={spawned_packs} beings={spawned_beings} result=seeded"
                ),
            );
        } else {
            file_log(
                "wildlife",
                "host",
                &format!(
                    "seed macrochunk={macro_chunk_pos} dim={dim_ref:?} biome={biome_ent:?} pack_count={pack_count} result=no_beings"
                ),
            );
        }
    }
}

#[allow(unused_parens, )]
pub fn unfreeze_natural_wildlife_for_first_time_loaded_chunks(
    mut cmd: Commands,
    mut built_chunks: MessageReader<ChunkTerrainBuilt>,
    built_chunk_query: Query<(&DimensionRef, &ChunkPos), With<Chunk>>,
    mut blocking_tiles: BlockingTileParamSet,
    mut pending_wildlife_by_chunk: ResMut<NaturalSpawnReservationIndex>,
    mut being_query: Query<
        (
            Entity,
            &DimensionRef,
            Has<PendingNaturalSpawnUnfreeze>,
            &mut Transform,
            &mut GridLockedMovement,
        ),
        With<Disabled>,
    >,
    spawn_tile_tags_query: Query<(
        Option<&WhitelistedSpawnTileTags>,
        Option<&BlacklistedSpawnTileTags>,
    )>,
    bit_race_query: Query<&RaceRef>,
    being_str_id_query: Query<&StrId>,
    mut processed_beings: Local<EntityHashSet>,
    mut whitelisted_spawn_tile_tags: Local<WhitelistedSpawnTileTags>,
    mut blacklisted_spawn_tile_tags: Local<BlacklistedSpawnTileTags>,
) {
    let pending_wildlife_by_chunk_ptr = &mut *pending_wildlife_by_chunk as *mut NaturalSpawnReservationIndex;
    processed_beings.clear();
    let mut activated_beings = 0usize;
    let mut touched_chunks = 0usize;
    for built_chunk in built_chunks.read() {
        let Ok((&dim_ref, &chunk_pos)) = built_chunk_query.get(built_chunk.chunk_ent) else {
            error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze got ChunkTerrainBuilt for missing chunk entity {:?}", built_chunk.chunk_ent);
            continue;
        };
        let key = (dim_ref, chunk_pos);
        let Some(being_ents) = pending_wildlife_by_chunk.by_chunk.get_mut(&key) else {
            trace!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze got built chunk {:?} in {:?} but no pending wildlife reservation existed", chunk_pos, dim_ref);
            continue;
        };
        touched_chunks += 1;
        let pending_wildlife_by_chunk_ptr = pending_wildlife_by_chunk_ptr;
        being_ents.retain(|being_ent| {
            if !processed_beings.insert(*being_ent) {
                return true;
            }
            let being_str_id = being_str_id_query.get(*being_ent).ok();
            let Ok((being_ent, &dim_ref, is_pending_natural_spawn, mut transform, mut movement)) = being_query.get_mut(*being_ent) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze could not fetch reserved being {:?} in {:?} for chunk {:?}; likely despawned before activation", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, key.1);
                unsafe {
                    (*pending_wildlife_by_chunk_ptr).remove_being_except_chunk(*being_ent, key);
                }
                return false;
            };
            if !is_pending_natural_spawn {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze found reserved being {:?} in {:?} for chunk {:?} without PendingNaturalSpawnUnfreeze", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, key.1);
                unsafe {
                    (*pending_wildlife_by_chunk_ptr).remove_being_except_chunk(being_ent, key);
                }
                return false;
            }
            let bit_ref = blocking_tiles.get_being_bit_ref(being_ent);
            let race_ref = blocking_tiles.get_being_race_ref(being_ent);
            Being::collect_spawn_tile_tag_filters(
                bit_ref.map(|bit_ref| bit_ref.0),
                race_ref.map(|race_ref| race_ref.0),
                &spawn_tile_tags_query,
                |bit_ent| bit_race_query.get(bit_ent).ok().map(|race_ref| race_ref.0),
                &mut whitelisted_spawn_tile_tags,
                &mut blacklisted_spawn_tile_tags,
            );
            let Ok(&gpos) = blocking_tiles.gpos_query.get(being_ent) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze could not read gpos for reserved being {:?} in {:?} for chunk {:?}", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, key.1);
                unsafe {
                    (*pending_wildlife_by_chunk_ptr).remove_being_except_chunk(being_ent, key);
                }
                return false;
            };
            let home_chunk = gpos.to_chunkpos();
            let Some(found_gpos) = blocking_tiles.find_closest_allowe_gpos(
                dim_ref,
                gpos,
                being_ent,
                &whitelisted_spawn_tile_tags.0,
                &blacklisted_spawn_tile_tags.0,
            ) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze found no valid tile for reserved being {:?} in {:?} around chunk {:?}; it will stay frozen for now", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, home_chunk);
                return true;
            };
            let Ok(mut gpos) = blocking_tiles.gpos_query.get_mut(being_ent) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze could not reacquire mutable gpos for reserved being {:?} in {:?} for chunk {:?}", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, key.1);
                return true;
            };
            *gpos = found_gpos;
            movement.clear_step(found_gpos);
            transform.translation = found_gpos.to_translation(transform.translation.z);
            cmd.entity(being_ent)
                .try_remove::<(Disabled, PendingNaturalSpawnUnfreeze)>();
            unsafe {
                (*pending_wildlife_by_chunk_ptr).remove_being_except_chunk(being_ent, key);
            }
            activated_beings += 1;
            false
        });
        if being_ents.is_empty() {
            pending_wildlife_by_chunk.by_chunk.remove(&key);
        }
    }

    if activated_beings > 0 {
        debug!(target: WILDLIFE_SYSTEM, "Natural spawn activated {} reserved wildlife beings", activated_beings);
    }
    file_log(
        "wildlife",
        "host",
        &format!("activate touched_chunks={touched_chunks} activated_beings={activated_beings}"),
    );
}

pub fn watched_chunk_keys(
    dim_ref: DimensionRef,
    home_chunk: ChunkPos,
) -> impl Iterator<Item = (DimensionRef, ChunkPos)> {
    WATCHED_CHUNK_OFFSETS
        .into_iter()
        .map(move |offset| (dim_ref, ChunkPos(home_chunk.0 + offset)))
}
