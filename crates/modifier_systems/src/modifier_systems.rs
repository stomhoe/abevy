use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use game_common::game_common_components::{EntityZero, EntityZeroRef, TimeBasedMultiplier};
use common::common_components::Tag;

use modifier_shared::resolve_modifier_component;
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::*;

pub fn materialize_modifier_synergies(
    mut cmd: Commands,
    query: Query<
        (
            Entity,
            &ModifierSynergies,
            Option<&ModifierTags>,
            Option<&OffsetValForSelf>,
            Option<&CopyFracOfOthersIntoSelf>,
        ),
        Changed<ModifierSynergies>,
    >,
) {
    for (entity, synergies, existing_tags, existing_offsets, existing_mults) in query.iter() {
        let mut tags = existing_tags.cloned().unwrap_or_default();
        let mut offsets = existing_offsets.cloned().unwrap_or_default();
        let mut mults = existing_mults.cloned().unwrap_or_default();

        for (tag, synergy) in &synergies.0 {
            tags.insert(tag.clone());
            match synergy {
                ModifierSynergy::Offset(value) => {
                    if *value != 0.0 {
                        offsets.0.insert(tag.clone(), *value);
                    }
                }
                ModifierSynergy::CopyFrac(value) => {
                    if *value != 0.0 {
                        mults.0.insert(tag.clone(), *value);
                    }
                }
            }
        }

        cmd.entity(entity).insert(tags);
        if !offsets.0.is_empty() {
            cmd.entity(entity).insert(offsets);
        }
        if !mults.0.is_empty() {
            cmd.entity(entity).insert(mults);
        }
    }
}

#[derive(Default)]
struct TargetAggregates {
    tag_counts: HashMap<Tag, usize>,
    tag_value_sum: HashMap<Tag, f32>,
    antidote_sum: HashMap<Tag, f32>,
}

#[inline]
fn compute_raw_value(base_value: Option<&BaseValue>, current_value: Option<&CurrEffectiveValue>, time_multiplier: Option<&TimeBasedMultiplier>) -> Option<f32> {
    let base_or_current = base_value.map(|b| b.0).or_else(|| current_value.map(|v| v.0))?;
    let multiplier = time_multiplier.map_or(1.0, |tm| tm.sample());
    Some(base_or_current * multiplier)
}

#[allow(unused_parens)]
pub fn update_modifier_effective_values(
    mut cmd: Commands,
    modifiers_query: Query<(
        Entity,
        &ModifierTarget,
        Option<&EntityZeroRef>,
        Option<&BaseValue>,
        Option<&TimeBasedMultiplier>,
        Option<&ModifierTags>,
        Option<&OffsetValForSelf>,
        Option<&CopyFracOfOthersIntoSelf>,
        Option<&Antidote>,
    )>,
    base_values_query: Query<&BaseValue>,
    time_multipliers_query: Query<&TimeBasedMultiplier>,
    modifier_tags_query: Query<&ModifierTags>,
    offset_vals_query: Query<&OffsetValForSelf>,
    copy_fracs_query: Query<&CopyFracOfOthersIntoSelf>,
    antidotes_query: Query<&Antidote>,
    mut effective_query: Query<&mut CurrEffectiveValue>,
) {
    let mut target_aggs: EntityHashMap<TargetAggregates> = EntityHashMap::new();
    let mut new_eff_values = Vec::new();

    for (entity, target, ezero_ref, _base_value, _time_multiplier, _tags, _offsets, _copy_fracs, _antidote) in modifiers_query.iter() {
        let base_value = resolve_modifier_component(entity, ezero_ref, &base_values_query);
        let time_multiplier = resolve_modifier_component(entity, ezero_ref, &time_multipliers_query);
        let Some(raw_value) = compute_raw_value(base_value.as_ref(), None, time_multiplier.as_ref()) else { continue; };
        let tags = resolve_modifier_component(entity, ezero_ref, &modifier_tags_query).unwrap_or_default();

        let target_agg = target_aggs.entry(target.0).or_default();
        for tag in tags.iter() {
            *target_agg.tag_counts.entry(tag.clone()).or_insert(0) += 1;
            *target_agg.tag_value_sum.entry(tag.clone()).or_insert(0.0) += raw_value;
        }
        let antidote = resolve_modifier_component(entity, ezero_ref, &antidotes_query);
        if let Some(antidote) = antidote {
            for (tag, effectiveness) in antidote.0.iter() {
                *target_agg.antidote_sum.entry(tag.clone()).or_insert(0.0) += raw_value * effectiveness;
            }
        }
    }

    let mut computed_values: EntityHashMap<f32> = EntityHashMap::new();

    for (modi_entity, target, ezero_ref, _base_value, _time_multiplier, _tags, _offsets, _copy_fracs, _antidote) in modifiers_query.iter() {
        let Some(target_agg) = target_aggs.get(&target.0) else { continue; };
        let base_value = resolve_modifier_component(modi_entity, ezero_ref, &base_values_query);
        let time_multiplier = resolve_modifier_component(modi_entity, ezero_ref, &time_multipliers_query);
        let Some(raw_value) = compute_raw_value(base_value.as_ref(), None, time_multiplier.as_ref()) else { continue; };
        let tags = resolve_modifier_component(modi_entity, ezero_ref, &modifier_tags_query).unwrap_or_default();

        let mut value = raw_value;

        let offsets = resolve_modifier_component(modi_entity, ezero_ref, &offset_vals_query);
        if let Some(offsets) = offsets {
            for (tag, offset) in offsets.0.iter() {
                let Some(count) = target_agg.tag_counts.get(tag) else { continue; };
                let has_other = if tags.contains(tag.clone()) {
                    *count > 1
                } else {
                    *count > 0
                };
                if has_other {
                    value += offset;
                }
            }
        }

        let copy_fracs = resolve_modifier_component(modi_entity, ezero_ref, &copy_fracs_query);
        if let Some(copy_fracs) = copy_fracs {
            for (tag, mult) in copy_fracs.0.iter() {
                let Some(sum) = target_agg.tag_value_sum.get(tag) else { continue; };
                let mut copy_value = *sum;
                if tags.contains(tag.clone()) {
                    copy_value -= raw_value;
                }
                if copy_value != 0.0 {
                    value += copy_value * mult.clamp(0.0, 1.0);
                }
            }
        }

        for tag in tags.iter() {
            if let Some(counter) = target_agg.antidote_sum.get(tag) {
                value -= *counter;
            }
        }

        computed_values.insert(modi_entity, value);
    }

    for (&entity, &new_value) in computed_values.iter() {
        match effective_query.get_mut(entity) {
            Ok(mut current) => current.0 = new_value,
            Err(_) => new_eff_values.push((entity, CurrEffectiveValue(new_value))),
        }
    }
    cmd.try_insert_batch(new_eff_values);
}

