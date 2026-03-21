use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::common_components::StrId;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use modifier_shared::{modifier_has_marker, resolve_modifier_component};
use modifier_shared::{modifier_components::*, modifier_types::*};
use ::being_shared::*;
use game_common::game_common_components::Dead;

use crate::body::{body_tree_components::*, };

#[derive(Default, Clone)]
struct BodyAggregates {
    total_max_hp: f32,
    total_hp: f32,
    total_pain: f32,
    bleed_rate: f32,
    blood_capacity: f32,
    regen_rate: f32,
    vital_missing: bool,
}

#[derive(SystemParam)]
pub struct BodyHealthQueryParams<'w, 's> {
    bodies_query: Query<'w, 's, Entity, With<BodyOf>>,
    parts_query: Query<'w, 's, (Entity, &'static ChildOf, Option<&'static Vital>, Option<&'static EntityZeroRef>, Has<Missing>), (With<BodypartChildOfBodypart>, Without<EntityZero>)>,
    modifiers_query: Query<'w, 's, (Entity, &'static ModifierTarget, Option<&'static EntityZeroRef>), Without<EntityZero>>,
    part_applied_mods_query: Query<'w, 's, &'static AppliedModifiers>,
    damage_query: Query<'w, 's, (Entity, Option<&'static mut BodypartDamage>), >,
    body_health_query: Query<'w, 's, Option<&'static mut BodySums>>,
    dead_query: Query<'w, 's, Option<&'static mut Dead>>,
    curr_values_query: Query<'w, 's, &'static CurrEffectiveValue>,
    base_values_query: Query<'w, 's, &'static BaseValue>,
    modifier_tags_query: Query<'w, 's, &'static ModifierTags>,
    hitpoints_capacity_markers: Query<'w, 's, (), With<HitpointsCapacity>>,
    bleed_markers: Query<'w, 's, (), With<BleedRate>>,
    blood_capacity_markers: Query<'w, 's, (), With<BloodCapacity>>,
    hp_regen_markers: Query<'w, 's, (), With<HitpointRegenRate>>,
    consciousness_markers: Query<'w, 's, (), With<Consciousness>>,
    pain_infliction_markers: Query<'w, 's, (), With<PainInfliction>>,
    pain_sensitivity_markers: Query<'w, 's, (), With<PainSensitivity>>,
    vision_markers: Query<'w, 's, (), With<Vision>>,
}

#[derive(SystemParam)]
pub struct BodyHealthLocalParams<'s> {
    bodies: Local<'s, EntityHashSet>,
    parts: Local<'s, EntityHashSet>,
    part_to_body: Local<'s, EntityHashMap<Entity>>,
    aggregates: Local<'s, EntityHashMap<BodyAggregates>>,
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

#[inline]
fn add_modifier_sum(
    sums: &mut EntityHashMap<f32>,
    target: Entity,
    value: f32,
    bodies: &EntityHashSet,
    part_to_body: &EntityHashMap<Entity>,
) {
    let body = if bodies.contains(&target) {
        Some(target)
    } else {
        part_to_body.get(&target).copied()
    };

    if let Some(body) = body {
        *sums.entry(body).or_insert(0.0) += value;
    }
}

#[allow(unused_parens)]
pub fn apply_body_damage(
    mut cmd: Commands,
    time: Res<Time>,
    mut reader: MessageReader<IncomingDamage>,
    bodies_query: Query<&BodypartChildrenBodyparts, (With<BodyOf>, Without<EntityZero>)>,
    parts_query: Query<
        (Option<&BodypartCoverageWeight>, Option<&Missing>),
        (With<BodypartChildOfBodypart>, Without<EntityZero>),
    >,
    mut damage_query: Query<&mut BodypartDamage, (Without<EntityZero>)>,
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
    parts_query: Query<(Entity, Option<&EntityZeroRef>), (With<BodypartChildOfBodypart>, Without<EntityZero>)>,
    part_applied_mods_query: Query<&AppliedModifiers>,
    hp_mods_query: Query<
        (
            Entity,
            &ModifierTarget,
            Option<&EntityZeroRef>,
        ),
        Without<EntityZero>,
    >,
    curr_values_query: Query<&CurrEffectiveValue>,
    base_values_query: Query<&BaseValue>,
    modifier_tags_query: Query<&ModifierTags>,
    hitpoints_capacity_markers: Query<(), With<HitpointsCapacity>>,
    mut damage_query: Query<(Entity, Option<&mut BodypartDamage>), (With<BodypartChildOfBodypart>, Without<EntityZero>)>,
    mut max_hp_by_part: Local<EntityHashMap<f32>>,
) {
    max_hp_by_part.clear();

    for (part_ent, part_ezero_ref) in parts_query.iter() {
        let mut max_hp = 0.0;
        if let Ok(applied_mods) = part_applied_mods_query.get(part_ent) {
            for mod_ent in applied_mods.iter() {
                let Ok((mod_ent, target, ezero_ref)) = hp_mods_query.get(mod_ent) else {
                    continue;
                };
                if target.0 != part_ent {
                    continue;
                }
                let is_hitpoints_capacity = modifier_has_marker::<HitpointsCapacity>(
                    mod_ent,
                    ezero_ref,
                    &hitpoints_capacity_markers,
                );
                if !is_hitpoints_capacity {
                    continue;
                }
                let tags = resolve_modifier_component(mod_ent, ezero_ref, &modifier_tags_query).unwrap_or_default();
                if tags.contains("current_hp") {
                    continue;
                }
                let base_value = resolve_modifier_component(mod_ent, ezero_ref, &base_values_query);
                let current_value = resolve_modifier_component(mod_ent, ezero_ref, &curr_values_query);
                let value = current_value
                    .map(|v| v.0)
                    .or_else(|| base_value.map(|v| v.0))
                    .unwrap_or(0.0);
                max_hp += value;
            }
        }
        if max_hp <= 0.0 {
            let Some(part_ezero_ref) = part_ezero_ref else {
                continue;
            };
            let Ok(source_applied_mods) = part_applied_mods_query.get(part_ezero_ref.0) else {
                continue;
            };
            for mod_ent in source_applied_mods.iter() {
                let Ok((mod_ent, target, ezero_ref)) = hp_mods_query.get(mod_ent) else {
                    continue;
                };
                if target.0 != part_ezero_ref.0 {
                    continue;
                }
                let is_hitpoints_capacity = modifier_has_marker::<HitpointsCapacity>(
                    mod_ent,
                    ezero_ref,
                    &hitpoints_capacity_markers,
                );
                if !is_hitpoints_capacity {
                    continue;
                }
                let tags = resolve_modifier_component(mod_ent, ezero_ref, &modifier_tags_query).unwrap_or_default();
                if tags.contains("current_hp") {
                    continue;
                }
                let base_value = resolve_modifier_component(mod_ent, ezero_ref, &base_values_query);
                let current_value = resolve_modifier_component(mod_ent, ezero_ref, &curr_values_query);
                let value = current_value
                    .map(|v| v.0)
                    .or_else(|| base_value.map(|v| v.0))
                    .unwrap_or(0.0);
                max_hp += value;
            }
        }
        if max_hp > 0.0 {
            max_hp_by_part.insert(part_ent, max_hp);
        }
    }

    for (part_ent, _) in parts_query.iter() {
        let max_hp = max_hp_by_part
            .get(&part_ent)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        let mut current_hp = max_hp;
        if let Ok((_ent, damage)) = damage_query.get_mut(part_ent) {
            if let Some(damage) = damage {
                current_hp = (max_hp - damage.0).max(0.0);
            }
        }

        if max_hp > 0.0 {
            current_hp = current_hp.clamp(0.0, max_hp);
        } else {
            current_hp = 0.0;
        }

        if current_hp <= 0.0 {
            cmd.entity(part_ent).try_insert_if_new(Missing);
        } else {
            cmd.entity(part_ent).try_remove::<Missing>();
        }
    }
}

#[allow(unused_parens)]
pub fn update_body_health_from_parts(
    mut cmd: Commands,
    time: Res<Time>,
    mut queries: BodyHealthQueryParams,
    mut local_params: BodyHealthLocalParams,
) {
    local_params.bodies.clear();
    local_params.parts.clear();
    local_params.part_to_body.clear();
    local_params.aggregates.clear();
    local_params.max_hp_by_part.clear();
    local_params.body_max_hp_bonus.clear();
    local_params.pain_infliction_by_part.clear();
    local_params.pain_sensitivity_by_part.clear();
    local_params.pain_infliction_by_body.clear();
    local_params.pain_sensitivity_by_body.clear();
    let mut instance_part_by_source_part: EntityHashMap<Entity> = EntityHashMap::default();

    queries.bodies_query.iter().for_each(|ent| {
        local_params.bodies.insert(ent);
    });

    queries.parts_query.iter().for_each(|(ent, _, _, part_ezero_ref, _)| {
        local_params.parts.insert(ent);
        let Some(part_ezero_ref) = part_ezero_ref else {
            return;
        };
        instance_part_by_source_part.insert(part_ezero_ref.0, ent);
    });

    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_hitpoints_capacity = modifier_has_marker::<HitpointsCapacity>(
            mod_ent,
            ezero_ref,
            &queries.hitpoints_capacity_markers,
        );
        if !is_hitpoints_capacity {
            continue;
        }
        let tags = resolve_modifier_component(mod_ent, ezero_ref, &queries.modifier_tags_query).unwrap_or_default();
        let base_value = resolve_modifier_component(mod_ent, ezero_ref, &queries.base_values_query);
        let current_value = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query);
        let value = current_value
            .map(|v| v.0)
            .or_else(|| base_value.map(|v| v.0))
            .unwrap_or(0.0);
        let is_current = tags.contains("current_hp");

        if !is_current && local_params.parts.contains(&target.0) {
            *local_params.max_hp_by_part.entry(target.0).or_insert(0.0) += value;
        } else if !is_current
            && let Some(part_ent) = instance_part_by_source_part.get(&target.0)
        {
            *local_params.max_hp_by_part.entry(*part_ent).or_insert(0.0) += value;
        } else if local_params.bodies.contains(&target.0) && !is_current {
            *local_params.body_max_hp_bonus.entry(target.0).or_insert(0.0) += value;
        }
    }

    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_pain_infliction = modifier_has_marker::<PainInfliction>(
            mod_ent,
            ezero_ref,
            &queries.pain_infliction_markers,
        );
        if !is_pain_infliction {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        if local_params.parts.contains(&target.0) {
            *local_params.pain_infliction_by_part.entry(target.0).or_insert(0.0) += value.0;
        } else if let Some(part_ent) = instance_part_by_source_part.get(&target.0) {
            *local_params.pain_infliction_by_part.entry(*part_ent).or_insert(0.0) += value.0;
        } else if local_params.bodies.contains(&target.0) {
            *local_params.pain_infliction_by_body.entry(target.0).or_insert(0.0) += value.0;
        }
    }
    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_pain_sensitivity = modifier_has_marker::<PainSensitivity>(
            mod_ent,
            ezero_ref,
            &queries.pain_sensitivity_markers,
        );
        if !is_pain_sensitivity {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        if local_params.parts.contains(&target.0) {
            *local_params.pain_sensitivity_by_part.entry(target.0).or_insert(0.0) += value.0;
        } else if let Some(part_ent) = instance_part_by_source_part.get(&target.0) {
            *local_params.pain_sensitivity_by_part.entry(*part_ent).or_insert(0.0) += value.0;
        } else if local_params.bodies.contains(&target.0) {
            *local_params.pain_sensitivity_by_body.entry(target.0).or_insert(0.0) += value.0;
        }
    }

