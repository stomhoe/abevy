use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::EntityZero;
use modifier::{modifier_components::*, modifier_types::*};

use crate::body::{
    body_part::body_part_components::*, body_part::body_part_resources::*, body_tree_resources::*,
};

fn stat_from_map(map: &bevy::platform::collections::HashMap<String, f32>, key: HashId) -> f32 {
    for (k, &v) in map {
        if HashId::from(k) == key {
            return v.max(0.0);
        }
    }
    0.0
}

fn stats_to_hashid_map(map: &bevy::platform::collections::HashMap<String, f32>) -> HashIdMap<f32> {
    let mut out = HashIdMap::default();
    for (k, &v) in map {
        out.overwrite(HashId::from(k), v.max(0.0));
    }
    out
}

#[allow(unused_parens)]
pub fn init_body_parts(
    mut cmd: Commands,
    part_map: Res<BodyPartEntityMap>,
) {
    if !part_map.0.is_empty() {
        return;
    }

    let mut spawned_ids: HashSet<StrId> = HashSet::default();

    for part in load_body_part_seri_defs() {
        let part_id = match StrId::new_with_result(&part.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for BodyPart: {}", e));
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
            .insert((part_id.clone(), BodyPart, EntityZero));

        if !part.name.trim().is_empty() {
            cmd.entity(part_ent).insert(DisplayName::trunc(part.name.clone()));
        } else {
            cmd.entity(part_ent)
                .insert(DisplayName::trunc(part_id.as_str()));
        }

        if !part.slots.is_empty() {
            cmd.entity(part_ent)
                .insert(BodyPartSlots::new(part.slots.clone()));
        }

        if !part.tags.is_empty() {
            cmd.entity(part_ent).insert(TagSet::new(&part.tags));
        }

        if part.coverage_weight > 0 {
            let weight = part.coverage_weight;
            cmd.entity(part_ent).insert(BodyPartCoverageWeight(weight));
        }
        let mut forced_stats = stats_to_hashid_map(&part.forced_stats);
        if !forced_stats.contains_key(STAT_PAIN_SENSITIVITY) {
            forced_stats.overwrite(STAT_PAIN_SENSITIVITY, 1.0);
        }
        let weighted_stats = stats_to_hashid_map(&part.weighted_stats);
        cmd.entity(part_ent).insert((
            BodyPartForcedDistribution(forced_stats),
            BodyPartWeightedDistribution(weighted_stats),
        ));
        let hp_capacity = stat_from_map(&part.forced_stats, STAT_HP_CAPACITY);
        if hp_capacity > 0.0 {
            let max_hp = hp_capacity;
            cmd.spawn((
                ModifierTarget(part_ent),
                BaseValue(max_hp),
                CurrEffectiveValue(max_hp),
                ApplyMode::Add,
                HitpointsCapacity,
                ChildOf(part_ent),
                EntityZero,
            ));
            cmd.entity(part_ent).try_insert(BodyPartDamage(0.0));
        }

        let hp_regen_rate = stat_from_map(&part.forced_stats, STAT_HP_REGEN_RATE);
        if hp_regen_rate > 0.0 {
            cmd.spawn((
                ModifierTarget(part_ent),
                BaseValue(hp_regen_rate),
                CurrEffectiveValue(hp_regen_rate),
                ApplyMode::Add,
                HitpointRegenRate,
                ChildOf(part_ent),
                EntityZero,
            ));
        }

        if part.bleed_rate > 0.0 {
            let bleed_rate = part.bleed_rate;
            cmd.spawn((
                ModifierTarget(part_ent),
                BaseValue(bleed_rate),
                CurrEffectiveValue(bleed_rate),
                ApplyMode::Add,
                BleedRate,
                ChildOf(part_ent),
                EntityZero,
            ));
        }

        let blood_capacity = stat_from_map(&part.forced_stats, STAT_BLOOD_CAPACITY);
        if blood_capacity > 0.0 {
            cmd.spawn((
                ModifierTarget(part_ent),
                BaseValue(blood_capacity),
                CurrEffectiveValue(blood_capacity),
                ApplyMode::Add,
                BloodCapacity,
                ChildOf(part_ent),
                EntityZero,
            ));
        }

        let pain_sensitivity = stat_from_map(&part.forced_stats, STAT_PAIN_SENSITIVITY);
        if pain_sensitivity > 0.0 {
            cmd.spawn((
                ModifierTarget(part_ent),
                BaseValue(pain_sensitivity),
                CurrEffectiveValue(pain_sensitivity),
                ApplyMode::Add,
                PainSensitivity,
                ChildOf(part_ent),
                EntityZero,
            ));
        }

        let manipulation = stat_from_map(&part.forced_stats, STAT_MANIPULATION);
        if manipulation > 0.0 {
            let modifier_ent = cmd
                .spawn((
                    ModifierTarget(part_ent),
                    BaseValue(manipulation),
                    CurrEffectiveValue(manipulation),
                    ApplyMode::Add,
                    Manipulation,
                    ChildOf(part_ent),
                    EntityZero,
                ))
                .id();
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }

        let walk_speed = stat_from_map(&part.forced_stats, STAT_WALK_SPEED);
        if walk_speed > 0.0 {
            let modifier_ent = cmd
                .spawn((
                    ModifierTarget(part_ent),
                    BaseValue(walk_speed),
                    CurrEffectiveValue(walk_speed),
                    ApplyMode::Add,
                    WalkSpeed,
                    ChildOf(part_ent),
                    EntityZero,
                ))
                .id();
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }

        let swim_speed = stat_from_map(&part.forced_stats, STAT_SWIM_SPEED);
        if swim_speed > 0.0 {
            let modifier_ent = cmd
                .spawn((
                    ModifierTarget(part_ent),
                    BaseValue(swim_speed),
                    CurrEffectiveValue(swim_speed),
                    ApplyMode::Add,
                    SwimSpeed,
                    ChildOf(part_ent),
                    EntityZero,
                ))
                .id();
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }

        let fly_speed = stat_from_map(&part.forced_stats, STAT_FLY_SPEED);
        if fly_speed > 0.0 {
            let modifier_ent = cmd
                .spawn((
                    ModifierTarget(part_ent),
                    BaseValue(fly_speed),
                    CurrEffectiveValue(fly_speed),
                    ApplyMode::Add,
                    FlySpeed,
                    ChildOf(part_ent),
                    EntityZero,
                ))
                .id();
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }

        let vision = stat_from_map(&part.forced_stats, STAT_VISION);
        if vision > 0.0 {
            let modifier_ent = cmd
                .spawn((
                    ModifierTarget(part_ent),
                    BaseValue(vision),
                    CurrEffectiveValue(vision),
                    ApplyMode::Add,
                    Vision,
                    ChildOf(part_ent),
                    EntityZero,
                ))
                .id();
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }

        if !part.depth.trim().is_empty() {
            cmd.entity(part_ent)
                .insert(BodyPartDepth::from(part.depth.clone()));
        }

        if part.vital {
            cmd.entity(part_ent).insert(BodyPartVital);
        }
    }
}

fn apply_synergy_to_modifier(cmd: &mut Commands, modifier_ent: Entity, part: &BodyPartSeri) {
    if part.synergy_tags.is_empty() {
        return;
    }

    let mut tags = ModifierTags::default();
    for tag_str in &part.synergy_tags {
        let tag_str = tag_str.trim();
        if tag_str.is_empty() {
            continue;
        }
        tags.insert(Tag::from(tag_str));
    }
    if tags.is_empty() {
        return;
    }
    cmd.entity(modifier_ent).insert(tags);

    if part.synergy_offset != 0.0 {
        let mut offsets = HashMap::default();
        for tag_str in &part.synergy_tags {
            let tag_str = tag_str.trim();
            if tag_str.is_empty() {
                continue;
            }
            offsets.insert(Tag::from(tag_str), part.synergy_offset);
        }
        if !offsets.is_empty() {
            cmd.entity(modifier_ent).insert(OffsetValForSelf(offsets));
        }
    }

    if part.synergy_copy_mult != 0.0 {
        let mut mults = HashMap::default();
        for tag_str in &part.synergy_tags {
            let tag_str = tag_str.trim();
            if tag_str.is_empty() {
                continue;
            }
            mults.insert(Tag::from(tag_str), part.synergy_copy_mult);
        }
        if !mults.is_empty() {
            cmd.entity(modifier_ent)
                .insert(CopyMultOfOthersIntoSelf(mults));
        }
    }
}
