use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::EntityZero;
use item_shared::item_components::SlottedItemHolder;
use modifier_shared::{modifier_components::*, modifier_types::*};
use modifier_shared::modifier_seris::ModifierSynergySeri;

use crate::body::{
    body_part::body_part_components::*, body_part::body_part_resources::*,
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

        if !part.slots.slots.is_empty() {
            cmd.entity(part_ent)
                .insert(SlottedItemHolder::new(&part.slots));
        }

        if !part.tags.is_empty() {
            cmd.entity(part_ent).insert(TagSet::new(&part.tags));
        }

        if part.coverage_weight > 0 {
            let weight = part.coverage_weight;
            cmd.entity(part_ent).insert(BodyPartCoverageWeight(weight));
        }
        let mut forced_stats = stats_to_hashid_map(&part.forced_stats);
        if !forced_stats.contains_key(BodyPartStat::STAT_PAIN_SENSITIVITY) {
            forced_stats.overwrite(BodyPartStat::STAT_PAIN_SENSITIVITY, 1.0);
        }
        let weighted_stats = stats_to_hashid_map(&part.weighted_stats);
        cmd.entity(part_ent).insert((
            BodyPartForcedDistribution(forced_stats),
            BodyPartWeightedDistribution(weighted_stats),
        ));
        let hp_capacity = stat_from_map(&part.forced_stats, BodyPartStat::STAT_HP_CAPACITY);
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

        let hp_regen_rate = stat_from_map(&part.forced_stats, BodyPartStat::STAT_HP_REGEN_RATE);
        macro_rules! spawn_bodypart_modifier {
            ($value:expr, $component:expr) => {{
                let value = $value;
                if value > 0.0 {
                    Some(
                        cmd.spawn((
                            ModifierTarget(part_ent),
                            BaseValue(value),
                            CurrEffectiveValue(value),
                            ApplyMode::Add,
                            $component,
                            ChildOf(part_ent),
                            EntityZero,
                        ))
                        .id(),
                    )
                } else {
                    None
                }
            }};
        }
        let _ = spawn_bodypart_modifier!(hp_regen_rate, HitpointRegenRate);

        if part.bleed_rate > 0.0 {
            let bleed_rate = part.bleed_rate;
            let _ = spawn_bodypart_modifier!(bleed_rate, BleedRate);
        }

        let blood_capacity = stat_from_map(&part.forced_stats, BodyPartStat::STAT_BLOOD_CAPACITY);
        let _ = spawn_bodypart_modifier!(blood_capacity, BloodCapacity);

        let pain_sensitivity = stat_from_map(&part.forced_stats, BodyPartStat::STAT_PAIN_SENSITIVITY);
        let _ = spawn_bodypart_modifier!(pain_sensitivity, PainSensitivity);

        let manipulation = stat_from_map(&part.forced_stats, BodyPartStat::STAT_MANIPULATION_DEXTERITY);
        if let Some(modifier_ent) = spawn_bodypart_modifier!(manipulation, ManipulationDexterity) {
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }
        let manip_str = stat_from_map(&part.forced_stats, BodyPartStat::STAT_MANIPULATION_STRENGTH);
        if let Some(modifier_ent) = spawn_bodypart_modifier!(manip_str, ManipulationStrength) {
            apply_synergy_to_modifier(&mut cmd, modifier_ent, &part);
        }

        let walk_speed = stat_from_map(&part.forced_stats, BodyPartStat::STAT_WALK_SPEED);
        let _ = spawn_bodypart_modifier!(walk_speed, WalkSpeed);

        let swim_speed = stat_from_map(&part.forced_stats, BodyPartStat::STAT_SWIM_SPEED);
        let _ = spawn_bodypart_modifier!(swim_speed, SwimSpeed);

        let fly_speed = stat_from_map(&part.forced_stats, BodyPartStat::STAT_FLY_SPEED);
        let _ = spawn_bodypart_modifier!(fly_speed, FlySpeed);

        let vision = stat_from_map(&part.forced_stats, BodyPartStat::STAT_VISION);
        let _ = spawn_bodypart_modifier!(vision, Vision);

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
    if part.synergies.is_empty() {
        return;
    }

    let mut tags = ModifierTags::default();
    let mut offsets = HashMap::default();
    let mut mults = HashMap::default();
    for (tag_str, synergy) in &part.synergies {
        let tag_str = tag_str.trim();
        if tag_str.is_empty() {
            continue;
        }
        let tag = Tag::from(tag_str);
        tags.insert(tag.clone());
        match synergy {
            ModifierSynergySeri::Offset(value) => {
                if *value != 0.0 {
                    offsets.insert(tag, *value);
                }
            }
            ModifierSynergySeri::CopyFrac(value) => {
                if *value != 0.0 {
                    mults.insert(tag, *value);
                }
            }
        }
    }
    if tags.is_empty() {
        return;
    }
    cmd.entity(modifier_ent).insert(tags);
    if !offsets.is_empty() {
        cmd.entity(modifier_ent).insert(OffsetValForSelf(offsets));
    }
    if !mults.is_empty() {
        cmd.entity(modifier_ent)
            .insert(CopyFracOfOthersIntoSelf(mults));
    }
}
