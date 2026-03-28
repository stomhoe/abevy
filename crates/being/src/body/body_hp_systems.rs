use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::common_components::StrId;
use common::log_targets::BODY_HP_SYSTEM;
use game_common::game_common_components::{Templ, TemplEntiRef};
use modifier_shared::resolve_modifier_component;
use modifier_shared::{modifier_components::*, modifier_types::*};
use ::being_shared::*;
use game_common::game_common_components::Dead;

use crate::body::{body_tree_components::*, };



#[allow(unused_parens)]
pub fn apply_body_damage(
    mut cmd: Commands,
    time: Res<Time>,
    mut reader: MessageReader<IncomingDamage>,
    bodies_query: Query<&BodypartChildrenBodyparts, (With<BodyOf>, Without<Templ>)>,
    parts_query: Query<
        (Option<&BodypartCoverageWeight>, Option<&Missing>),
        (With<BodypartChildOfBodypart>, Without<Templ>),
    >,
    mut damage_query: Query<&mut BodypartDamage, (Without<Templ>)>,
    mut weighted_parts: Local<Vec<(Entity, u16)>>,
) {
    for damage_msg in reader.read() {
        weighted_parts.clear();
        let mut total_weight: u32 = 0;

        let body_ent = damage_msg.body;
        let damage_amount = damage_msg.amount;

        let Ok(parts) = bodies_query.get(body_ent) else {
            continue;
        };

        for part_ent in parts.iter() {
            let Ok((weight_opt, missing)) = parts_query.get(part_ent) else {
                continue;
            };
            if missing.is_some() {
                continue;
            }
            let weight = weight_opt.map(|w| w.0).unwrap_or(1).max(1);
            total_weight += weight as u32;
            weighted_parts.push((part_ent, weight));
        }

        let Some(total_weight) = (total_weight > 0).then_some(total_weight) else { continue };

        let seed = (body_ent.to_bits() as u64)
            ^ time.elapsed_secs_f64().to_bits()
            ^ (damage_amount.to_bits() as u64);
        let mut roll = (seed % total_weight as u64) as u32;

        let mut selected = None;
        for (part_ent, weight) in weighted_parts.iter() {
            if roll < *weight as u32 {
                selected = Some(*part_ent);
                break;
            }
            roll -= *weight as u32;
        }

        if let Some(part_ent) = selected {
            if let Ok(mut part_damage) = damage_query.get_mut(part_ent) {
                part_damage.0 += damage_amount.max(0.0);
            } else {
                cmd.entity(part_ent)
                    .try_insert(BodypartDamage(damage_amount.max(0.0)));
            }
        }
    }
}

