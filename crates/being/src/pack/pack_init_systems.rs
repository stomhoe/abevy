use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};
use ::being_shared::*;
use common::common_components::StrId;
use common::common_id_components::HashId;
use game_common::{
    game_common_components::Templ,
};
use tilemap::terrain::biome::{biome_components::CreatureSampler, biome_resources::BiomeEntityMap};
use tilemap_shared::CappedNormalDist;

use crate::pack::pack_seris::load_pack_seri_defs;


pub fn init_packs(
    mut cmd: Commands,
    pack_emap: Res<PackEntityMap>,
    race_emap: Option<Res<RaceEntityMap>>,
    bit_emap: Option<Res<BeingInstTemplateEntityMap>>,
    biome_emap: Res<BiomeEntityMap>,
    mut biome_pack_samplers: Query<&mut CreatureSampler>,
) {
    if !pack_emap.0.is_empty() {
        return;
    }
    let Some(race_emap) = race_emap else {
        return;
    };
    let Some(bit_emap) = bit_emap else {
        return;
    };
    let pack_seris = load_pack_seri_defs();
    let mut pack_by_id: HashMap<StrId, Entity> = HashMap::default();
    let mut being_samplers_by_pack: EntityHashMap<PackRaceOrBitSampler> = EntityHashMap::default();
    let mut rank_sampler_by_pack: EntityHashMap<PackMemberRankSampler> = EntityHashMap::default();
    let mut spawn_bounds_by_pack: EntityHashMap<PackRaceOrBitSpawnQuotas> = EntityHashMap::default();
    let mut center_rank_multipliers_by_pack: EntityHashMap<CenterWeightRankBasedMultiplier> = EntityHashMap::default();
    let mut min_dists_by_pack: EntityHashMap<PackMinSepToPacksOrRaces> = EntityHashMap::default();

    for pack_seri in &pack_seris {
        let str_id = match StrId::new_with_result(pack_seri.id.trim(), 0) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: "pack_init",
                    "Skipping pack with invalid id '{}': {}",
                    pack_seri.id,
                    err,
                );
                continue;
            }
        };
        let pack_entity = cmd.spawn((Pack, Templ, str_id.clone(), )).id();
        pack_by_id.insert(str_id, pack_entity);
    }

    for pack_seri in &pack_seris {
        let str_id = match StrId::new_with_result(pack_seri.id.trim(), 0) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: "pack_init",
                    "Skipping pack init pass for invalid id '{}': {}",
                    pack_seri.id,
                    err,
                );
                continue;
            }
        };
        let Some(&pack_entity) = pack_by_id.get(&str_id) else {
            continue;
        };
        if !pack_seri.spawn_pack_entity {
            cmd.entity(pack_entity).insert(NoSpawnSquadEntity);
        }
        cmd.entity(pack_entity).insert(PackSpawnRadius(pack_seri.pack_spawn_radius));
        cmd.entity(pack_entity).insert(pack_seri.tags_with_id());
        if !pack_seri.wander.is_disabled() {
            cmd.entity(pack_entity)
                .insert(pack_seri.wander.clone().sanitized());
        }
        if pack_seri.avgpos_rank_based_weight_multiplier != 1.0 {
            cmd.entity(pack_entity).insert(GlobalCenterRankWeightMultiplier(
                pack_seri.avgpos_rank_based_weight_multiplier,
            ));
        }

        let being_sampler = being_samplers_by_pack
            .entry(pack_entity)
            .or_default();
        let leader_priority = rank_sampler_by_pack
            .entry(pack_entity)
            .or_default();
        let spawn_bounds = spawn_bounds_by_pack
            .entry(pack_entity)
            .or_default();
        let center_rank_multipliers = center_rank_multipliers_by_pack
            .entry(pack_entity)
            .or_default();
        let min_dists = min_dists_by_pack
            .entry(pack_entity)
            .or_default();
        for (member_id, config) in &pack_seri.ids {
            let trimmed = member_id.as_str().trim();
            let weight = config.weight;
            let priority = &config.rank_dist;
            let min_count = config.min;
            let max_count = config.max;
            if trimmed.is_empty() || weight <= 0.0 {
                continue;
            }
            let resolved_ent = if config.race_first {
                race_emap
                    .0
                    .get_cloned(trimmed)
                    .or_else(|_| bit_emap.0.get_cloned(trimmed))
            } else {
                bit_emap
                    .0
                    .get_cloned(trimmed)
                    .or_else(|_| race_emap.0.get_cloned(trimmed))
            };
            let Ok(resolved_ent) = resolved_ent else {
                continue;
            };
            let resolved_hash = HashId::from(trimmed);
            if let Err(negative_items) = being_sampler.0.insert(resolved_hash, weight) {
                tilemap_shared::log_negative_weighted_sampler_items!("being_pack_init", pack_seri.id.as_str(), vec![negative_items]);
            }
            leader_priority.0.insert(resolved_ent, CappedNormalDist::from_seri(priority.clone()));
            if min_count > 0 || max_count < u32::MAX {
                let max_count = if max_count == u32::MAX {
                    u32::MAX
                } else {
                    max_count.max(min_count)
                };
                let entry = spawn_bounds.0.entry(resolved_ent).or_insert((0, 0));
                entry.0 = entry.0.max(min_count);
                entry.1 = if entry.1 == u32::MAX || max_count == u32::MAX {
                    u32::MAX
                } else {
                    entry.1.max(max_count)
                };
            }
        }

        for (member_id, multiplier) in &pack_seri.avgpos_rank_based_weight_multipliers {
            let trimmed = member_id.trim();
            if trimmed.is_empty() || *multiplier <= 0.0 {
                continue;
            }
            let member_ent = race_emap
                .0
                .get_cloned(trimmed)
                .or_else(|_| bit_emap.0.get_cloned(trimmed));
            let Ok(member_ent) = member_ent else {
                continue;
            };
            center_rank_multipliers.0.insert(member_ent, *multiplier);
        }

        if !pack_seri.behavior_on_member_attack.trim().is_empty() {
            let behavior_str_id = match StrId::new_with_result(pack_seri.behavior_on_member_attack.trim(), 0) {
                Ok(behavior_str_id) => behavior_str_id,
                Err(err) => {
                    error!(
                        target: "pack_init",
                        "Pack '{}' has invalid behavior_on_member_attack id '{}': {}",
                        str_id,
                        pack_seri.behavior_on_member_attack,
                        err,
                    );
                    continue;
                }
            };
            cmd.entity(pack_entity)
                .insert(PackOnPreyedOnBehavior(behavior_str_id));
        }
        cmd.entity(pack_entity).insert((
            PackAttackAlertEffectivenessFalloff(pack_seri.attack_alert_effectiveness_falloff.max(0.0)),
            PackCounterRegroupTightness(pack_seri.counter_regroup_tightness.max(0.0)),
        ));

        if !pack_seri.spawn_being_count_normal_dist.is_sentinel() {
            cmd.entity(pack_entity)
                .insert(PackInitialSizeSampler(CappedNormalDist::from_seri(
                    pack_seri.spawn_being_count_normal_dist.clone(),
                )));
        }

        for (target_id, min_inbetween_chunks) in &pack_seri.chunk_separation_to_others {
            let trimmed = target_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let target_str_id = match StrId::new_with_result(trimmed, 0) {
                Ok(target_str_id) => target_str_id,
                Err(err) => {
                    error!(
                        target: "pack_init",
                        "Pack '{}' has invalid chunk_separation target id '{}': {}",
                        str_id,
                        target_id,
                        err,
                    );
                    continue;
                }
            };
            if let Some(&other_pack_ent) = pack_by_id.get(&target_str_id) {
                min_dists.insert(other_pack_ent, *min_inbetween_chunks);
                continue;
            }
            let Ok(race_ent) = race_emap.0.get_cloned(trimmed) else {
                continue;
            };
            min_dists.insert(race_ent, *min_inbetween_chunks);
        }

        for (biome_id, weight) in &pack_seri.biome_affinity {
            if *weight == 0.0 {
                continue;
            }
            let Ok(biome_ent) = biome_emap.0.get_cloned(biome_id) else {
                continue;
            };
            let Ok(mut biome_pack_sampler) = biome_pack_samplers.get_mut(biome_ent) else {
                continue;
            };
            biome_pack_sampler.add_affinity(HashId::from(pack_seri.id.as_str()), *weight);
        }
    }

    for race_seri in load_race_seri_defs() {
        let race_id = match StrId::new_with_result(race_seri.id.trim(), 0) {
            Ok(race_id) => race_id,
            Err(err) => {
                error!(
                    target: "pack_init",
                    "Skipping race membership mapping due to invalid race id '{}': {}",
                    race_seri.id,
                    err,
                );
                continue;
            }
        };
        let Ok(race_ent) = race_emap.0.get_cloned(&race_id) else {
            continue;
        };
        for pack_id in &race_seri.belongs_to_packs {
            let trimmed = pack_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let pack_str_id = match StrId::new_with_result(trimmed, 0) {
                Ok(pack_str_id) => pack_str_id,
                Err(err) => {
                    error!(
                        target: "pack_init",
                        "Race '{}' references invalid pack id '{}': {}",
                        race_id,
                        pack_id,
                        err,
                    );
                    continue;
                }
            };
            let Some(&pack_ent) = pack_by_id.get(&pack_str_id) else {
                continue;
            };
            if let Err(negative_item) = being_samplers_by_pack
                .entry(pack_ent)
                .or_default()
                .0
                .insert(HashId::from(race_id.as_str()), 1.0)
            {
                tilemap_shared::log_negative_weighted_sampler_items!("being_pack_init", race_id.as_str(), vec![negative_item]);
            }
            rank_sampler_by_pack
                .entry(pack_ent)
                .or_default()
                .0.insert(race_ent, CappedNormalDist::default());
        }
    }

    for bit_seri in load_bit_seri_defs() {
        let bit_id = match StrId::new_with_result(bit_seri.id.trim(), 0) {
            Ok(bit_id) => bit_id,
            Err(err) => {
                error!(
                    target: "pack_init",
                    "Skipping bit membership mapping due to invalid bit id '{}': {}",
                    bit_seri.id,
                    err,
                );
                continue;
            }
        };
        let Ok(bit_ent) = bit_emap.0.get_cloned(&bit_id) else {
            continue;
        };
        for pack_id in &bit_seri.belongs_to_packs {
            let trimmed = pack_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let pack_str_id = match StrId::new_with_result(trimmed, 0) {
                Ok(pack_str_id) => pack_str_id,
                Err(err) => {
                    error!(
                        target: "pack_init",
                        "Bit '{}' references invalid pack id '{}': {}",
                        bit_id,
                        pack_id,
                        err,
                    );
                    continue;
                }
            };
            let Some(&pack_ent) = pack_by_id.get(&pack_str_id) else {
                continue;
            };
            if let Err(negative_item) = being_samplers_by_pack
                .entry(pack_ent)
                .or_default()
                .0
                .insert(HashId::from(bit_id.as_str()), 1.0)
            {
                tilemap_shared::log_negative_weighted_sampler_items!("being_pack_init", bit_id.as_str(), vec![negative_item]);
            }
            rank_sampler_by_pack
                .entry(pack_ent)
                .or_default()
                .0.insert(bit_ent, CappedNormalDist::default());
        }
    }

    for (pack_ent, being_sampler) in being_samplers_by_pack {
        if being_sampler.0.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(being_sampler);
    }
    for (pack_ent, rank_sampler) in rank_sampler_by_pack {
        if rank_sampler.0.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(rank_sampler);
    }
    for (pack_ent, spawn_bounds) in spawn_bounds_by_pack {
        if spawn_bounds.0.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(spawn_bounds);
    }
    for (pack_ent, center_rank_multipliers) in center_rank_multipliers_by_pack {
        if center_rank_multipliers.0.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(center_rank_multipliers);
    }
    for (pack_ent, min_dists) in min_dists_by_pack {
        if min_dists.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(min_dists);
    }
}
