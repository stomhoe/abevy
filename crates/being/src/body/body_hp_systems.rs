use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::file_logging::file_log;
use common::log_targets::BODY_HP_SYSTEM;
use game_common::game_common_components::{Templ, TemplEntiRef};
use rand::RngExt;
use modifier_shared::resolve_modifier_component;
use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker};
use modifier_shared::{modifier_components::*, modifier_types::*};
use ::being_shared::*;
use game_common::game_common_components::Dead;

use crate::body::{body_components::*, };

#[derive(Resource, Default)]
pub struct BodypartMaxHpMap(pub EntityHashMap<f32>);

#[derive(Resource, Default)]
pub struct BodypartTemplateByPart(pub EntityHashMap<Entity>);

#[derive(Component, Debug, Default, Clone)]
pub struct UserBodypartInstances(pub Vec<Entity>);

fn remove_part_from_bucket(bucket: &mut Vec<Entity>, part_ent: Entity) {
    if let Some(idx) = bucket.iter().position(|ent| *ent == part_ent) {
        bucket.swap_remove(idx);
    }
}

#[allow(unused_parens, )]
pub fn refresh_template_bodyparts_users_list(
    mut templ_by_part: ResMut<BodypartTemplateByPart>,
    mut templ_instances_query: Query<&mut UserBodypartInstances>,
    changed_parts_query: Query<(Entity, &TemplEntiRef), (With<BodypartChildOfBodypart>, Changed<TemplEntiRef>, Without<Templ>)>,
    mut removed_templ_refs: RemovedComponents<TemplEntiRef>,
) {
    for part_ent in removed_templ_refs.read() {
        let Some(prev_templ_ent) = templ_by_part.0.remove(&part_ent) else {
            continue;
        };
        let Ok(mut instances) = templ_instances_query.get_mut(prev_templ_ent) else {
            continue;
        };
        remove_part_from_bucket(&mut instances.0, part_ent);
    }

    for (part_ent, part_templ_ref) in changed_parts_query.iter() {
        let new_templ_ent = part_templ_ref.0;
        let prev_templ_ent = templ_by_part.0.insert(part_ent, new_templ_ent);
        if prev_templ_ent == Some(new_templ_ent) {
            continue;
        }

        if let Some(prev_templ_ent) = prev_templ_ent {
            if let Ok(mut instances) = templ_instances_query.get_mut(prev_templ_ent) {
                remove_part_from_bucket(&mut instances.0, part_ent);
            }
        }

        let Ok(mut instances) = templ_instances_query.get_mut(new_templ_ent) else {
            continue;
        };
        instances.0.push(part_ent);
    }
}

#[allow(unused_parens)]
pub fn update_bodypart_max_hp_map(
    mut max_hp_by_part: ResMut<BodypartMaxHpMap>,
    parts_query: Query<(Entity, Option<&TemplEntiRef>, ), (With<BodypartChildOfBodypart>, Without<Templ>, )>,
    applied_mods_query: Query<&AppliedModifiers>,
    templ_enti_refs_query: Query<&TemplEntiRef>,
    hp_mods_query: Query<(), (With<HitpointsCapacity>,)>,
    curr_values_query: Query<&CurrEffectiveValue>,
    mut effects: Local<EntityHashSet>,
) {
    max_hp_by_part.0.clear();

    for (part_ent, part_templ_ref, ) in parts_query.iter() {
        effects.clear();
        collect_applied_modifier_entities(
            &mut effects,
            part_ent,
            part_templ_ref,
            &applied_mods_query,
        );
        let mut max_hp = 0.0;
        let mut modifiers_seen = 0usize;
        let mut hp_modifiers_seen = 0usize;
        for mod_ent in effects.iter() {
            modifiers_seen += 1;
            let templ_ref = templ_enti_refs_query.get(*mod_ent).ok();
            let has_hp_marker = hp_mods_query.get(*mod_ent).is_ok()
                || templ_ref.is_some_and(|templ_ref| hp_mods_query.get(templ_ref.0).is_ok());
            if !has_hp_marker {
                continue;
            }
            hp_modifiers_seen += 1;
            let current_value = resolve_modifier_component(*mod_ent, templ_ref, &curr_values_query);
            let value = current_value
                .map(|v| v.0)
                .unwrap_or_default();
            max_hp += value;
        }
        if max_hp > 0.0 {
            max_hp_by_part.0.insert(part_ent, max_hp);
            continue;
        }
        if hp_modifiers_seen > 0 {
            file_log(
                BODY_HP_SYSTEM,
                "host",
                &format!(
                    "part_zero_cap_with_hp_markers part={part_ent:?} templ_part={:?} hp_modifiers_seen={hp_modifiers_seen} max_hp={max_hp:.3}",
                    part_templ_ref.map(|templ_ref| templ_ref.0),
                ),
            );
        }
        if modifiers_seen > 0 && hp_modifiers_seen == 0 {
            file_log(
                BODY_HP_SYSTEM,
                "host",
                &format!(
                    "part_no_hp_markers part={part_ent:?} templ_part={:?} modifiers_seen={modifiers_seen}",
                    part_templ_ref.map(|templ_ref| templ_ref.0),
                ),
            );
            trace!(
                target: BODY_HP_SYSTEM,
                "Part {:?} has {} modifier refs but no HitpointsCapacity marker found (templ_part={:?})",
                part_ent,
                modifiers_seen,
                part_templ_ref.map(|templ_ref| templ_ref.0),
            );
        }
    }
}