pub fn sync_modifier_name_to_effects(
    mut modifiers_query: Query<
        (
            Entity,
            &mut Name,
            Option<&EntityZeroRef>,
            Option<&CurrEffectiveValue>,
            Has<WalkSpeed>,
            Has<FlySpeed>,
            Has<SwimSpeed>,
            Has<BleedRate>,
            Has<InvertMovement>,
            Has<PainSlowdown>,
            Has<HitpointsCapacity>,
            Has<HitpointRegenRate>,
        ),
        (
            Without<EntityZero>,
            Or<(
                Or<(
                    Added<WalkSpeed>,
                    Added<FlySpeed>,
                    Added<SwimSpeed>,
                    Added<BleedRate>,
                    Added<InvertMovement>,
                    Added<PainSlowdown>,
                    Added<HitpointsCapacity>,
                    Added<HitpointRegenRate>,
                    Added<BloodCapacity>,
                )>,
                Or<(
                    Added<Consciousness>,
                    Added<PainSensitivity>,
                    Added<PainInfliction>,
                    Added<ManipulationDexterity>,
                    Added<ManipulationStrength>,
                    Added<Vision>,
                    Added<MinForDamage>,
                    Changed<CurrEffectiveValue>,
                )>,
            )>,
        ),
    >,
    ezero_curr_values: Query<&CurrEffectiveValue>,
    effects_query: Query<
        (
            Has<BloodCapacity>,
            Has<Consciousness>,
            Has<PainSensitivity>,
            Has<PainInfliction>,
            Has<ManipulationDexterity>,
            Has<ManipulationStrength>,
            Has<Vision>,
            Has<MinForDamage>,
        ),
    >,
) {
    for (
        entity,
        mut name,
        ezero_ref,
        curr_value,
        has_walk_speed,
        has_fly_speed,
        has_swim_speed,
        has_bleed_rate,
        has_invert_movement,
        has_pain_slowdown,
        has_hitpoints_capacity,
        has_hitpoint_regen_rate,
    ) in modifiers_query.iter_mut() {
        let Ok((
            has_blood_capacity,
            has_consciousness,
            has_pain_sensitivity,
            has_pain_infliction,
            has_manipulation,
            has_manipulation_strength,
            has_vision,
            has_min_for_damage,
        )) = effects_query.get(entity) else { continue; };
        let curr_value = resolve_modifier_component(entity, ezero_ref, &ezero_curr_values);

        let mut effects = Vec::with_capacity(17);
        if has_walk_speed { effects.push("Walk"); }
        if has_fly_speed { effects.push("Fly"); }
        if has_swim_speed { effects.push("Swim"); }
        if has_bleed_rate { effects.push("Bleed"); }
        if has_invert_movement { effects.push("InvMove"); }
        if has_pain_slowdown { effects.push("PainSlow"); }
        if has_hitpoints_capacity { effects.push("HpCap"); }
        if has_hitpoint_regen_rate { effects.push("HpRegen"); }
        if has_blood_capacity { effects.push("BloodCap"); }
        if has_consciousness { effects.push("Consc"); }
        if has_pain_sensitivity { effects.push("PainSens"); }
        if has_pain_infliction { effects.push("PainInfli"); }
        if has_manipulation { effects.push("ManipDex"); }
        if has_manipulation_strength { effects.push("ManipStr"); }
        if has_vision { effects.push("Vis"); }
        if has_min_for_damage { effects.push("MinForDamage"); }

        let effects_label = if effects.is_empty() {
            "Modifier".to_string()
        } else {
            effects.join("|")
        };
        let curr_label = curr_value.map(|v| format!("{:.2}", v.0)).unwrap_or_else(|| "n/a".to_string());
        *name = Name::new(format!("{} [{}]", effects_label, curr_label));
    }
}
