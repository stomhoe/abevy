use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::common_components::StrId;
use game_common::game_common_components::EntityZero;
use modifier_shared::{modifier_components::*, modifier_types::*};

use game_common::game_common_components::Dead;

use crate::body::{body_tree_components::*, body_part::body_part_components::*};

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
    parts_query: Query<'w, 's, (Entity, &'static BodyPartOf, Option<&'static BodyPartVital>, Has<BodyPartMissing>), (With<BodyPart>, Without<EntityZero>)>,
    hp_capacity_mods_query: Query<'w, 's, (&'static ModifierTarget, Option<&'static ModifierTags>, Option<&'static BaseValue>, Option<&'static CurrEffectiveValue>), With<HitpointsCapacity>>,
    damage_query: Query<'w, 's, (Entity, Option<&'static mut BodyPartDamage>), With<BodyPart>>,
    body_health_query: Query<'w, 's, Option<&'static mut BodySums>>,
    dead_query: Query<'w, 's, Option<&'static mut Dead>>,
    bleed_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<BleedRate>>,
    blood_capacity_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<BloodCapacity>>,
    hp_regen_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<HitpointRegenRate>>,
    consciousness_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<Consciousness>>,
    pain_infliction_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<PainInfliction>>,
    pain_sensitivity_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<PainSensitivity>>,
    vision_mods: Query<'w, 's, (&'static ModifierTarget, &'static CurrEffectiveValue), With<Vision>>,
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
    bodies_query: Query<(Option<&BodyParts>), (With<BodyOf>, Without<EntityZero>)>,
    parts_query: Query<
        (Option<&BodyPartCoverageWeight>, Option<&BodyPartMissing>),
        (With<BodyPart>, Without<EntityZero>),
    >,
    mut damage_query: Query<Option<&mut BodyPartDamage>, (With<BodyPart>, Without<EntityZero>)>,
    mut weighted_parts: Local<Vec<(Entity, u16)>>,
) {
    for damage_msg in reader.read() {
        weighted_parts.clear();
        let mut total_weight: u32 = 0;

        let body_ent = damage_msg.body;
        let damage_amount = damage_msg.amount;

        let parts = match bodies_query.get(body_ent) {
            Ok(Some(p)) => p,
            Ok(None) | Err(_) => continue,
        };

        for &part_ent in parts.entities().iter() {
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
            match damage_query.get_mut(part_ent) {
                Ok(Some(mut part_damage)) => part_damage.0 += damage_amount.max(0.0),
                Ok(None) => {
                    cmd.entity(part_ent)
                        .try_insert(BodyPartDamage(damage_amount.max(0.0)));
                }
                Err(_) => {}
            }
        }
    }
}

