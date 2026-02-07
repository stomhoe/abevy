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

#[allow(unused_parens)]
pub fn init_body_parts(
    mut cmd: Commands,
    part_map: Res<BodyPartEntityMap>,
    seris_handles: Res<BodyPartSerisHandles>,
    assets: Res<Assets<BodyPartSeri>>,
) {
    if !part_map.0.is_empty() {
        return;
    }

    let mut spawned_ids: HashSet<StrId> = HashSet::default();

    for handle in seris_handles.handles.iter() {
        let Some(seri) = assets.get(handle) else {
            continue;
        };

        debug!(target: "body_init", "Loading BodyPartSeri from handle: {:?}", handle);

        let mut part = seri.clone();
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

        if let Some(name) = part.name.take() {
            if !name.trim().is_empty() {
                cmd.entity(part_ent).insert(DisplayName::trunc(name));
            }
        } else {
            cmd.entity(part_ent)
                .insert(DisplayName::trunc(part_id.as_str()));
        }

        if let Some(slots) = part.slots.take() {
            if !slots.is_empty() {
                cmd.entity(part_ent).insert(BodyPartSlots::new(slots));
            }
        }

        if let Some(tags) = part.tags.take() {
            if !tags.is_empty() {
                cmd.entity(part_ent).insert(TagSet::new(tags));
            }
        }

        if let Some(weight) = part.coverage_weight {
            cmd.entity(part_ent).insert(BodyPartCoverageWeight(weight));
        }

        if let Some(max_hp) = part.hp_capacity {
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

        if let Some(hp_regen_rate) = part.hp_regen_rate {
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

        if let Some(bleed_rate) = part.bleed_rate {
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

        if let Some(blood_capacity) = part.blood_capacity {
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

        if let Some(pain_sensitivity) = part.pain_sensitivity {
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

        if let Some(manipulation) = part.manipulation {
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

        if let Some(walk_speed) = part.walk_speed {
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

        if let Some(swim_speed) = part.swim_speed {
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

        if let Some(fly_speed) = part.fly_speed {
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

        if let Some(vision) = part.vision {
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

        if let Some(depth) = part.depth.take() {
            cmd.entity(part_ent).insert(BodyPartDepth::from(depth));
        }

        if let Some(kind) = part.kind.take().filter(|k| !k.trim().is_empty()) {
            cmd.entity(part_ent)
                .insert(BodyPartKind(StrId::trunc(kind)));
        }

        if part.vital == Some(true) {
            cmd.entity(part_ent).insert(BodyPartVital);
        }
    }
}

fn apply_synergy_to_modifier(cmd: &mut Commands, modifier_ent: Entity, part: &BodyPartSeri) {
    let Some(tag_str) = part
        .synergy_tag
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    else {
        return;
    };

    let tag = Tag::from(tag_str);

    let mut tags = ModifierTags::default();
    tags.insert(tag.clone());
    cmd.entity(modifier_ent).insert(tags);

    if let Some(offset) = part.synergy_offset {
        if offset != 0.0 {
            let mut offsets = HashMap::default();
            offsets.insert(tag.clone(), offset);
            cmd.entity(modifier_ent).insert(OffsetValForSelf(offsets));
        }
    }

    if let Some(copy_mult) = part.synergy_copy_mult {
        if copy_mult != 0.0 {
            let mut mults = HashMap::default();
            mults.insert(tag, copy_mult);
            cmd.entity(modifier_ent)
                .insert(CopyMultOfOthersIntoSelf(mults));
        }
    }
}
