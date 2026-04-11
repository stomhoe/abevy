use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::system::SystemParam;
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_id_components::*;
use game_common::game_common_components::{Templ, TemplEntiRef, TimeBasedMultiplier};

use being::body::*;
use being::body::body_modifier_effectiveness_helper::bodypart_modifier_effectiveness;
use modifier_shared::{modifier_has_marker, resolve_modifier_component};
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
            tags.0.insert(HashId::from(tag.as_ref()));
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


#[inline]
fn compute_time_affected_value(base_value: Option<&BaseValue>, time_multiplier: Option<&TimeBasedMultiplier>) -> Option<f32> {
    let base_or_current = base_value.map(|b| b.0)?;
    let multiplier = time_multiplier.map_or(1.0, |tm| tm.sample());
    Some(base_or_current * multiplier)
}
#[derive(Default, Clone)]
pub struct TargetAggregate {
    tag_count: usize,
    tag_value_sum: f32,
    antidote_sum: f32,
}

pub struct ResolvedModifier {
    entity: Entity,
    target: Entity,
    time_based_value: f32,
    tags: ModifierTags,
    offsets: Option<OffsetValForSelf>,
    copy_fracs: Option<CopyFracOfOthersIntoSelf>,
    _antidote: Option<Antidote>,
}

#[derive(SystemParam)]
pub struct ModifierEffectiveValuesQueries<'w, 's> {
    modifiers_query: Query<'w, 's, (
        Entity,
        &'static ModifierTarget,
    )>,
    base_values_query: Query<'w, 's, &'static BaseValue>,
    time_multipliers_query: Query<'w, 's, &'static TimeBasedMultiplier>,
    modifier_tags_query: Query<'w, 's, &'static ModifierTags>,
    offset_vals_query: Query<'w, 's, &'static OffsetValForSelf>,
    copy_fracs_query: Query<'w, 's, &'static CopyFracOfOthersIntoSelf>,
    antidotes_query: Query<'w, 's, &'static Antidote>,
    part_state_query: Query<'w, 's, (Has<Templ>, Has<Missing>), With<BodypartChildOfBodypart>>,
    templ_ref_query: Query<'w, 's, &'static TemplEntiRef>,
    part_applied_mods_query: Query<'w, 's, &'static AppliedModifiers>,
    hp_capacity_markers: Query<'w, 's, (), With<HitpointsCapacity>>,
    manip_dex_markers: Query<'w, 's, (), With<ManipulationDexterity>>,
    manip_str_markers: Query<'w, 's, (), With<ManipulationStrength>>,
    damage_query: Query<'w, 's, &'static AccuDamage>,
    curr_values_query: Query<'w, 's, &'static mut CurrEffectiveValue>,
}

#[derive(SystemParam)]
pub struct ModifierEffectiveValuesLocals<'s> {
    resolved_modifiers: Local<'s, Vec<ResolvedModifier>>,
    target_aggs: Local<'s, EntityHashMap<HashIdMap<TargetAggregate>>>,
}

