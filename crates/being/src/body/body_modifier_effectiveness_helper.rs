use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use game_common::game_common_components::{Templ, TemplEntiRef};
use modifier_shared::modifier_components::{AppliedModifiers, CurrEffectiveValue, ModifierTarget};
use modifier_shared::modifier_types::HitpointsCapacity;
use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker};

use crate::body::{BodypartChildOfBodypart, Missing};

pub fn bodypart_effectiveness_ratio(current_hp: f32, max_hp: f32, is_missing: bool) -> f32 {
    if max_hp <= 0.0 {
        return 1.0;
    }
    if is_missing {
        return 0.0;
    }
    (current_hp.max(0.0) / max_hp).clamp(0.0, 1.0)
}

pub fn bodypart_modifier_effectiveness(
    target_ent: Entity,
    part_state_query: &Query<(Has<Templ>, Has<Missing>), With<BodypartChildOfBodypart>>,
    templ_ref_query: &Query<&TemplEntiRef>,
    part_applied_mods_query: &Query<&AppliedModifiers>,
    mods_query: &Query<(Entity, &ModifierTarget)>,
    curr_values_query: &Query<&mut CurrEffectiveValue>,
    hp_capacity_markers: &Query<(), With<HitpointsCapacity>>,
    damage_query: &Query<&crate::body::body_components::AccuDamage>,
) -> f32 {
    let Ok((is_templ, is_missing)) = part_state_query.get(target_ent) else {
        return 1.0;
    };
    if is_templ {
        return 1.0;
    }
    if is_missing {
        return 0.0;
    }

    let part_templ_ref = templ_ref_query.get(target_ent).ok();
    let mut effects = EntityHashSet::default();
    collect_applied_modifier_entities(&mut effects, target_ent, part_templ_ref, part_applied_mods_query);

    let mut max_hp = 0.0;
    for mod_ent in effects.iter() {
        let Ok((entity, target)) = mods_query.get(*mod_ent) else {
            continue;
        };
        let templ_ref = templ_ref_query.get(entity).ok();
        if target.0 != target_ent
            && part_templ_ref.map(|part_templ_ref| target.0 != part_templ_ref.0).unwrap_or(true)
        {
            continue;
        }
        if modifier_has_marker::<HitpointsCapacity>(entity, templ_ref, hp_capacity_markers) {
            max_hp += resolve_curr_effective_value(entity, templ_ref, curr_values_query).map_or(0.0, |value| value.0.max(0.0));
        }
    }

    if max_hp <= 0.0 {
        return 1.0;
    }

    let current_hp = damage_query
        .get(target_ent)
        .map_or(max_hp, |damage| (max_hp - damage.total).max(0.0));
    bodypart_effectiveness_ratio(current_hp, max_hp, false)
}

fn resolve_curr_effective_value(
    modi_ent: Entity,
    templ_ref: Option<&TemplEntiRef>,
    query: &Query<&mut CurrEffectiveValue>,
) -> Option<CurrEffectiveValue> {
    if let Ok(value) = query.get(modi_ent) {
        return Some(*value);
    }
    let Some(templ_ref) = templ_ref else {
        return None;
    };
    query.get(templ_ref.0).ok().map(|value| *value)
}