#[allow(unused_parens)]
pub fn sync_body_part_missing(
    mut cmd: Commands,
    parts_query: Query<(Entity, Option<&BodyPartMissing>), With<BodyPart>>,
    hp_mods_query: Query<
        (
            Entity,
            &ModifierTarget,
            Option<&ModifierTags>,
            Option<&BaseValue>,
            Option<&CurrEffectiveValue>,
        ),
        With<HitpointsCapacity>,
    >,
    mut damage_query: Query<(Entity, Option<&mut BodyPartDamage>), With<BodyPart>>,
    mut to_remove_missing: Local<Vec<Entity>>,
    mut to_add_missing: Local<Vec<Entity>>,
    mut parts: Local<EntityHashSet>,
    mut max_hp_by_part: Local<EntityHashMap<f32>>,
) {
    to_remove_missing.clear();
    to_add_missing.clear();
    parts.clear();
    max_hp_by_part.clear();

    parts_query.iter().for_each(|(ent, _)| {
        parts.insert(ent);
    });
    for (_mod_ent, target, _tags, base_value, current_value) in hp_mods_query.iter() {
        let value = current_value
            .map(|v| v.0)
            .or_else(|| base_value.map(|v| v.0))
            .unwrap_or(0.0);
        if parts.contains(&target.0) {
            *max_hp_by_part.entry(target.0).or_insert(0.0) += value;
        }
    }

    for (part_ent, missing) in parts_query.iter() {
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
            if missing.is_none() {
                to_add_missing.push(part_ent);
            }
        } else if missing.is_some() {
            to_remove_missing.push(part_ent);
        }
    }

    to_add_missing.iter().for_each(|&ent| {
        cmd.entity(ent).try_insert(BodyPartMissing);
    });
    to_remove_missing.iter().for_each(|&ent| {
        cmd.entity(ent).try_remove::<BodyPartMissing>();
    });
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

    queries.bodies_query.iter().for_each(|ent| {
        local_params.bodies.insert(ent);
    });

    queries.parts_query.iter().for_each(|(ent, ..)| {
        local_params.parts.insert(ent);
    });

    for (target, tags, base_value, current_value) in queries.hp_capacity_mods_query.iter() {
        let value = current_value
            .map(|v| v.0)
            .or_else(|| base_value.map(|v| v.0))
            .unwrap_or(0.0);
        let is_current = tags.map(|t| t.contains("current_hp")).unwrap_or(false);

        if local_params.parts.contains(&target.0) && !is_current {
            *local_params.max_hp_by_part.entry(target.0).or_insert(0.0) += value;
        } else if local_params.bodies.contains(&target.0) && !is_current {
            *local_params.body_max_hp_bonus.entry(target.0).or_insert(0.0) += value;
        }
    }

    for (target, value) in queries.pain_infliction_mods.iter() {
        if local_params.parts.contains(&target.0) {
            *local_params.pain_infliction_by_part.entry(target.0).or_insert(0.0) += value.0;
        } else if local_params.bodies.contains(&target.0) {
            *local_params.pain_infliction_by_body.entry(target.0).or_insert(0.0) += value.0;
        }
    }
    for (target, value) in queries.pain_sensitivity_mods.iter() {
        if local_params.parts.contains(&target.0) {
            *local_params.pain_sensitivity_by_part.entry(target.0).or_insert(0.0) += value.0;
        } else if local_params.bodies.contains(&target.0) {
            *local_params.pain_sensitivity_by_body.entry(target.0).or_insert(0.0) += value.0;
        }
    }

    for (part_ent, body_of, vital, missing) in queries.parts_query.iter() {
        if !local_params.bodies.contains(&body_of.body) {
            continue;
        }

        local_params.part_to_body.insert(part_ent, body_of.body);
        let agg = local_params.aggregates.entry(body_of.body).or_default();

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
        let pain_infliction = local_params.pain_infliction_by_part
            .get(&part_ent)
            .copied()
            .unwrap_or(0.0)
            + local_params.pain_infliction_by_body
                .get(&body_of.body)
                .copied()
                .unwrap_or(0.0);
        let pain_sensitivity_part = local_params.pain_sensitivity_by_part
            .get(&part_ent)
            .copied()
            .unwrap_or(1.0);
        let pain_sensitivity_body = local_params.pain_sensitivity_by_body
            .get(&body_of.body)
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

    for (target, value) in queries.bleed_mods.iter() {
        add_modifier_sum(&mut local_params.bleed_mod_sum, target.0, value.0, &local_params.bodies, &local_params.part_to_body);
    }
    for (target, value) in queries.blood_capacity_mods.iter() {
        add_modifier_sum(
            &mut local_params.blood_capacity_mod_sum,
            target.0,
            value.0,
            &local_params.bodies,
            &local_params.part_to_body,
        );
    }
    for (target, value) in queries.hp_regen_mods.iter() {
        add_modifier_sum(
            &mut local_params.hp_regen_mod_sum,
            target.0,
            value.0,
            &local_params.bodies,
            &local_params.part_to_body,
        );
    }
    for (target, value) in queries.consciousness_mods.iter() {
        add_modifier_sum(
            &mut local_params.consciousness_mod_sum,
            target.0,
            value.0,
            &local_params.bodies,
            &local_params.part_to_body,
        );
    }
    for (target, value) in queries.vision_mods.iter() {
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
    mut body_health_query: Query<(Entity, &BodyOf, &BodySums), Changed<BodySums>>,
    mut slowdown_mod_query: Query<(&ModifierTarget, &mut BaseValue), With<PainSlowdown>>,
    mut has_slowdown: Local<EntityHashSet>,
) {
    has_slowdown.clear();

    for (target, mut base_value) in slowdown_mod_query.iter_mut() {
        for (_, body_of, body_health) in body_health_query.iter_mut() {
            if body_of.being != target.0 {
                continue;
            }
            base_value.0 = (1.0 - body_health.pain).max(0.0);
            break;
        }
    }
    for (target, _) in slowdown_mod_query.iter() {
        has_slowdown.insert(target.0);
    }

    for (body_ent, body_of, body_health) in body_health_query.iter_mut() {
        if has_slowdown.contains(&body_of.being) {
            continue;
        }
        has_slowdown.insert(body_of.being);

        let pain_multiplier = (1.0 - body_health.pain).max(0.0);

        let bundle = (
            ModifierTarget(body_of.being),
            BaseValue(pain_multiplier),
            CurrEffectiveValue(pain_multiplier),
            ApplyMode::Mul,
            PainSlowdown,
            ModifierTags::default(),
            StrId::trunc("pain_slowdown"),
            ChildOf(body_ent),
        );

        cmd.spawn((WalkSpeed, bundle.clone()));
        cmd.spawn((SwimSpeed, bundle.clone()));
        cmd.spawn((FlySpeed, bundle.clone()));
    }
}
