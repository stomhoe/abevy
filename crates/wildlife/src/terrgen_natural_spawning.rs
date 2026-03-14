use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use being::{
    being_bundles::BeingBundle,
    being_inst_template::being_inst_template_resources::BitRef,
    pack::pack_components::PackInitialSpawnNormalDist,
    race::{race_components::Race, race_resources::RaceRef},
};
use being_shared::{BeingInstTemplate, BiomeHidPackSamplers};
use common::log_targets::WILDLIFE_SYSTEM;
use game_common::{game_common_samplers::EntityWeightedSampler, game_common_timers::TimerComp};
use tilemap_shared::{
    BiomeTagDistributionAtTimeout,
    DiscoveredMacroChunks,
    DimensionRef,
    MacroChunkBiomeTagDistributionMap,
    MacroChunkPos,
};

pub fn spawn_natural_wildlife_for_chunk(
    mut cmd: Commands,
    discovered_areas: Res<DiscoveredMacroChunks>,
    biome_wildlife_samplers: Res<BiomeHidPackSamplers>,
    mut pending_macro_chunks: Local<HashMap<(DimensionRef, MacroChunkPos), TimerComp>>,
    mut ready_macro_chunks: Local<Vec<(DimensionRef, MacroChunkPos)>>,
    mut macro_chunk_biome_tag_dist: ResMut<MacroChunkBiomeTagDistributionMap>,
    spawn_target_query: Query<(Has<BeingInstTemplate>, Has<Race>)>,
    spawn_pack_size_query: Query<&PackInitialSpawnNormalDist>,
    time: Res<Time>,
) {
    if !discovered_areas.is_changed() && pending_macro_chunks.is_empty() {
        return;
    }
    for &key in discovered_areas.0.iter() {
        pending_macro_chunks.entry(key).or_insert_with(|| TimerComp::new(4.0));
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

        let macro_sampler = build_weighted_spawn_target_sampler(distribution, &biome_wildlife_samplers);
        let Some(spawn_target) = macro_sampler.sample_with_rng(&mut rng) else {
            macro_chunk_biome_tag_dist.0.remove(&key);
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn found no candidate wildlife for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };

        let Ok((is_bit, is_race)) = spawn_target_query.get(spawn_target) else {
            macro_chunk_biome_tag_dist.0.remove(&key);
            warn!(target: WILDLIFE_SYSTEM, "Natural spawn target {:?} for macrochunk {} in {:?} is neither bit nor race", spawn_target, macro_chunk_pos, dim_ref);
            continue;
        };

        let pack_size = spawn_pack_size_query
            .get(spawn_target)
            .ok()
            .map(|dist| dist.sample_count(&mut rng))
            .unwrap_or(1);

        let spawn_positions = macro_chunk_pos.random_unique_gposes(pack_size, &mut rng);
        if spawn_positions.is_empty() {
            macro_chunk_biome_tag_dist.0.remove(&key);
            continue;
        }

        for gpos in spawn_positions {
            let entity = cmd.spawn(BeingBundle::new(dim_ref, gpos)).id();
            if is_bit {
                cmd.entity(entity).insert(BitRef(spawn_target));
            } else if is_race {
                cmd.entity(entity).insert(RaceRef(spawn_target));
            }
        }

        macro_chunk_biome_tag_dist.0.remove(&key);
        trace!(target: WILDLIFE_SYSTEM, "Natural spawn seeded macrochunk {} in {:?} with target {:?} and pack size {}", macro_chunk_pos, dim_ref, spawn_target, pack_size);
    }
}

fn build_weighted_spawn_target_sampler(
    distribution: &BiomeTagDistributionAtTimeout,
    biome_wildlife_samplers: &BiomeHidPackSamplers,
) -> EntityWeightedSampler {
    let mut weights_by_entity: EntityHashMap<f32> = EntityHashMap::default();
    for (tag, biome_weight) in distribution.sums.iter() {
        if *biome_weight <= 0.0 {
            continue;
        }
        let Some(sampler) = biome_wildlife_samplers.0.get(tag) else {
            continue;
        };
        for &(entity, sampler_weight) in sampler.iter() {
            if sampler_weight <= 0.0 {
                continue;
            }
            let total = weights_by_entity.entry(entity).or_insert(0.0);
            *total += *biome_weight * sampler_weight;
        }
    }
    let weights = weights_by_entity.into_iter().collect::<Vec<_>>();
    EntityWeightedSampler::new(&weights)
}