#[allow(unused_parens, )]
pub fn apply_damage(
    mut cmd: Commands,
    mut reader: MessageReader<IncHealthDamageOrHeal>,
    max_hp_by_part: Res<BodypartMaxHpMap>,
    target_body_query: Query<&HeldBody>,
    bodies_query: Query<&Children, (With<BodyOf>, Without<Templ>)>,
    parts_query: Query<
        (Option<&BodypartCoverageWeight>, ),
        (With<BodypartChildOfBodypart>, Without<Missing>, Without<Templ>),
    >,
    mut damage_query: Query<&mut AccuDamage, (Without<Templ>, )>,
    mut weighted_parts: Local<Vec<(Entity, u16)>>,
    mut body_parts: Local<Vec<Entity>>,
    mut already_hurt_parts: Local<Vec<(Entity, f32)>>,
) {
    let mut rng = rand::rng();
    for damage_msg in reader.read() {
        weighted_parts.clear();
        body_parts.clear();
        already_hurt_parts.clear();
        let mut total_weight: u32 = 0;

        let body_ent = target_body_query
            .get(damage_msg.target_ent)
            .map(|held_body| held_body.entity())
            .unwrap_or(damage_msg.target_ent);
        let damage_amount = damage_msg.amount;
        let source_ent = damage_msg.source_ent;

        let Ok(parts) = bodies_query.get(body_ent) else {
            continue;
        };

        for part_ent in parts.iter() {
            let Ok((weight_opt, )) = parts_query.get(part_ent) else {
                continue;
            };
            let weight = weight_opt.map(|w| w.0).unwrap_or(1).max(1);
            total_weight += weight as u32;
            weighted_parts.push((part_ent, weight));
            body_parts.push(part_ent);
            let Ok(part_damage) = damage_query.get_mut(part_ent) else {
                continue;
            };
            if part_damage.total > 0.0 {
                already_hurt_parts.push((part_ent, part_damage.total));
            }
        }

        match damage_msg.distribute_mode {
            DamageDistributeMode::SampledBodyPart => {
                let Some(total_weight) = (total_weight > 0).then_some(total_weight) else {
                    continue;
                };
                let mut roll = rng.random_range(0..total_weight);

                let mut selected = None;
                for (part_ent, weight) in weighted_parts.iter() {
                    let weight = *weight as u32;
                    if roll < weight {
                        selected = Some(*part_ent);
                        break;
                    }
                    roll -= weight;
                }
                let Some(part_ent) = selected else {
                    continue;
                };
                apply_damage_to_part(&mut cmd, part_ent, damage_amount, source_ent, &mut damage_query);
            }
            DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity => {
                let mut total_capacity = 0.0;
                for part_ent in body_parts.iter() {
                    total_capacity += max_hp_by_part.0.get(part_ent).copied().unwrap_or(0.0).max(0.0);
                }
                if total_capacity <= 0.0 {
                    continue;
                }
                for part_ent in body_parts.iter() {
                    let part_capacity = max_hp_by_part.0.get(part_ent).copied().unwrap_or(0.0).max(0.0);
                    if part_capacity <= 0.0 {
                        continue;
                    }
                    let part_damage = damage_amount * (part_capacity / total_capacity);
                    apply_damage_to_part(&mut cmd, *part_ent, part_damage, source_ent, &mut damage_query);
                }
            }
            DamageDistributeMode::DistributeProportionalToPreexistentDamage => {
                let mut total_existing_damage = 0.0;
                for (_, part_damage) in already_hurt_parts.iter() {
                    total_existing_damage += *part_damage;
                }
                if total_existing_damage <= 0.0 {
                    continue;
                }
                for (part_ent, part_existing_damage) in already_hurt_parts.iter() {
                    let part_damage = damage_amount * (*part_existing_damage / total_existing_damage);
                    apply_damage_to_part(&mut cmd, *part_ent, part_damage, source_ent, &mut damage_query);
                }
            }
        }
    }
}

