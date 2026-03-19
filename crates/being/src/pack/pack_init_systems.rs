use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};
use common::common_components::StrId;
use game_common::{
    game_common_components::EntityZero,
    game_common_samplers::CappedNormalDist,
};
use tilemap::terrain::biome::{biome_components::SpawnablesPerBiome, biome_resources::BiomeEntityMap};

use crate::{
    being_inst_template::being_inst_template_resources::{BeingInstTemplateEntityMap, load_bit_seri_defs},
    pack::{pack_components::*, pack_resources::*},
    race::race_resources::{RaceEntityMap, load_race_seri_defs},
};

pub fn init_packs(
    mut cmd: Commands,
    pack_emap: Res<PackEntityMap>,
    race_emap: Option<Res<RaceEntityMap>>,
    bit_emap: Option<Res<BeingInstTemplateEntityMap>>,
    biome_emap: Res<BiomeEntityMap>,
    mut biome_pack_samplers: Query<&mut SpawnablesPerBiome>,
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
    let mut being_samplers_by_pack: EntityHashMap<PackBeingSampler> = EntityHashMap::default();
    let mut rank_sampler_by_pack: EntityHashMap<PackMemberRankSampler> = EntityHashMap::default();
    let mut min_dists_by_pack: EntityHashMap<PackMinDistsToPacksOrRaces> = EntityHashMap::default();

    for pack_seri in &pack_seris {
        let str_id = StrId::trunc(&pack_seri.id);
        let pack_entity = cmd.spawn((Pack, EntityZero, str_id.clone())).id();
        pack_by_id.insert(str_id, pack_entity);
    }

    for pack_seri in &pack_seris {
        let str_id = StrId::trunc(&pack_seri.id);
        let Some(&pack_entity) = pack_by_id.get(&str_id) else {
            continue;
        };

        let being_sampler = being_samplers_by_pack
            .entry(pack_entity)
            .or_default();
        let leader_priority = rank_sampler_by_pack
            .entry(pack_entity)
            .or_default();
        let min_dists = min_dists_by_pack
            .entry(pack_entity)
            .or_default();
        for (race_id, config) in &pack_seri.race_ids {
            let trimmed = race_id.as_str().trim();
            let (weight, priority) = config;
            if trimmed.is_empty() || *weight <= 0.0 {
                continue;
            }
            let Ok(race_ent) = race_emap.0.get_cloned(trimmed) else {
                continue;
            };
            being_sampler.0.insert(race_ent, *weight);
            leader_priority.0.insert(race_ent, CappedNormalDist::from_seri(priority.clone()));
        }

        for (bit_id, config) in &pack_seri.bit_ids {
            let trimmed = bit_id.as_str().trim();
            let (weight, priority) = config;
            if trimmed.is_empty() || *weight <= 0.0 {
                continue;
            }
            let Ok(bit_ent) = bit_emap.0.get_cloned(trimmed) else {
                continue;
            };
            being_sampler.0.insert(bit_ent, *weight);
            leader_priority.0.insert(bit_ent, CappedNormalDist::from_seri(priority.clone()));
        }

        if !pack_seri.behavior_on_member_attack.trim().is_empty() {
            cmd.entity(pack_entity)
                .insert(PackBehavior(StrId::trunc(&pack_seri.behavior_on_member_attack)));
        }

        if !pack_seri.spawn_being_count_normal_dist.is_sentinel() {
            cmd.entity(pack_entity)
                .insert(PackInitialSize(CappedNormalDist::from_seri(
                    pack_seri.spawn_being_count_normal_dist.clone(),
                )));
        }

        for (target_id, min_inbetween_chunks) in &pack_seri.chunk_separation_to_others {
            let trimmed = target_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(&other_pack_ent) = pack_by_id.get(&StrId::trunc(trimmed)) {
                min_dists.insert(other_pack_ent, *min_inbetween_chunks);
                continue;
            }
            let Ok(race_ent) = race_emap.0.get_cloned(trimmed) else {
                continue;
            };
            min_dists.insert(race_ent, *min_inbetween_chunks);
        }

        for (biome_id, weight) in &pack_seri.biome_affinity {
            if *weight <= 0.0 {
                continue;
            }
            let Ok(biome_ent) = biome_emap.0.get_cloned(biome_id) else {
                continue;
            };
            let Ok(mut biome_pack_sampler) = biome_pack_samplers.get_mut(biome_ent) else {
                continue;
            };
            biome_pack_sampler.0.insert(pack_entity, *weight);
        }
    }

    for race_seri in load_race_seri_defs() {
        let race_id = StrId::trunc(&race_seri.id);
        let Ok(race_ent) = race_emap.0.get_cloned(&race_id) else {
            continue;
        };
        for pack_id in &race_seri.belongs_to_packs {
            let trimmed = pack_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some(&pack_ent) = pack_by_id.get(&StrId::trunc(trimmed)) else {
                continue;
            };
            being_samplers_by_pack
                .entry(pack_ent)
                .or_default()
                .0.insert(race_ent, 1.0);
            rank_sampler_by_pack
                .entry(pack_ent)
                .or_default()
                .0.insert(race_ent, CappedNormalDist::default());
        }
    }

    for bit_seri in load_bit_seri_defs() {
        let bit_id = StrId::trunc(&bit_seri.id);
        let Ok(bit_ent) = bit_emap.0.get_cloned(&bit_id) else {
            continue;
        };
        for pack_id in &bit_seri.belongs_to_packs {
            let trimmed = pack_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some(&pack_ent) = pack_by_id.get(&StrId::trunc(trimmed)) else {
                continue;
            };
            being_samplers_by_pack
                .entry(pack_ent)
                .or_default()
                .0.insert(bit_ent, 1.0);
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
    for (pack_ent, min_dists) in min_dists_by_pack {
        if min_dists.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(min_dists);
    }
}