#[allow(unused_parens)]
pub fn sync_bodypart_missing(
    mut cmd: Commands,
    time: Res<Time>,
    parts_damage_query: Query<(Entity, Option<&BodypartDamage>), (
        Or<(Changed<BodypartDamage>, Changed<BodypartChildOfBodypart>)>,
        With<BodypartChildOfBodypart>, Without<Templ>)>,
    applied_mods_query: Query<&AppliedModifiers>,
    templ_enti_refs_query: Query<&TemplEntiRef>,
    hp_mods_query: Query<(), (With<HitpointsCapacity>,)>,
    curr_values_query: Query<&CurrEffectiveValue>,
    base_values_query: Query<&BaseValue>,
    mut missing_timers: Local<EntityHashMap<Timer>>,
) {
    let mut seen_parts = EntityHashSet::default();

    for (part_ent, damage) in parts_damage_query {
        seen_parts.insert(part_ent);
        let mut max_hp = 0.0;
        if let Ok(applied_mods) = applied_mods_query.get(part_ent) {
            for mod_ent in applied_mods.iter() {
                let Ok(()) = hp_mods_query.get(mod_ent) else {
                    continue;
                };

                let templ_ref = templ_enti_refs_query.get(mod_ent).ok();
                let base_value = resolve_modifier_component(mod_ent, templ_ref, &base_values_query);
                let current_value = resolve_modifier_component(mod_ent, templ_ref, &curr_values_query);
                let value = current_value
                    .map(|v| v.0)
                    .or_else(|| base_value.map(|v| v.0))
                    .unwrap_or(0.0);
                max_hp += value;
            }
        }
        if let Ok(part_templ_ref) = templ_enti_refs_query.get(part_ent) {
            if let Ok(templ_modis) = applied_mods_query.get(part_templ_ref.0) {
                for mod_ent in templ_modis.iter() {
                    let Ok(()) = hp_mods_query.get(mod_ent) else {
                        continue;
                    };
                    let templ_ref = templ_enti_refs_query.get(mod_ent).ok();
                    let base_value = resolve_modifier_component(mod_ent, templ_ref, &base_values_query);
                    let current_value = resolve_modifier_component(mod_ent, templ_ref, &curr_values_query);
                    let value = current_value
                        .map(|v| v.0)
                        .or_else(|| base_value.map(|v| v.0))
                        .unwrap_or(0.0);
                    max_hp += value;
                }
            }
        }

        if max_hp <= 0.0 {
            missing_timers.remove(&part_ent);
            error_once!(
                target: BODY_HP_SYSTEM,
                "Skipping Missing update for part {:?}: unresolved max_hp={:.3} (likely pre-init or missing hp modifiers)",
                part_ent,
                max_hp,
            );
            continue;
        }
        let current_hp = max_hp - damage.cloned().unwrap_or_default().0;
        let current_hp = current_hp.clamp(0.0, max_hp);

        if current_hp <= 0.0 {
            let timer = missing_timers
                .entry(part_ent)
                .or_insert_with(|| Timer::from_seconds(0.2, TimerMode::Once));
            timer.tick(time.delta());
            if timer.is_finished() {
                cmd.entity(part_ent).try_insert_if_new(Missing);
            } else {
                cmd.entity(part_ent).try_remove::<Missing>();
            }
        } else {
            missing_timers.remove(&part_ent);
            cmd.entity(part_ent).try_remove::<Missing>();
        }
    }

    missing_timers.retain(|part_ent, _| seen_parts.contains(part_ent));
}

#[derive(SystemParam)]
pub struct BodyHealthQueryParams<'w, 's> {
    bodies_query: Query<'w, 's, (Entity, &'static BodyOf), Without<Templ>>,
    parts_query: Query<'w, 's, (Entity, &'static ChildOf, Option<&'static Vital>, Has<Missing>), (With<BodypartChildOfBodypart>, Without<Templ>)>,
    hp_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<HitpointsCapacity>,)>,
    pain_infliction_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<PainInfliction>, Without<Templ>)>,
    pain_sensitivity_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<PainSensitivity>, Without<Templ>)>,
    bleed_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<BleedRate>, Without<Templ>)>,
    blood_capacity_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<BloodCapacity>, Without<Templ>)>,
    hp_regen_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<HitpointRegenRate>, Without<Templ>)>,
    consciousness_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<Consciousness>, Without<Templ>)>,
    vision_mods_query: Query<'w, 's, (Entity, &'static ModifierTarget), (With<Vision>, Without<Templ>)>,
    templ_enti_refs_query: Query<'w, 's, &'static TemplEntiRef>,
    part_applied_mods_query: Query<'w, 's, &'static AppliedModifiers>,
    damage_query: Query<'w, 's, (Entity, Option<&'static mut BodypartDamage>), >,
    body_health_query: Query<'w, 's, Option<&'static mut BodySums>>,
    curr_values_query: Query<'w, 's, &'static CurrEffectiveValue>,
    base_values_query: Query<'w, 's, &'static BaseValue>,
}

#[derive(SystemParam)]
pub struct BodyHealthLocalParams<'s> {
    max_hp_by_part: Local<'s, EntityHashMap<f32>>,
    body_max_hp_bonus: Local<'s, EntityHashMap<f32>>,
    pain_infliction_by_part: Local<'s, EntityHashMap<f32>>,
    pain_sensitivity_by_part: Local<'s, EntityHashMap<f32>>,
    pain_infliction_by_body: Local<'s, EntityHashMap<f32>>,
    pain_sensitivity_by_body: Local<'s, EntityHashMap<f32>>,
    bleed_mod_sum: Local<'s, EntityHashMap<f32>>,
    blood_capacity_mod_sum: Local<'s, EntityHashMap<f32>>,
    hp_regen_mod_sum: Local<'s, EntityHashMap<f32>>,
    consciousness_mod_sum: Local<'s, EntityHashMap<f32>>,
    vision_mod_sum: Local<'s, EntityHashMap<f32>>,
    body_regen_map: Local<'s, EntityHashMap<(f32, f32)>>,
}