#[inline]
fn apply_damage_to_part(
    cmd: &mut Commands,
    part_ent: Entity,
    amount: f32,
    source_ent: Entity,
    damage_query: &mut Query<&mut AccuDamage, (Without<Templ>, )>,
) {
    if amount == 0.0 {
        return;
    }
    let Ok(mut part_damage) = damage_query.get_mut(part_ent) else {
        if amount <= 0.0 {
            return;
        }
        cmd.entity(part_ent).try_insert(AccuDamage::with_hit(amount, source_ent));
        return;
    };
    part_damage.push_hit(amount, source_ent);
}

#[allow(unused_parens, )]
pub fn set_bodypart_as_missing_if_0_hp(
    mut cmd: Commands,
    changed_bodyparts_query: Query<
        (Entity, &AccuDamage, Has<Missing>),
        (With<BodypartChildOfBodypart>, Without<Templ>, Changed<AccuDamage>),
    >,
    bodyparts_query: Query<
        (&AccuDamage, Has<Vital>, Has<Missing>),
        (With<BodypartChildOfBodypart>, Without<Templ>),
    >,
    time: Res<Time>,
    child_of_query: Query<&ChildOf>,
    max_hp_by_part: Res<BodypartMaxHpMap>,
    mut missing_timers: Local<EntityHashMap<Timer>>,
    mut stale_timers: Local<Vec<Entity>>,
    mut finished_timers: Local<Vec<(Entity, bool)>>,
) {
    for (part_ent, damage, has_missing) in changed_bodyparts_query.iter() {
        let max_hp = max_hp_by_part.0.get(&part_ent).copied().unwrap_or(0.0).max(0.0);
        if max_hp <= 0.0 {
            missing_timers.remove(&part_ent);
            if has_missing {
                cmd.entity(part_ent).try_remove::<Missing>();
            }
            continue;
        }

        let current_hp = max_hp - damage.total;
        let current_hp = current_hp.clamp(0.0, max_hp);

        if current_hp <= 0.0 {
            missing_timers
                .entry(part_ent)
                .or_insert_with(|| Timer::from_seconds(0.1, TimerMode::Once));
        } else {
            missing_timers.remove(&part_ent);
            if has_missing {
                cmd.entity(part_ent).try_remove::<Missing>();
            }
        }
    }

    stale_timers.clear();
    finished_timers.clear();
    for (part_ent, timer) in missing_timers.iter_mut() {
        let Ok((damage, is_vital, has_missing)) = bodyparts_query.get(*part_ent) else {
            stale_timers.push(*part_ent);
            continue;
        };

        let max_hp = max_hp_by_part.0.get(part_ent).copied().unwrap_or(0.0).max(0.0);
        if max_hp <= 0.0 {
            stale_timers.push(*part_ent);
            if has_missing {
                cmd.entity(*part_ent).try_remove::<Missing>();
            }
            continue;
        }

        let current_hp = (max_hp - damage.total).clamp(0.0, max_hp);
        if current_hp > 0.0 {
            stale_timers.push(*part_ent);
            if has_missing {
                cmd.entity(*part_ent).try_remove::<Missing>();
            }
            continue;
        }

        timer.tick(time.delta());
        if timer.is_finished() {
            finished_timers.push((*part_ent, is_vital));
            stale_timers.push(*part_ent);
        }
    }

    for part_ent in stale_timers.drain(..) {
        missing_timers.remove(&part_ent);
    }

    for (part_ent, is_vital) in finished_timers.drain(..) {
        cmd.entity(part_ent).try_insert_if_new(Missing);
        if !is_vital {
            continue;
        }
        let Ok(child_of) = child_of_query.get(part_ent) else {
            continue;
        };
        let Ok(child_of) = child_of_query.get(child_of.parent()) else {
            continue;
        };
        let being_ent = child_of.parent();
        cmd.entity(being_ent).try_insert_if_new((Dead));
    }
}