    for (part_ent, body_of, vital, part_ezero_ref, missing) in queries.parts_query.iter() {
        let body_ent = body_of.parent();
        if !local_params.bodies.contains(&body_ent) {
            continue;
        }

        local_params.part_to_body.insert(part_ent, body_ent);
        let agg = local_params.aggregates.entry(body_ent).or_default();

        let max_hp = local_params.max_hp_by_part
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
        let source_part_pain_infliction = part_ezero_ref.and_then(|part_ezero_ref| {
            let Ok(applied_mods) = queries.part_applied_mods_query.get(part_ezero_ref.0) else {
                return None;
            };
            let mut total = 0.0;
            for mod_ent in applied_mods.iter() {
                let Ok((mod_ent, _, ezero_ref)) = queries.modifiers_query.get(mod_ent) else {
                    continue;
                };
                let is_pain_infliction = modifier_has_marker::<PainInfliction>(
                    mod_ent,
                    ezero_ref,
                    &queries.pain_infliction_markers,
                );
                if !is_pain_infliction {
                    continue;
                }
                let Some(value) = resolve_modifier_component(
                    mod_ent,
                    ezero_ref,
                    &queries.curr_values_query,
                ) else {
                    continue;
                };
                total += value.0;
            }
            Some(total)
        }).unwrap_or(0.0);

        let pain_infliction = local_params.pain_infliction_by_part
            .get(&part_ent)
            .copied()
            .unwrap_or(0.0)
            + source_part_pain_infliction
            + local_params.pain_infliction_by_body
                .get(&body_ent)
                .copied()
                .unwrap_or(0.0);
        let source_part_pain_sensitivity = part_ezero_ref.and_then(|part_ezero_ref| {
            let Ok(applied_mods) = queries.part_applied_mods_query.get(part_ezero_ref.0) else {
                return None;
            };
            let mut total = 0.0;
            let mut has_any = false;
            for mod_ent in applied_mods.iter() {
                let Ok((mod_ent, _, ezero_ref)) = queries.modifiers_query.get(mod_ent) else {
                    continue;
                };
                let is_pain_sensitivity = modifier_has_marker::<PainSensitivity>(
                    mod_ent,
                    ezero_ref,
                    &queries.pain_sensitivity_markers,
                );
                if !is_pain_sensitivity {
                    continue;
                }
                let Some(value) = resolve_modifier_component(
                    mod_ent,
                    ezero_ref,
                    &queries.curr_values_query,
                ) else {
                    continue;
                };
                total += value.0;
                has_any = true;
            }
            if has_any { Some(total) } else { None }
        }).unwrap_or(1.0);

        let pain_sensitivity_part = local_params.pain_sensitivity_by_part
            .get(&part_ent)
            .copied()
            .unwrap_or(0.0)
            + source_part_pain_sensitivity;
        let pain_sensitivity_body = local_params.pain_sensitivity_by_body
            .get(&body_ent)
            .copied()
            .unwrap_or(1.0);
        let pain_sensitivity_mult = pain_sensitivity_part * pain_sensitivity_body;
        let pain_mult = (1.0 + pain_infliction) * pain_sensitivity_mult;
        let part_pain = pain_ratio * pain_mult;

        if !missing {
            agg.total_max_hp += max_hp;
            agg.total_hp += current_hp;
            agg.total_pain += part_pain;
        } else if vital.is_some() {
            agg.vital_missing = true;
        }
    }