#[allow(unused_parens)]
pub fn update_body_health_from_parts(
    mut cmd: Commands,
    time: Res<Time>,
    mut queries: BodyHealthQueryParams,
    mut locals: BodyHealthLocalParams,
) {
    locals.max_hp_by_part.clear();
    locals.body_max_hp_bonus.clear();
    locals.pain_infliction_by_part.clear();
    locals.pain_sensitivity_by_part.clear();
    locals.pain_infliction_by_body.clear();
    locals.pain_sensitivity_by_body.clear();
    let mut instance_part_by_source_part: EntityHashMap<Entity> = EntityHashMap::default();
    queries.parts_query.iter().for_each(|(ent, _, _, _)| {
        let Ok(part_templ_ref) = queries.templ_enti_refs_query.get(ent) else {
            return;
        };
        instance_part_by_source_part.insert(part_templ_ref.0, ent);
    });

    for (mod_ent, target) in queries.hp_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let base_value = resolve_modifier_component(mod_ent, templ_ref, &queries.base_values_query);
        let current_value = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query);
        let value = current_value
            .map(|v| v.0)
            .or_else(|| base_value.map(|v| v.0))
            .unwrap_or(0.0);

        if queries.parts_query.get(target.0).is_ok() {
            *locals.max_hp_by_part.entry(target.0).or_insert(0.0) += value;
        } else if let Some(part_ent) = instance_part_by_source_part.get(&target.0)
        {
            *locals.max_hp_by_part.entry(*part_ent).or_insert(0.0) += value;
        } else if queries.bodies_query.get(target.0).is_ok() {
            *locals.body_max_hp_bonus.entry(target.0).or_insert(0.0) += value;
        }
    }

    for (mod_ent, target) in queries.pain_infliction_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        if queries.parts_query.get(target.0).is_ok() {
            *locals.pain_infliction_by_part.entry(target.0).or_insert(0.0) += value.0;
        } else if let Some(part_ent) = instance_part_by_source_part.get(&target.0) {
            *locals.pain_infliction_by_part.entry(*part_ent).or_insert(0.0) += value.0;
        } else if queries.bodies_query.get(target.0).is_ok() {
            *locals.pain_infliction_by_body.entry(target.0).or_insert(0.0) += value.0;
        }
    }
    for (mod_ent, target) in queries.pain_sensitivity_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        if queries.parts_query.get(target.0).is_ok() {
            *locals.pain_sensitivity_by_part.entry(target.0).or_insert(0.0) += value.0;
        } else if let Some(part_ent) = instance_part_by_source_part.get(&target.0) {
            *locals.pain_sensitivity_by_part.entry(*part_ent).or_insert(0.0) += value.0;
        } else if queries.bodies_query.get(target.0).is_ok() {
            *locals.pain_sensitivity_by_body.entry(target.0).or_insert(0.0) += value.0;
        }
    }

    locals.bleed_mod_sum.clear();
    locals.blood_capacity_mod_sum.clear();
    locals.hp_regen_mod_sum.clear();
    locals.consciousness_mod_sum.clear();
    locals.vision_mod_sum.clear();

    for (mod_ent, target) in queries.bleed_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut locals.bleed_mod_sum, target.0, value.0, &queries.bodies_query, &queries.parts_query);
    }
    for (mod_ent, target) in queries.blood_capacity_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut locals.blood_capacity_mod_sum, target.0, value.0, &queries.bodies_query, &queries.parts_query);
    }
    for (mod_ent, target) in queries.hp_regen_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut locals.hp_regen_mod_sum, target.0, value.0, &queries.bodies_query, &queries.parts_query);
    }
    for (mod_ent, target) in queries.consciousness_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut locals.consciousness_mod_sum, target.0, value.0, &queries.bodies_query, &queries.parts_query);
    }
    for (mod_ent, target) in queries.vision_mods_query.iter() {
        let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
        let Some(value) = resolve_modifier_component(mod_ent, templ_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut locals.vision_mod_sum, target.0, value.0, &queries.bodies_query, &queries.parts_query);
    }

    let delta = time.delta_secs();

    for (body, body_of) in queries.bodies_query.iter() {
        let mut total_max_hp = 0.0;
        let mut total_hp = 0.0;
        let mut total_pain = 0.0;
        let bleed_rate = 0.0;
        let blood_capacity = 0.0;
        let regen_rate = 0.0;
        let mut vital_missing = false;

        for (part_ent, body_of, vital, missing) in queries.parts_query.iter() {
            if body_of.parent() != body {
                continue;
            }
            let part_templ_ref = queries.templ_enti_refs_query.get(part_ent).ok();

            let max_hp = locals.max_hp_by_part
                .get(&part_ent)
                .copied()
                .unwrap_or(0.0)
                .max(0.0);
            let mut current_hp = max_hp;
            if let Ok((_ent, damage)) = queries.damage_query.get_mut(part_ent) {
                if let Some(damage) = damage {
                    current_hp = (max_hp - damage.0).max(0.0);
                }
            }
            let current_hp = current_hp.clamp(0.0, max_hp);
            let damage_amount = (max_hp - current_hp).max(0.0);
            let pain_ratio = if max_hp > 0.0 {
                damage_amount / max_hp
            } else {
                0.0
            };
            let source_part_pain_infliction = part_templ_ref.and_then(|part_templ_ref| {
                let Ok(applied_mods) = queries.part_applied_mods_query.get(part_templ_ref.0) else {
                    return None;
                };
                let mut total = 0.0;
                for mod_ent in applied_mods.iter() {
                    let Ok((mod_ent, _)) = queries.pain_infliction_mods_query.get(mod_ent) else {
                        continue;
                    };
                    let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
                    let Some(value) = resolve_modifier_component(
                        mod_ent,
                        templ_ref,
                        &queries.curr_values_query,
                    ) else {
                        continue;
                    };
                    total += value.0;
                }
                Some(total)
            }).unwrap_or(0.0);

            let pain_infliction = locals.pain_infliction_by_part
                .get(&part_ent)
                .copied()
                .unwrap_or(0.0)
                + source_part_pain_infliction
                + locals.pain_infliction_by_body
                    .get(&body)
                    .copied()
                    .unwrap_or(0.0);
            let source_part_pain_sensitivity = part_templ_ref.and_then(|part_templ_ref| {
                let Ok(applied_mods) = queries.part_applied_mods_query.get(part_templ_ref.0) else {
                    return None;
                };
                let mut total = 0.0;
                let mut has_any = false;
                for mod_ent in applied_mods.iter() {
                    let Ok((mod_ent, _)) = queries.pain_sensitivity_mods_query.get(mod_ent) else {
                        continue;
                    };
                    let templ_ref = queries.templ_enti_refs_query.get(mod_ent).ok();
                    let Some(value) = resolve_modifier_component(
                        mod_ent,
                        templ_ref,
                        &queries.curr_values_query,
                    ) else {
                        continue;
                    };
                    total += value.0;
                    has_any = true;
                }
                if has_any { Some(total) } else { None }
            }).unwrap_or(1.0);

            let pain_sensitivity_part = locals.pain_sensitivity_by_part
                .get(&part_ent)
                .copied()
                .unwrap_or(0.0)
                + source_part_pain_sensitivity;
            let pain_sensitivity_body = locals.pain_sensitivity_by_body
                .get(&body)
                .copied()
                .unwrap_or(1.0);
            let pain_sensitivity_mult = pain_sensitivity_part * pain_sensitivity_body;
            let pain_mult = (1.0 + pain_infliction) * pain_sensitivity_mult;
            let part_pain = pain_ratio * pain_mult;

            if !missing {
                total_max_hp += max_hp;
                total_hp += current_hp;
                total_pain += part_pain;
            } else if vital.is_some() {
                vital_missing = true;
            }
        }
        let total_max_hp = (total_max_hp + locals.body_max_hp_bonus.get(&body).copied().unwrap_or(0.0)).max(0.0);
        let bleed_rate = bleed_rate + locals.bleed_mod_sum.get(&body).copied().unwrap_or(0.0);
        let blood_capacity = (blood_capacity + locals.blood_capacity_mod_sum.get(&body).copied().unwrap_or(0.0)).max(0.0);
        let regen_rate = regen_rate + locals.hp_regen_mod_sum.get(&body).copied().unwrap_or(0.0);
        let base_consciousness = (1.0 + locals.consciousness_mod_sum.get(&body).copied().unwrap_or(0.0)).max(0.0);
        let vision = locals.vision_mod_sum.get(&body).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let pain = total_pain.max(0.0);

        let mut blood = match queries.body_health_query.get_mut(body) {
            Ok(Some(ref mut health)) => health.blood,
            Ok(None) => blood_capacity,
            Err(_) => blood_capacity,
        };

        if blood_capacity <= 0.0 {
            blood = 0.0;
        } else {
            blood = (blood - bleed_rate * delta).clamp(0.0, blood_capacity);
        }

        let blood_factor = if blood_capacity > 0.0 {
            blood / blood_capacity
        } else {
            0.0
        };
        let mut consciousness = (base_consciousness * blood_factor).clamp(0.0, 1.0);
        if vital_missing {
            consciousness = 0.0;
        }

        let has_initialized_vitals = total_max_hp > 0.0 || blood_capacity > 0.0 || vital_missing;
        let dead = if has_initialized_vitals {
            vital_missing || blood <= 0.0
        } else {
            false
        };

        match queries.body_health_query.get_mut(body) {
            Ok(Some(mut health)) => {
                health.total_hp = total_max_hp;
                health.current_hp = total_hp;
                health.blood = blood;
                health.blood_capacity = blood_capacity;
                health.bleed_rate = bleed_rate;
                health.consciousness = consciousness;
                health.pain = pain;
                health.vision = vision;
            }
            Ok(None) => {
                cmd.entity(body).try_insert(BodySums {
                    total_hp: total_max_hp,
                    current_hp: total_hp,
                    blood,
                    blood_capacity,
                    bleed_rate,
                    consciousness,
                    pain,
                    vision,
                    manip_dex: 0.0,
                    manip_str: 0.0,
                });
            }
            Err(_) => {}
        }

        let being_ent = body_of.being;
        if dead {
            debug!(
                target: BODY_HP_SYSTEM,
                "Inserting Dead on being {:?} from body {:?}: vital_missing={} blood={:.3} blood_capacity={:.3} total_max_hp={:.3} initialized_vitals={}",
                being_ent,
                body,
                vital_missing,
                blood,
                blood_capacity,
                total_max_hp,
                has_initialized_vitals,
            );
            cmd.entity(being_ent).try_insert_if_new(Dead);
        } else {
            debug!(
                target: BODY_HP_SYSTEM,
                "Removing Dead from being {:?} from body {:?}: vital_missing={} blood={:.3} blood_capacity={:.3} total_max_hp={:.3} initialized_vitals={}",
                being_ent,
                body,
                vital_missing,
                blood,
                blood_capacity,
                total_max_hp,
                has_initialized_vitals,
            );
            cmd.entity(being_ent).try_remove::<Dead>();
        }

        locals.body_regen_map.insert(body, (regen_rate, total_max_hp));
    }

    for (part_ent, damage) in queries.damage_query.iter_mut() {
        let Ok(part_of) = queries.parts_query.get(part_ent) else {
            continue;
        };
        let Some((regen_rate, total_max_hp)) = locals.body_regen_map
            .get(&part_of.1.parent())
        else {
            continue;
        };
        if *total_max_hp <= 0.0 {
            continue;
        }

        let max_hp = locals.max_hp_by_part
            .get(&part_ent)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        if max_hp <= 0.0 {
            continue;
        }

        let delta_hp = regen_rate * delta * (max_hp / *total_max_hp);
        if delta_hp == 0.0 {
            continue;
        }

        if let Some(mut damage) = damage {
            damage.0 = (damage.0 - delta_hp).max(0.0);
        }
    }
}
#[inline]
fn add_modifier_sum(
    sums: &mut EntityHashMap<f32>,
    target: Entity,
    value: f32,
    bodies_query: &Query<(Entity, &'static BodyOf), Without<Templ>>,
    parts_query: &Query<(Entity, &'static ChildOf, Option<&'static Vital>, Has<Missing>), (With<BodypartChildOfBodypart>, Without<Templ>)>,
) {
    let body = if bodies_query.get(target).is_ok() {
        Some(target)
    } else {
        parts_query.get(target).ok().map(|(_, child, _, _)| child.parent())
    };

    if let Some(body) = body {
        *sums.entry(body).or_insert(0.0) += value;
    }
}

/// Apply pain-based slowdown to movement speeds via the modifier system.
/// Slowdown multiplier is 1.0 - pain, so pain = 0 means no slowdown, pain = 1.0 means full stop.
pub fn apply_pain_slowdown(
    mut cmd: Commands,
    mut body_health_query: Query<(Entity, &BodyOf, &BodySums), (Changed<BodySums>,)>,
    mut slowdown_mod_query: Query<(Entity, &ModifierTarget, &mut BaseValue), (With<PainSlowdown>,)>,
    mut slowdown_by_being: Local<EntityHashMap<(Entity, f32)>>,
    mut kept_slowdown_ent_by_being: Local<EntityHashMap<Entity>>,
    mut seen_beings: Local<EntityHashSet>,
    mut extra_slowdown_ents: Local<Vec<Entity>>,
) {
    slowdown_by_being.clear();
    kept_slowdown_ent_by_being.clear();
    seen_beings.clear();
    extra_slowdown_ents.clear();

    for (body_ent, body_of, body_health) in body_health_query.iter_mut() {
        let pain_multiplier = (1.0 - body_health.pain).clamp(0.0, 1.0);
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
        if !seen_beings.insert(target.0) {
            extra_slowdown_ents.push(mod_ent);
            continue;
        }
        base_value.0 = *pain_multiplier;
        cmd.entity(mod_ent).insert(ChildOf(target.0));
        kept_slowdown_ent_by_being.insert(target.0, mod_ent);
    }

    for extra_ent in extra_slowdown_ents.iter() {
        cmd.entity(*extra_ent).try_despawn();
    }

    for (being_ent, (_, pain_multiplier)) in slowdown_by_being.iter() {
        if kept_slowdown_ent_by_being.contains_key(being_ent) {
            continue;
        }
        cmd.spawn((
            WalkSpeed,
            SwimSpeed,
            FlySpeed,
            ModifierTarget(*being_ent),
            BaseValue(*pain_multiplier),
            CurrEffectiveValue(*pain_multiplier),
            ApplyMode::Mul,
            PainSlowdown,
            ModifierTags::default(),
            StrId::trunc("pain_slowdown"),
            ChildOf(*being_ent),
        ));
    }
}