#[derive(SystemParam)]
pub struct BodyHealthQueryParams<'w, 's> {
    bodies_query: Query<'w, 's, (Entity, &'static BodyOf), Without<Templ>>,
    parts_query: Query<'w, 's, (Entity, &'static ChildOf), (With<BodypartChildOfBodypart>, Without<Templ>, Without<Missing>)>,
    templ_enti_refs_query: Query<'w, 's, &'static TemplEntiRef>,
    user_body_instances_query: Query<'w, 's, &'static UserBodypartInstances>,
    part_applied_mods_query: Query<'w, 's, &'static AppliedModifiers>,
    modifier_target_query: Query<'w, 's, &'static ModifierTarget>,
    pain_infliction_query: Query<'w, 's, (), With<PainInfliction>>,
    pain_sensitivity_query: Query<'w, 's, (), With<PainSensitivity>>,
    bleed_query: Query<'w, 's, (), With<BleedRate>>,
    blood_capacity_query: Query<'w, 's, (), With<BloodCapacity>>,
    consciousness_query: Query<'w, 's, (), With<Consciousness>>,
    vision_query: Query<'w, 's, (), With<Vision>>,
    damage_query: Query<'w, 's, &'static mut AccuDamage>,
    body_sums_query: Query<'w, 's, &'static mut BodySums>,
    curr_values_query: Query<'w, 's, &'static CurrEffectiveValue>,
}

#[derive(SystemParam)]
pub struct BodyHealthLocalParams<'s> {
    pain_infliction_by_part: Local<'s, EntityHashMap<f32>>,
    pain_sensitivity_by_part: Local<'s, EntityHashMap<f32>>,
    pain_infliction_by_body: Local<'s, EntityHashMap<f32>>,
    pain_sensitivity_by_body: Local<'s, EntityHashMap<f32>>,
    bleed_mod_sum: Local<'s, EntityHashMap<f32>>,
    blood_capacity_mod_sum: Local<'s, EntityHashMap<f32>>,
    consciousness_mod_sum: Local<'s, EntityHashMap<f32>>,
    vision_mod_sum: Local<'s, EntityHashMap<f32>>,
    body_modifiers: Local<'s, EntityHashSet>,
    body_parts_by_body: Local<'s, EntityHashMap<Vec<(Entity, Entity)>>>,
}

#[allow(unused_parens, )]
pub fn update_body_health_from_parts(
    mut cmd: Commands,
    time: Res<Time>,
    max_hp_by_part: Res<BodypartMaxHpMap>,
    mut queries: BodyHealthQueryParams,
    mut locals: BodyHealthLocalParams,
) {
    locals.pain_infliction_by_part.clear();
    locals.pain_sensitivity_by_part.clear();
    locals.pain_infliction_by_body.clear();
    locals.pain_sensitivity_by_body.clear();
    locals.bleed_mod_sum.clear();
    locals.blood_capacity_mod_sum.clear();
    locals.consciousness_mod_sum.clear();
    locals.vision_mod_sum.clear();
    locals.body_parts_by_body.clear();

    for (part_ent, part_child_of) in queries.parts_query.iter() {
        let Ok(part_templ_ref) = queries.templ_enti_refs_query.get(part_ent) else {
            continue;
        };
        locals
            .body_parts_by_body
            .entry(part_child_of.parent())
            .or_insert_with(Vec::new)
            .push((part_ent, part_templ_ref.0));
    }

    for (body, body_of) in queries.bodies_query.iter() {
        let mut total_max_hp = 0.0;
        let mut total_hp = 0.0;
        let mut total_pain = 0.0;
        let mut parts_count = 0usize;
        let bleed_rate = 0.0;
        let blood_capacity = 0.0;
        let body_templ_ref = queries.templ_enti_refs_query.get(body).ok();

        locals.body_modifiers.clear();
        collect_applied_modifier_entities(
            &mut locals.body_modifiers,
            body,
            body_templ_ref,
            &queries.part_applied_mods_query,
        );
        for &mod_ent in locals.body_modifiers.iter() {
            let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
            let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
                continue;
            };
            let Ok(target) = queries.modifier_target_query.get(mod_ent) else {
                continue;
            };
            if modifier_has_marker(mod_ent, templ_ref, &queries.pain_infliction_query) {
                add_part_or_body_modifier_sum(
                    &mut locals.pain_infliction_by_part,
                    &mut locals.pain_infliction_by_body,
                    target.0,
                    value.0,
                    &queries.bodies_query,
                    &queries.parts_query,
                    &queries.user_body_instances_query,
                );
            }
            if modifier_has_marker(mod_ent, templ_ref, &queries.pain_sensitivity_query) {
                add_part_or_body_modifier_sum(
                    &mut locals.pain_sensitivity_by_part,
                    &mut locals.pain_sensitivity_by_body,
                    target.0,
                    value.0,
                    &queries.bodies_query,
                    &queries.parts_query,
                    &queries.user_body_instances_query,
                );
            }
            if modifier_has_marker(mod_ent, templ_ref, &queries.bleed_query) {
                add_modifier_sum(
                    &mut locals.bleed_mod_sum,
                    target.0,
                    value.0,
                    &queries.bodies_query,
                    &queries.parts_query,
                    &queries.user_body_instances_query,
                );
            }
            if modifier_has_marker(mod_ent, templ_ref, &queries.blood_capacity_query) {
                add_modifier_sum(
                    &mut locals.blood_capacity_mod_sum,
                    target.0,
                    value.0,
                    &queries.bodies_query,
                    &queries.parts_query,
                    &queries.user_body_instances_query,
                );
            }
            if modifier_has_marker(mod_ent, templ_ref, &queries.consciousness_query) {
                add_modifier_sum(
                    &mut locals.consciousness_mod_sum,
                    target.0,
                    value.0,
                    &queries.bodies_query,
                    &queries.parts_query,
                    &queries.user_body_instances_query,
                );
            }
            if modifier_has_marker(mod_ent, templ_ref, &queries.vision_query) {
                add_modifier_sum(
                    &mut locals.vision_mod_sum,
                    target.0,
                    value.0,
                    &queries.bodies_query,
                    &queries.parts_query,
                    &queries.user_body_instances_query,
                );
            }
        }

        if let Some(body_parts) = locals.body_parts_by_body.get(&body) {
            parts_count = body_parts.len();
            for (part_ent, part_templ_ent) in body_parts.iter().copied() {
                locals.body_modifiers.clear();
                let part_templ_ref = TemplEntiRef(part_templ_ent);
                collect_applied_modifier_entities(
                    &mut locals.body_modifiers,
                    part_ent,
                    Some(&part_templ_ref),
                    &queries.part_applied_mods_query,
                );
                for &mod_ent in locals.body_modifiers.iter() {
                    let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
                    let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
                        continue;
                    };
                    let Ok(target) = queries.modifier_target_query.get(mod_ent) else {
                        continue;
                    };
                    let target_is_this_part = target.0 == part_ent || target.0 == part_templ_ent;
                    let Some(target_part_ent) = target_is_this_part.then_some(part_ent) else {
                        continue;
                    };
                    if modifier_has_marker(mod_ent, templ_ref, &queries.pain_infliction_query) {
                        add_part_or_body_modifier_sum(
                            &mut locals.pain_infliction_by_part,
                            &mut locals.pain_infliction_by_body,
                            target_part_ent,
                            value.0,
                            &queries.bodies_query,
                            &queries.parts_query,
                            &queries.user_body_instances_query,
                        );
                    }
                    if modifier_has_marker(mod_ent, templ_ref, &queries.pain_sensitivity_query) {
                        add_part_or_body_modifier_sum(
                            &mut locals.pain_sensitivity_by_part,
                            &mut locals.pain_sensitivity_by_body,
                            target_part_ent,
                            value.0,
                            &queries.bodies_query,
                            &queries.parts_query,
                            &queries.user_body_instances_query,
                        );
                    }
                    if modifier_has_marker(mod_ent, templ_ref, &queries.bleed_query) {
                        add_modifier_sum(
                            &mut locals.bleed_mod_sum,
                            target_part_ent,
                            value.0,
                            &queries.bodies_query,
                            &queries.parts_query,
                            &queries.user_body_instances_query,
                        );
                    }
                    if modifier_has_marker(mod_ent, templ_ref, &queries.blood_capacity_query) {
                        add_modifier_sum(
                            &mut locals.blood_capacity_mod_sum,
                            target_part_ent,
                            value.0,
                            &queries.bodies_query,
                            &queries.parts_query,
                            &queries.user_body_instances_query,
                        );
                    }
                    if modifier_has_marker(mod_ent, templ_ref, &queries.consciousness_query) {
                        add_modifier_sum(
                            &mut locals.consciousness_mod_sum,
                            target_part_ent,
                            value.0,
                            &queries.bodies_query,
                            &queries.parts_query,
                            &queries.user_body_instances_query,
                        );
                    }
                    if modifier_has_marker(mod_ent, templ_ref, &queries.vision_query) {
                        add_modifier_sum(
                            &mut locals.vision_mod_sum,
                            target_part_ent,
                            value.0,
                            &queries.bodies_query,
                            &queries.parts_query,
                            &queries.user_body_instances_query,
                        );
                    }
                }

                let max_hp = max_hp_by_part.0.get(&part_ent).copied().unwrap_or(0.0).max(0.0);
                let mut current_hp = max_hp;
                if let Ok(damage) = queries.damage_query.get_mut(part_ent) {
                    current_hp = (max_hp - damage.total).max(0.0);
                }
                let current_hp = current_hp.clamp(0.0, max_hp);
                let damage_amount = (max_hp - current_hp).max(0.0);
                let pain_ratio = if max_hp > 0.0 {
                    damage_amount / max_hp
                } else {
                    0.0
                };

                let pain_infliction = locals.pain_infliction_by_part
                    .get(&part_ent)
                    .copied()
                    .unwrap_or(0.0)
                    + locals.pain_infliction_by_body
                        .get(&body)
                        .copied()
                        .unwrap_or(0.0);

                let pain_sensitivity_part = locals.pain_sensitivity_by_part
                    .get(&part_ent)
                    .copied()
                    .unwrap_or(1.0);
                let pain_sensitivity_body = locals.pain_sensitivity_by_body
                    .get(&body)
                    .copied()
                    .unwrap_or(1.0);
                let pain_sensitivity_mult = pain_sensitivity_part * pain_sensitivity_body;
                let pain_mult = (1.0 + pain_infliction) * pain_sensitivity_mult;
                let part_pain = pain_ratio * pain_mult;

                total_max_hp += max_hp;
                total_hp += current_hp;
                total_pain += part_pain;
            }
        }
        let bleed_rate = bleed_rate + locals.bleed_mod_sum.get(&body).copied().unwrap_or(0.0);
        let blood_capacity = (blood_capacity + locals.blood_capacity_mod_sum.get(&body).copied().unwrap_or(0.0)).max(0.0);
        if parts_count > 0 && total_max_hp <= 0.0 {
            file_log(
                BODY_HP_SYSTEM,
                "host",
                &format!(
                    "body_zero_total_hp body={body:?} being={:?} templ_body={:?} parts_count={parts_count} total_hp={total_hp:.3} blood_capacity={blood_capacity:.3}",
                    body_of.being,
                    body_templ_ref.map(|templ| templ.0),
                ),
            );
        }
        let base_consciousness = (1.0 + locals.consciousness_mod_sum.get(&body).copied().unwrap_or(0.0)).max(0.0);
        let vision = locals.vision_mod_sum.get(&body).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let pain = (total_pain / 10.0).max(0.0);
        let delta = time.delta_secs();