    // Note: bleed_mod_sum, blood_capacity_mod_sum, hp_regen_mod_sum, consciousness_mod_sum, vision_mod_sum are reused
    // They are accessed in the body loop below, so they are scoped here
    local_params.bleed_mod_sum.clear();
    local_params.blood_capacity_mod_sum.clear();
    local_params.hp_regen_mod_sum.clear();
    local_params.consciousness_mod_sum.clear();
    local_params.vision_mod_sum.clear();
    local_params.body_regen_map.clear();

    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_bleed = modifier_has_marker::<BleedRate>(mod_ent, ezero_ref, &queries.bleed_markers);
        if !is_bleed {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut local_params.bleed_mod_sum, target.0, value.0, &local_params.bodies, &local_params.part_to_body);
    }
    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_blood_capacity = modifier_has_marker::<BloodCapacity>(
            mod_ent,
            ezero_ref,
            &queries.blood_capacity_markers,
        );
        if !is_blood_capacity {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(
            &mut local_params.blood_capacity_mod_sum,
            target.0,
            value.0,
            &local_params.bodies,
            &local_params.part_to_body,
        );
    }
    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_hp_regen = modifier_has_marker::<HitpointRegenRate>(
            mod_ent,
            ezero_ref,
            &queries.hp_regen_markers,
        );
        if !is_hp_regen {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(
            &mut local_params.hp_regen_mod_sum,
            target.0,
            value.0,
            &local_params.bodies,
            &local_params.part_to_body,
        );
    }
    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_consciousness = modifier_has_marker::<Consciousness>(
            mod_ent,
            ezero_ref,
            &queries.consciousness_markers,
        );
        if !is_consciousness {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(
            &mut local_params.consciousness_mod_sum,
            target.0,
            value.0,
            &local_params.bodies,
            &local_params.part_to_body,
        );
    }
    for (mod_ent, target, ezero_ref) in queries.modifiers_query.iter() {
        let is_vision = modifier_has_marker::<Vision>(mod_ent, ezero_ref, &queries.vision_markers);
        if !is_vision {
            continue;
        }
        let Some(value) = resolve_modifier_component(mod_ent, ezero_ref, &queries.curr_values_query) else {
            continue;
        };
        add_modifier_sum(&mut local_params.vision_mod_sum, target.0, value.0, &local_params.bodies, &local_params.part_to_body);
    }

    let delta = time.delta_secs();

    for body in local_params.bodies.iter() {
        let agg = local_params.aggregates.get(body).cloned().unwrap_or_default();
        let total_max_hp =
            (agg.total_max_hp + local_params.body_max_hp_bonus.get(body).copied().unwrap_or(0.0)).max(0.0);
        let total_hp = agg.total_hp.clamp(0.0, total_max_hp);
        let bleed_rate = agg.bleed_rate + local_params.bleed_mod_sum.get(body).copied().unwrap_or(0.0);
        let blood_capacity =
            (agg.blood_capacity + local_params.blood_capacity_mod_sum.get(body).copied().unwrap_or(0.0))
                .max(0.0);
        let regen_rate = agg.regen_rate + local_params.hp_regen_mod_sum.get(body).copied().unwrap_or(0.0);
        let base_consciousness =
            (1.0 + local_params.consciousness_mod_sum.get(body).copied().unwrap_or(0.0)).max(0.0);
        let pain = agg.total_pain.max(0.0);
        let vision = local_params.vision_mod_sum
            .get(body)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);

        let mut blood = match queries.body_health_query.get_mut(*body) {
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
        if agg.vital_missing {
            consciousness = 0.0;
        }

        let dead = agg.vital_missing || blood <= 0.0;

        match queries.body_health_query.get_mut(*body) {
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
                cmd.entity(*body).try_insert(BodySums {
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

        match queries.dead_query.get_mut(*body) {
            Ok(Some(_)) if !dead => {
                cmd.entity(*body).try_remove::<Dead>();
            }
            Ok(None) if dead => {
                cmd.entity(*body).try_insert(Dead);
            }
            Ok(_) => {}
            Err(_) => {}
        }

        local_params.body_regen_map.insert(*body, (regen_rate, total_max_hp));
    }

    for (part_ent, damage) in queries.damage_query.iter_mut() {
        let Some((regen_rate, total_max_hp)) = local_params.part_to_body
            .get(&part_ent)
            .and_then(|body| local_params.body_regen_map.get(body))
        else {
            continue;
        };
        if *total_max_hp <= 0.0 {
            continue;
        }

        let max_hp = local_params.max_hp_by_part
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
