use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use game_common::game_common_components::TimeBasedMultiplier;
use common::common_components::Tag;

use crate::modifier_components::*;

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
        Option<&BaseValue>,
        Option<&TimeBasedMultiplier>,
        Option<&ModifierTags>,
        Option<&OffsetValForSelf>,
        Option<&CopyMultOfOthersIntoSelf>,
        Option<&Antidote>,
    )>,
    mut effective_query: Query<&mut CurrEffectiveValue>,
) {
    let mut target_aggs: EntityHashMap<TargetAggregates> = EntityHashMap::new();
    let mut new_eff_values = Vec::new();

    for (_entity, target, base_value, time_multiplier, tags, _offsets, _copy_mults, antidote) in modifiers_query.iter() {
        let Some(raw_value) = compute_raw_value(base_value, None, time_multiplier) else { continue; };
        let tags = tags.cloned().unwrap_or_default();

        let target_agg = target_aggs.entry(target.0).or_default();
        for tag in tags.iter() {
            *target_agg.tag_counts.entry(tag.clone()).or_insert(0) += 1;
            *target_agg.tag_value_sum.entry(tag.clone()).or_insert(0.0) += raw_value;
        }
        if let Some(antidote) = antidote {
            for (tag, effectiveness) in antidote.0.iter() {
                *target_agg.antidote_sum.entry(tag.clone()).or_insert(0.0) += raw_value * effectiveness;
            }
        }
    }

    let mut computed_values: EntityHashMap<f32> = EntityHashMap::new();

    for (modi_entity, target, base_value, time_multiplier, tags, offsets, copy_mults, _antidote) in modifiers_query.iter() {
        let Some(target_agg) = target_aggs.get(&target.0) else { continue; };
        let Some(raw_value) = compute_raw_value(base_value, None, time_multiplier) else { continue; };
        let tags = tags.cloned().unwrap_or_default();

        let mut value = raw_value;

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

        if let Some(copy_mults) = copy_mults {
            for (tag, mult) in copy_mults.0.iter() {
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