        let bloodless_nonbleeding = blood_capacity == 0.0 && bleed_rate == 0.0;
        let mut curr_blood = match queries.body_sums_query.get_mut(body) {
            Ok(health) => health.blood,
            Err(_) => blood_capacity,
        };

        if blood_capacity < 0.0 {
            curr_blood = 0.0;
        } else if blood_capacity == 0.0 {
            curr_blood = 0.0;
        } else {
            if curr_blood.is_nan() {
                curr_blood = blood_capacity;
            }
            curr_blood = (curr_blood - bleed_rate * delta * 0.0).clamp(0.0, blood_capacity);//TODO hacer q solo bleedee si tiene una herida, no porque sí
        }

        let curr_blood_ratio = if blood_capacity > 0.0 {
            curr_blood / blood_capacity
        } else if bloodless_nonbleeding {
            1.0
        } else {
            0.0
        };
        let consciousness = (base_consciousness * curr_blood_ratio).clamp(0.0, 1.0);

        let has_initialized_vitals = total_max_hp > 0.0 || blood_capacity > 0.0 || bleed_rate > 0.0;
        let dead = has_initialized_vitals && !bloodless_nonbleeding && (consciousness < 0.01 || curr_blood <= 0.0);
        if dead {
            let being_ent = body_of.being;
            warn!(
                target: BODY_HP_SYSTEM,
                "Inserting Dead on being {:?} from body {:?}: blood={:.3} blood_capacity={:.3} total_max_hp={:.3} initialized_vitals={}",
                being_ent,
                body,
                curr_blood,
                blood_capacity,
                total_max_hp,
                has_initialized_vitals,
            );
            cmd.entity(being_ent).try_insert_if_new(Dead);
        }

