use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::Templ;
use item_shared::item_components::SlottedItemHolder;
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_seris::ModifierSynergySeri;
use ::being_shared::*;

use crate::body::{
    bodypart::bodypart_resources::*,
};

const STAT_BLEED_RATE: HashId = HashId::hash("bleed_rate");

fn stats_to_hashid_map(map: &bevy::platform::collections::HashMap<String, f32>) -> HashIdMap<f32> {
    let mut out = HashIdMap::default();
    for (k, &v) in map {
        out.overwrite(HashId::from(k), v.max(0.0));
    }
    out
}

fn stat_from_hashid_map(map: &HashIdMap<f32>, key: HashId) -> f32 {
    map.get_opt(key).copied().unwrap_or(0.0).max(0.0)
}

fn synergies_from_part(part: &BodypartSeri) -> ModifierSynergies {
    let mut synergies = ModifierSynergies::default();
    for (tag_str, synergy) in &part.synergies {
        let tag_str = tag_str.trim();
        if tag_str.is_empty() {
            continue;
        }
        let tag = Tag::from(tag_str);
        match synergy {
            ModifierSynergySeri::Offset(value) => {
                if *value != 0.0 {
                    synergies.0.insert(tag, ModifierSynergy::Offset(*value));
                }
            }
            ModifierSynergySeri::CopyFrac(value) => {
                if *value != 0.0 {
                    synergies.0.insert(tag, ModifierSynergy::CopyFrac(*value));
                }
            }
        }
    }
    synergies
}

#[allow(unused_parens)]
pub fn init_bodyparts(
    mut cmd: Commands,
    part_map: Res<BodypartEntityMap>,
) {
    if !part_map.0.is_empty() {
        return;
    }

    let mut spawned_ids: HashSet<StrId> = HashSet::default();

    for part in load_bodypart_seri_defs() {
        let part_id = match StrId::new_with_result(&part.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for Bodypart: {}", e));
                error!(target: "body_init", "{}", err);
                continue;
            }
        };

        if spawned_ids.contains(&part_id) {
            continue;
        }
        if part_map.0.get_cloned(&part_id).is_ok() {
            spawned_ids.insert(part_id);
            continue;
        }

        let part_ent = cmd.spawn_empty().id();
        spawned_ids.insert(part_id.clone());

        cmd.entity(part_ent)
            .insert((part_id.clone(), Bodypart, Templ));

        if !part.name.trim().is_empty() {
            cmd.entity(part_ent).insert(DisplayName::trunc(part.name.clone()));
        } else {
            cmd.entity(part_ent)
                .insert(DisplayName::trunc(part_id.as_str()));
        }

        if !part.slots.slots.is_empty() {
            cmd.entity(part_ent)
                .insert(SlottedItemHolder::new(&part.slots));
        }

        if !part.tags.is_empty() {
            cmd.entity(part_ent).insert(TagSet::new(&part.tags));
        }

        if part.coverage_weight > 0 {
            let weight = part.coverage_weight;
            cmd.entity(part_ent).insert(BodypartCoverageWeight(weight));
        }
        let mut forced_stats = stats_to_hashid_map(&part.forced_stats);
        if !forced_stats.contains_key(BodypartStat::STAT_PAIN_SENSITIVITY) {
            forced_stats.overwrite(BodypartStat::STAT_PAIN_SENSITIVITY, 1.0);
        }
        let mut weighted_stats = stats_to_hashid_map(&part.weighted_stats);
        if part.bleed_rate > 0.0 {
            weighted_stats.overwrite(STAT_BLEED_RATE, part.bleed_rate);
        }
        if !weighted_stats.contains_key(BodypartStat::STAT_PAIN_SENSITIVITY) {
            weighted_stats.overwrite(BodypartStat::STAT_PAIN_SENSITIVITY, 1.0);
        }
        let hp_capacity = stat_from_hashid_map(&weighted_stats, BodypartStat::STAT_HP_CAPACITY);
        cmd.entity(part_ent).insert((
            BodypartForcedStats(forced_stats),
            BodypartWeightedDistribution(weighted_stats),
        ));
        let synergies = synergies_from_part(&part);
        if !synergies.0.is_empty() {
            cmd.entity(part_ent).insert(synergies);
        }
        if hp_capacity > 0.0 {
            cmd.entity(part_ent).try_insert(BodypartDamage(0.0));
        }

        if !part.depth.trim().is_empty() {
            cmd.entity(part_ent)
                .insert(BodypartDepth::from(part.depth.clone()));
        }

        if part.vital {
            cmd.entity(part_ent).insert(Vital);
        }
    }
}
