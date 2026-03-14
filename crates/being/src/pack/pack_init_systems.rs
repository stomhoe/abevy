use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, platform::collections::HashMap, prelude::*};
use common::common_components::StrId;
use game_common::{
    game_common_components::EntityZero,
    game_common_samplers::CappedNormalDist,
};
use tilemap::terrain::biome::{biome_components::BiomePackSampler, biome_resources::BiomeEntityMap};

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
    mut biome_pack_samplers: Query<&mut BiomePackSampler>,
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
    let mut pack_by_id: HashMap<StrId, Entity> = HashMap::default();
    let mut race_members_by_pack: EntityHashMap<EntityHashSet> = EntityHashMap::default();
    let mut bit_members_by_pack: EntityHashMap<EntityHashSet> = EntityHashMap::default();

    for pack_seri in load_pack_seri_defs() {
        let str_id = StrId::trunc(&pack_seri.id);
        let pack_entity = cmd.spawn((Pack, EntityZero, str_id.clone())).id();
        pack_by_id.insert(str_id, pack_entity);

        let race_members = race_members_by_pack
            .entry(pack_entity)
            .or_insert_with(EntityHashSet::default);
        for race_id in &pack_seri.race_ids {
            let trimmed = race_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(race_ent) = race_emap.0.get_cloned(trimmed) else {
                continue;
            };
            race_members.insert(race_ent);
        }

        let bit_members = bit_members_by_pack
            .entry(pack_entity)
            .or_insert_with(EntityHashSet::default);
        for bit_id in &pack_seri.bit_ids {
            let trimmed = bit_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(bit_ent) = bit_emap.0.get_cloned(trimmed) else {
                continue;
            };
            bit_members.insert(bit_ent);
        }

        if !pack_seri.behavior.trim().is_empty() {
            cmd.entity(pack_entity)
                .insert(PackBehavior(pack_seri.behavior.clone()));
        }

        if !pack_seri.initial_spawn_normal_dist.is_disabled() {
            cmd.entity(pack_entity)
                .insert(PackInitialSpawnNormalDist(CappedNormalDist::from_seri(
                    pack_seri.initial_spawn_normal_dist.clone(),
                )));
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
            race_members_by_pack
                .entry(pack_ent)
                .or_insert_with(EntityHashSet::default)
                .insert(race_ent);
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
            bit_members_by_pack
                .entry(pack_ent)
                .or_insert_with(EntityHashSet::default)
                .insert(bit_ent);
        }
    }

    for (pack_ent, race_members) in race_members_by_pack {
        if race_members.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(PackRaceMembers(race_members.into_iter().collect()));
    }
    for (pack_ent, bit_members) in bit_members_by_pack {
        if bit_members.is_empty() {
            continue;
        }
        cmd.entity(pack_ent)
            .insert(PackBitMembers(bit_members.into_iter().collect()));
    }
}