        let Ok(mut health) = queries.body_sums_query.get_mut(body) else {
            continue;
        };
        health.total_hp = total_max_hp;
        health.current_hp = total_hp;
        health.blood = curr_blood;
        health.blood_capacity = blood_capacity;
        health.bleed_rate = bleed_rate;
        health.consciousness = consciousness;
        health.pain = pain;
        health.vision = vision;


    }
}
#[inline]
fn add_modifier_sum(
    sums: &mut EntityHashMap<f32>,
    target: Entity,
    value: f32,
    bodies_query: &Query<(Entity, &'static BodyOf), Without<Templ>>,
    parts_query: &Query<(Entity, &'static ChildOf), (With<BodypartChildOfBodypart>, Without<Templ>, Without<Missing>)>,
    user_body_instances_query: &Query<&UserBodypartInstances>,
) {
    if bodies_query.get(target).is_ok() {
        *sums.entry(target).or_insert(0.0) += value;
        return;
    }
    if let Ok((_, child)) = parts_query.get(target) {
        *sums.entry(child.parent()).or_insert(0.0) += value;
        return;
    }

    let Ok(user_body_instances) = user_body_instances_query.get(target) else {
        return;
    };
    for part_ent in user_body_instances.0.iter() {
        let Ok((_, child)) = parts_query.get(*part_ent) else {
            continue;
        };
        *sums.entry(child.parent()).or_insert(0.0) += value;
    }
}

#[inline]
fn add_part_or_body_modifier_sum(
    part_sums: &mut EntityHashMap<f32>,
    body_sums: &mut EntityHashMap<f32>,
    target: Entity,
    value: f32,
    bodies_query: &Query<(Entity, &'static BodyOf), Without<Templ>>,
    parts_query: &Query<(Entity, &'static ChildOf), (With<BodypartChildOfBodypart>, Without<Templ>, Without<Missing>)>,
    user_body_instances_query: &Query<&UserBodypartInstances>,
) {
    if bodies_query.get(target).is_ok() {
        *body_sums.entry(target).or_insert(0.0) += value;
        return;
    }
    if parts_query.get(target).is_ok() {
        *part_sums.entry(target).or_insert(0.0) += value;
        return;
    }

    let Ok(user_body_instances) = user_body_instances_query.get(target) else {
        return;
    };
    for part_ent in user_body_instances.0.iter() {
        *part_sums.entry(*part_ent).or_insert(0.0) += value;
    }
}
#[allow(unused_parens)]
pub fn apply_bodypart_hp_regen(
    time: Res<Time>,
    max_hp_by_part: Res<BodypartMaxHpMap>,
    mut parts_query: Query<(Entity, Option<&mut AccuDamage>,), (With<BodypartChildOfBodypart>, Without<Templ>, Without<Missing>)>,
    applied_mods_query: Query<&AppliedModifiers>,
    hp_regen_query: Query<(), (With<HitpointRegenRate>,)>,

    templ_enti_refs_query: Query<&'static TemplEntiRef>,
    curr_values_query: Query<&'static CurrEffectiveValue>,
    mut effects: Local<EntityHashSet>,
) {
    let delta = time.delta_secs();
    if delta <= 0.0 {
        return;
    }

    for (part_ent, damage) in parts_query.iter_mut() {
        let part_templ_ref = templ_enti_refs_query.get(part_ent).ok();
        effects.clear();
        collect_applied_modifier_entities(
            &mut effects,
            part_ent,
            part_templ_ref,
            &applied_mods_query,
        );
        let max_hp = max_hp_by_part.0.get(&part_ent).copied().unwrap_or(0.0).max(0.0);
        if max_hp <= 0.0 {
            continue;
        }

        let mut regen_rate = 0.0;
        for mod_ent in effects.iter() {
            let templ_ref = templ_enti_refs_query.get(*mod_ent).ok();
            let has_regen_marker = hp_regen_query.get(*mod_ent).is_ok()
                || templ_ref.is_some_and(|templ_ref| hp_regen_query.get(templ_ref.0).is_ok());
            if !has_regen_marker {
                continue;
            }
            let Some(value) = resolve_modifier_component(*mod_ent, templ_ref, &curr_values_query) else {
                continue;
            };
            regen_rate += value.0;
        }
        if regen_rate == 0.0 {
            continue;
        }

        let delta_hp = regen_rate * delta;
        if delta_hp <= 0.0 {
            continue;
        }
        let Some(mut damage) = damage else {
            continue;
        };
        damage.heal(delta_hp);
    }
}

/// Apply pain-based slowdown to movement speeds via the modifier system.
/// Slowdown multiplier is 1.0 - pain, so pain = 0 means no slowdown, pain = 1.0 means full stop.
#[allow(unused_parens, )]
pub fn ensure_pain_slowdown_modifiers(
    mut cmd: Commands,
    mut body_health_query: Query<(Entity, &BodyOf, &BodySums), (Changed<BodySums>,)>,
    mut slowdown_mod_query: Query<(Entity, &ModifierTarget, &mut BaseValue), (With<PainSlowdown>,)>,
    mut slowdown_by_being: Local<EntityHashMap<(Entity, f32)>>,
    mut kept_slowdown_ent_by_being: Local<EntityHashMap<Entity>>,
) {
    slowdown_by_being.clear();
    kept_slowdown_ent_by_being.clear();

    for (body_ent, body_of, body_health) in body_health_query.iter_mut() {
        let pain = body_health.pain.clamp(0.0, 1.0);
        let pain_multiplier = (1.0 - pain ).clamp(0.0, 1.0);
        let entry = slowdown_by_being
            .entry(body_of.being)
            .or_insert((body_ent, pain_multiplier));
        if pain_multiplier < entry.1 {
            *entry = (body_ent, pain_multiplier);
        }
    }
    if slowdown_by_being.is_empty() {
        return;
    }

    for (mod_ent, target, mut base_value) in slowdown_mod_query.iter_mut() {
        let Some((_, pain_multiplier)) = slowdown_by_being.get(&target.0) else {
            continue;
        };
        if kept_slowdown_ent_by_being.insert(target.0, mod_ent).is_some() {
            cmd.entity(mod_ent).try_despawn();
            continue;
        }
        base_value.0 = *pain_multiplier;
        cmd.entity(mod_ent).try_insert(ChildOf(target.0));
    }

    for (being_ent, (_, pain_multiplier)) in slowdown_by_being.iter() {
        if kept_slowdown_ent_by_being.contains_key(being_ent) {
            continue;
        }
        cmd.spawn((
            WalkStrength,
            SwimStrength,
            FlyStrength,
            ModifierTarget(*being_ent),
            BaseValue(*pain_multiplier),
            CurrEffectiveValue(*pain_multiplier),
            ApplyMode::Mul,
            PainSlowdown,
            ChildOf(*being_ent),
        ));
    }
}