#[allow(unused_parens)]
pub fn update_modifier_effective_values(
    mut cmd: Commands,
    queries: ModifierEffectiveValuesQueries,
    mut locals: ModifierEffectiveValuesLocals,
) {
    let ModifierEffectiveValuesQueries {
        modifiers_query,
        base_values_query,
        time_multipliers_query,
        modifier_tags_query,
        offset_vals_query,
        copy_fracs_query,
        antidotes_query,
        part_state_query,
        templ_ref_query,
        part_applied_mods_query,
        hp_capacity_markers,
        manip_dex_markers,
        manip_str_markers,
        damage_query,
        curr_values_query,
    } = queries;
    let ModifierEffectiveValuesLocals {
        resolved_modifiers,
        target_aggs,
    } = &mut locals;
    let mut curr_values_query = curr_values_query;
    resolved_modifiers.clear();
    target_aggs.clear();

    for (entity, target, ) in modifiers_query.iter() {
        let modifier_templ_ref = templ_ref_query.get(entity).ok();
        let base_value = resolve_modifier_component(entity, modifier_templ_ref, &base_values_query);
        let time_multiplier = resolve_modifier_component(entity, modifier_templ_ref, &time_multipliers_query);
        let Some(mut raw_value) = compute_time_affected_value(base_value.as_ref(), time_multiplier.as_ref()) else { continue; };
        let tags = resolve_modifier_component(entity, modifier_templ_ref, &modifier_tags_query).unwrap_or_default();
        let offsets = resolve_modifier_component(entity, modifier_templ_ref, &offset_vals_query);
        let copy_fracs = resolve_modifier_component(entity, modifier_templ_ref, &copy_fracs_query);
        let antidote = resolve_modifier_component(entity, modifier_templ_ref, &antidotes_query);
        let effectiveness = if modifier_has_marker::<ManipulationDexterity>(entity, modifier_templ_ref, &manip_dex_markers)
            || modifier_has_marker::<ManipulationStrength>(entity, modifier_templ_ref, &manip_str_markers)
        {
            bodypart_modifier_effectiveness(
                target.0,
                &part_state_query,
                &templ_ref_query,
                &part_applied_mods_query,
                &modifiers_query,
                &curr_values_query,
                &hp_capacity_markers,
                &damage_query,
            )
        } else {
            1.0
        };
        raw_value *= effectiveness;

        let target_agg = target_aggs.entry(target.0).or_insert_with(|| HashIdMap::with_capacity(tags.len()));
        for tag in tags.iter() {
            let tag_hash = *tag;
            let target_agg = target_agg.0.entry(tag_hash).or_default();
            target_agg.tag_count += 1;
            target_agg.tag_value_sum += raw_value;
        }
        if let Some(antidote) = antidote.as_ref() {
            for (tag, effectiveness) in antidote.0.iter() {
                let tag_hash = HashId::from(tag.as_ref());
                let target_agg = target_agg.0.entry(tag_hash).or_default();
                target_agg.antidote_sum += raw_value * effectiveness;
            }
        }

        resolved_modifiers.push(ResolvedModifier {
            entity,
            target: target.0,
            time_based_value: raw_value,
            tags,
            offsets,
            copy_fracs,
            _antidote: antidote,
        });
    }

    let mut computed_values: EntityHashMap<CurrEffectiveValue> = EntityHashMap::with_capacity(resolved_modifiers.len());

    for resolved in resolved_modifiers.iter() {
        let Some(target_agg) = target_aggs.get(&resolved.target) else { continue; };
        let mut value = resolved.time_based_value;

        if let Some(offsets) = resolved.offsets.as_ref() {
            for (tag, offset) in offsets.0.iter() {
                let tag_hash = HashId::from(tag.as_ref());
                let Some(tag_agg) = target_agg.get_opt(tag_hash) else { continue; };
                let has_other = if resolved.tags.contains(&tag_hash) { tag_agg.tag_count > 1 } else { tag_agg.tag_count > 0 };
                if has_other {
                    value += offset;
                }
            }
        }
        if let Some(copy_fracs) = resolved.copy_fracs.as_ref() {
            for (tag, mult) in copy_fracs.0.iter() {
                let tag_hash = HashId::from(tag.as_ref());
                let Some(tag_agg) = target_agg.get_opt(tag_hash) else { continue; };
                let mut copy_value = tag_agg.tag_value_sum;
                if resolved.tags.contains(&tag_hash) {
                    copy_value -= resolved.time_based_value;
                }
                if copy_value != 0.0 {
                    value += copy_value * mult.clamp(0.0, 1.0);
                }
            }
        }
        for tag in resolved.tags.iter() {
            if let Some(tag_agg) = target_agg.get_opt(*tag) {
                value -= tag_agg.antidote_sum;
            }
        }
        computed_values.insert(resolved.entity, CurrEffectiveValue(value));
    }
    computed_values.retain(|entity, new_value| match curr_values_query.get_mut(*entity) {
        Ok(mut current) => {
            *current = *new_value;
            false
        }
        Err(_) => true,
    });
    cmd.try_insert_batch(computed_values);
}

pub fn sync_modifier_name_to_effects(
    mut modifiers_query: Query<
        (
            Entity,
            &mut Name,
            Option<&TemplEntiRef>,
            Has<WalkStrength>,
            Has<FlyStrength>,
            Has<SwimStrength>,
            Has<BleedRate>,
            Has<InvertMovement>,
            Has<PainSlowdown>,
            Has<HitpointsCapacity>,
            Has<HitpointRegenRate>,
        ),
        (
            With<ModifierTarget>,
            Or<(
                Or<(
                    Added<WalkStrength>,
                    Added<FlyStrength>,
                    Added<SwimStrength>,
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
                    Changed<TemplEntiRef>,
                )>,
            )>,
        ),
    >,
    curr_values: Query<&CurrEffectiveValue>,
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
        templ_ref,
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
        let curr_value = resolve_modifier_component(entity, templ_ref, &curr_values);

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
