use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use common::common_components::HashId;
use common::log_targets::BODY_ENERGY_SYSTEM;
use game_common::game_common_components::{Templ, TemplEntiRef};
use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker, resolve_modifier_component};
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_components::*;
use tilemap_shared::{Dimension, DimensionRef, Gravity};
use tilemap_shared::DimensionEntityMap;

use crate::body::body_components::*;
use ::being_shared::body_energy::*;

const KCAL_PER_KG_FAT: f32 = 7700.0;
const KCAL_PER_KG_LEAN: f32 = 1500.0;
const SECONDS_PER_DAY: f32 = 86_400.0;
const DEFAULT_KCAL_PER_KG_LEAN_PER_DAY: f32 = 30.0;
const MIN_LEAN_MASS_FLOOR_RATIO: f32 = 0.35;
const SOFT_DEFAULT_FAT_CAPACITY_RATIO: f32 = 0.20;
const HARD_DEFAULT_FAT_CAPACITY_RATIO: f32 = 0.60;
const MAX_DIGEST_KCAL_PER_SEC: f32 = 0.8;

#[derive(SystemParam)]
pub struct BodyWeightSumQueries<'w, 's> {
    body_changed_query: Query<'w, 's, Entity, (With<BodyOf>, Or<(Added<BodyOf>, Changed<BodypartChildrenBodyparts>)>)>,
    being_dim_changed_query: Query<'w, 's, Entity, (With<Being>, Changed<DimensionRef>)>,
    part_body_changed_query: Query<'w, 's, &'static ChildOf, (With<BodypartChildOfBodypart>, Or<(Changed<BodypartChildOfBodypart>, Changed<Missing>, Changed<BodypartChildrenBodyparts>)>)>,
    parts_query: Query<'w, 's, (Entity, &'static ChildOf, Option<&'static TemplEntiRef>, Has<Missing>), With<BodypartChildOfBodypart>>,
    part_applied_mods_query: Query<'w, 's, &'static AppliedModifiers>,
    mass_modifiers_query: Query<'w, 's, (Entity, &'static ModifierTarget, Option<&'static TemplEntiRef>), Without<Templ>>,
    curr_values_query: Query<'w, 's, &'static CurrEffectiveValue>,
    mass_markers_query: Query<'w, 's, (), With<MassKg>>,
    body_of_query: Query<'w, 's, (Entity, &'static BodyOf), With<BodyOf>>,
    being_dim_query: Query<'w, 's, &'static DimensionRef, With<Being>>,
    dimension_map: Res<'w, DimensionEntityMap>,
    dimension_hash_query: Query<'w, 's, &'static HashId, With<Dimension>>,
    being_weight_query: Query<'w, 's, &'static mut BodyWeightSum, With<Being>>,
    body_energy_store_query: Query<'w, 's, &'static mut BodyEnergyStore, (With<BodyOf>, Without<Templ>)>,
    gravity_changed_query: Query<'w, 's, Entity, (With<Dimension>, Changed<Gravity>)>,
    gravity_query: Query<'w, 's, &'static Gravity, With<Dimension>>,
    removed_missing: RemovedComponents<'w, 's, Missing>,
}

#[derive(SystemParam)]
pub struct BodyWeightSumLocals<'s> {
    body_and_dim_cache: Local<'s, (EntityHashSet, HashSet<HashId>)>,
}

/// Recomputes the body's current weight from body-part mass plus dynamic energy mass changes.
/// The weight sum is derived from the current energy store so starvation and fat gain affect physics immediately.
#[allow(unused_parens, )]
pub fn update_body_weight_sum(
    mut cmd: Commands,
    queries: BodyWeightSumQueries,
    mut locals: BodyWeightSumLocals,
) {
    let BodyWeightSumQueries {
        body_changed_query,
        being_dim_changed_query,
        part_body_changed_query,
        parts_query,
        part_applied_mods_query,
        mass_modifiers_query,
        curr_values_query,
        mass_markers_query,
        body_of_query,
        being_dim_query,
        dimension_map,
        dimension_hash_query,
        mut being_weight_query,
        mut body_energy_store_query,
        gravity_changed_query,
        gravity_query,
        mut removed_missing,
    } = queries;
    let BodyWeightSumLocals {
        body_and_dim_cache,
    } = &mut locals;
    let (affected_bodies, changed_dims) = &mut **body_and_dim_cache;
    affected_bodies.clear();
    changed_dims.clear();
    for body_ent in body_changed_query.iter() {
        affected_bodies.insert(body_ent);
    }
    for being_ent in being_dim_changed_query.iter() {
        for (body_ent, body_of) in body_of_query.iter() {
            if body_of.being != being_ent {
                continue;
            }
            affected_bodies.insert(body_ent);
        }
    }
    for dim_ent in gravity_changed_query.iter() {
        let Ok(&dim_hash) = dimension_hash_query.get(dim_ent) else {
            continue;
        };
        changed_dims.insert(dim_hash);
    }
    for body_of in part_body_changed_query.iter() {
        affected_bodies.insert(body_of.parent());
    }
    for part_ent in removed_missing.read() {
        let Ok((_, body_of, _, _)) = parts_query.get(part_ent) else { continue };
        affected_bodies.insert(body_of.parent());
    }
    if !changed_dims.is_empty() {
        for (body_ent, body_of) in body_of_query.iter() {
            let Ok(dim_ref) = being_dim_query.get(body_of.being) else { continue };
            if changed_dims.contains(&dim_ref.0) {
                affected_bodies.insert(body_ent);
            }
        }
    }
    if affected_bodies.is_empty() {
        return;
    }

    let mut mass_per_body: EntityHashMap<f32> = EntityHashMap::default();
    for (part_ent, body_of, part_templ_ref, missing) in parts_query.iter() {
        let body_ent = body_of.parent();
        if missing || !affected_bodies.contains(&body_ent) {
            continue;
        }
        let mut part_mass = 0.0;
        let mut effects = EntityHashSet::default();
        collect_applied_modifier_entities(&mut effects, part_ent, part_templ_ref, &part_applied_mods_query);
        for mod_ent in effects.iter() {
            let Ok((entity, target, templ_ref)) = mass_modifiers_query.get(*mod_ent) else {
                continue;
            };
            if target.0 != part_ent
                && part_templ_ref.map(|part_templ_ref| target.0 != part_templ_ref.0).unwrap_or(true)
            {
                continue;
            }
            if !modifier_has_marker::<MassKg>(entity, templ_ref, &mass_markers_query) {
                continue;
            }
            let Some(value) = resolve_modifier_component(entity, templ_ref, &curr_values_query) else {
                continue;
            };
            part_mass += value.0.max(0.0);
        }
        *mass_per_body.entry(body_ent).or_insert(0.0) += part_mass;
    }

    for body_ent in affected_bodies.iter() {
        let Ok((_, body_of)) = body_of_query.get(*body_ent) else { continue };
        let Ok(dim_ref) = being_dim_query.get(body_of.being) else { continue };
        let Some(dim_ent) = dimension_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        let gravity = gravity_query.get(dim_ent).copied().unwrap_or_default();
        let mut total_mass = mass_per_body.get(body_ent).copied().unwrap_or_default().max(0.0);
        if let Ok(mut energy_store) = body_energy_store_query.get_mut(*body_ent) {
            if energy_store.baseline_mass_kg <= 0.0 && total_mass > 0.0 {
                energy_store.baseline_mass_kg = total_mass;
                energy_store.lean_mass_kg = total_mass;
                energy_store.fat_mass_kg = total_mass * 0.12;
            } else if total_mass < energy_store.baseline_mass_kg {
                let prev_baseline_mass = energy_store.baseline_mass_kg;
                let prev_lean_mass = energy_store.lean_mass_kg;
                let mass_loss = (prev_baseline_mass - total_mass).max(0.0);
                energy_store.baseline_mass_kg = total_mass;
                energy_store.lean_mass_kg = (energy_store.lean_mass_kg - mass_loss).max(0.0);
                trace!(
                    target: BODY_ENERGY_SYSTEM,
                    "Shrank body energy baseline for body {:?} being {:?}: structural_mass {:.3}->{:.3} lean {:.3}->{:.3} fat {:.3}",
                    body_ent,
                    body_of.being,
                    prev_baseline_mass,
                    energy_store.baseline_mass_kg,
                    prev_lean_mass,
                    energy_store.lean_mass_kg,
                    energy_store.fat_mass_kg,
                );
            }
            let dynamic_delta_mass = (energy_store.lean_mass_kg + energy_store.fat_mass_kg - energy_store.baseline_mass_kg).max(-energy_store.baseline_mass_kg * 0.9);
            total_mass = (total_mass + dynamic_delta_mass).max(0.0);
        }
        let total_weight = gravity.mass_to_newtons(total_mass).max(0.0);
        let Ok(mut prev_weight) = being_weight_query.get_mut(body_of.being) else {
            cmd.entity(body_of.being).try_insert(BodyWeightSum(total_weight));
            continue;
        };
        if (prev_weight.0 - total_weight).abs() > f32::EPSILON {
            prev_weight.0 = total_weight;
        }
    }
}

/// Ensures newly spawned bodies have the runtime energy components needed by the hunger system.
/// This is the bootstrap step that gives bodies their runtime energy store and derived condition components.
#[allow(unused_parens, )]
pub fn ensure_body_energy_components(
    mut cmd: Commands,
    new_bodies: Query<(Entity, &'static BodyOf, ), (Added<BodyOf>, )>,
) {
    for (body_ent, body_of, ) in new_bodies.iter() {
        cmd.entity(body_ent).try_insert_if_new(BodyEnergyStore::default());
        cmd.entity(body_ent).try_insert_if_new(BodyEnergyBalance::default());
        cmd.entity(body_of.being).try_insert_if_new(BodyCondition::default());
        cmd.entity(body_of.being).try_insert_if_new(BodyStrengthScale::default());
    }
}

/// Adds calories from food or other digestion systems into the body's short-term stomach storage.
/// This is message-driven because food intake usually happens as a separate gameplay event.
#[allow(unused_parens, )]
pub fn add_calories_to_being_energy_store(
    mut calories: MessageReader<AddCaloriesToBeing>,
    held_body_query: Query<&'static HeldBody, (With<Being>, )>,
    mut body_energy_store_query: Query<&'static mut BodyEnergyStore, (With<BodyOf>, Without<Templ>, )>,
) {
    for msg in calories.read() {
        let Ok(held_body) = held_body_query.get(msg.being) else {
            trace!(target: BODY_ENERGY_SYSTEM, "Ignored AddCaloriesToBeing for {:?}: no HeldBody", msg.being);
            continue;
        };
        let body_ent = held_body.entity();
        let Ok(mut body_energy_store) = body_energy_store_query.get_mut(body_ent) else {
            trace!(target: BODY_ENERGY_SYSTEM, "Ignored AddCaloriesToBeing for {:?}: body {:?} has no BodyEnergyStore", msg.being, body_ent);
            continue;
        };
        body_energy_store.stomach_kcal += msg.kcal.max(0.0);
        trace!(target: BODY_ENERGY_SYSTEM, "Added {:.2} kcal to being {:?} body {:?} (stomach_kcal={:.2})", msg.kcal, msg.being, body_ent, body_energy_store.stomach_kcal);
    }
}

/// Updates the body's activity multiplier from its current movement state.
/// Higher speed means the body spends more energy this tick.
#[allow(unused_parens, unused)]
pub fn ______________update_body_energy_activity_multipliers(
    mut bodies_query: Query<(&'static BodyOf, &'static mut BodyEnergyStore, ), (With<BodyOf>, Without<Templ>, )>,
    speed_query: Query<&'static SpeedMagnitude, (With<Being>, )>,
) {
    return;
    for (body_of, mut body_energy_store, ) in bodies_query.iter_mut() {
        let speed_magnitude = speed_query.get(body_of.being).map_or(0.0, |speed| speed.0.max(0.0));
        body_energy_store.activity_multiplier = (1.0 + speed_magnitude * 0.35).clamp(0.5, 3.0);
    }
}

/// Ticks the global energy model once per second and converts deficits into fat loss, muscle loss, hunger, and starvation damage.
/// This is the core loop that turns stored calories into reserve loss and body penalties.
#[allow(unused_parens, )]
pub fn tick_global_body_energy(
    time: Res<Time>,
    mut body_query: Query<(Entity, &'static BodyOf, &'static TemplEntiRef, &'static mut BodyEnergyStore, &'static mut BodyEnergyBalance, ), (With<BodyOf>, Without<Templ>, )>,
    starvation_config_query: Query<&'static StarvationConfig, (With<Body>, With<Templ>, )>,
    body_energy_profile_query: Query<&'static BodyEnergyProfile, (With<Body>, With<Templ>, )>,
    mut body_condition_query: Query<&'static mut BodyCondition, (With<Being>, )>,
    mut body_strength_scale_query: Query<&'static mut BodyStrengthScale, (With<Being>, )>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (body_ent, body_of, body_templ_ref, mut body_energy_store, mut energy_balance, ) in body_query.iter_mut() {
        if body_energy_store.baseline_mass_kg <= 0.0 {
            continue;
        }
        let Ok(starvation_config) = starvation_config_query.get(body_templ_ref.0) else {
            trace!(target: BODY_ENERGY_SYSTEM, "Missing StarvationConfig on body template {:?} for body {:?} being {:?}", body_templ_ref.0, body_ent, body_of.being);
            continue;
        };
        let Ok(body_energy_profile) = body_energy_profile_query.get(body_templ_ref.0) else {
            trace!(target: BODY_ENERGY_SYSTEM, "Missing BodyEnergyProfile on body template {:?} for body {:?} being {:?}", body_templ_ref.0, body_ent, body_of.being);
            continue;
        };
        let min_lean_mass = (body_energy_store.baseline_mass_kg * MIN_LEAN_MASS_FLOOR_RATIO).max(0.1);
        if body_energy_store.lean_mass_kg <= 0.0 {
            body_energy_store.lean_mass_kg = body_energy_store.baseline_mass_kg;
        }
        body_energy_store.lean_mass_kg = body_energy_store.lean_mass_kg.max(min_lean_mass);
        body_energy_store.fat_mass_kg = body_energy_store.fat_mass_kg.max(0.0);

        // Base upkeep starts from lean body mass, then species and situation modifiers scale it.
        // Thermal multiplier lives on the runtime store because weather, wetness, and shelter can change it over time.
        let upkeep_kcal_per_day = body_energy_store.lean_mass_kg
            * DEFAULT_KCAL_PER_KG_LEAN_PER_DAY
            * body_energy_profile.burn_rate_multiplier.max(0.05)
            * body_energy_store.activity_multiplier.max(0.25);
        body_energy_store.burn_kcal_per_sec = upkeep_kcal_per_day / SECONDS_PER_DAY * body_energy_store.thermal_multiplier.max(0.05);

        // Move calories from stomach into usable energy, then subtract the current upkeep.
        let digested_kcal = body_energy_store.stomach_kcal.min(MAX_DIGEST_KCAL_PER_SEC * dt);
        body_energy_store.stomach_kcal -= digested_kcal;
        let burned_kcal = body_energy_store.burn_kcal_per_sec * dt;
        let net_kcal = digested_kcal - burned_kcal;
        energy_balance.last_tick_net_kcal = net_kcal;

        let soft_fat_cap_kg = (body_energy_store.baseline_mass_kg * SOFT_DEFAULT_FAT_CAPACITY_RATIO * body_energy_profile.healthy_fat_capacity_multiplier.max(0.1)).max(0.1);
        let hard_fat_cap_kg = (body_energy_store.baseline_mass_kg * HARD_DEFAULT_FAT_CAPACITY_RATIO * body_energy_profile.healthy_fat_capacity_multiplier.max(0.1)).max(soft_fat_cap_kg);
        if net_kcal >= 0.0 {
            // Surplus energy becomes body fat, up to the healthy cap.
            let fat_gain_kg = net_kcal / KCAL_PER_KG_FAT;
            body_energy_store.fat_mass_kg = (body_energy_store.fat_mass_kg + fat_gain_kg).min(hard_fat_cap_kg);
            energy_balance.unresolved_deficit_kcal = (energy_balance.unresolved_deficit_kcal - net_kcal * 0.25).max(0.0);
        } else {
            // Deficit burns fat first, then lean mass once fat can no longer cover the gap.
            let mut deficit_kcal = -net_kcal;
            let fat_kcal_available = body_energy_store.fat_mass_kg * KCAL_PER_KG_FAT;
            let max_fat_mobilization_kcal = starvation_config.max_fat_mobilization_kcal_per_sec * dt;
            let fat_kcal_used = fat_kcal_available.min(deficit_kcal).min(max_fat_mobilization_kcal);
            body_energy_store.fat_mass_kg = (body_energy_store.fat_mass_kg - fat_kcal_used / KCAL_PER_KG_FAT).max(0.0);
            deficit_kcal -= fat_kcal_used;

            if deficit_kcal > 0.0 {
                let max_lean_catabolism_kcal = starvation_config.max_lean_catabolism_kcal_per_sec * dt;
                let lean_catabolism_kcal = deficit_kcal.min(max_lean_catabolism_kcal);
                let lean_loss_kg = lean_catabolism_kcal / KCAL_PER_KG_LEAN * body_energy_profile.wasting_rate_multiplier.max(0.05);
                let prev_lean_mass = body_energy_store.lean_mass_kg;
                body_energy_store.lean_mass_kg = (body_energy_store.lean_mass_kg - lean_loss_kg).max(min_lean_mass);
                let actual_lean_loss_kg = (prev_lean_mass - body_energy_store.lean_mass_kg).max(0.0);
                let covered_by_lean_kcal = actual_lean_loss_kg * KCAL_PER_KG_LEAN;
                deficit_kcal = (deficit_kcal - covered_by_lean_kcal).max(0.0);
            }
            energy_balance.unresolved_deficit_kcal += deficit_kcal;
            if deficit_kcal > 0.0 && body_energy_store.fat_mass_kg <= f32::EPSILON && body_energy_store.lean_mass_kg <= min_lean_mass + f32::EPSILON {
                // Once both fat and lean reserves are exhausted, the body starts taking damage.
                let starvation_damage = (deficit_kcal / 80.0).max(starvation_config.damage_per_sec_at_zero_lean * dt);
                damage_messages.write(IncHealthDamageOrHeal {
                    target_ent: body_of.being,
                    source_ent: Entity::PLACEHOLDER,
                    amount: starvation_damage,
                    distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
                });
                trace!(target: BODY_ENERGY_SYSTEM, "Starvation damage {:.3} applied to being {:?} (body {:?})", starvation_damage, body_of.being, body_ent);
            }
        }

        let lean_ratio = (body_energy_store.lean_mass_kg / body_energy_store.baseline_mass_kg).clamp(0.0, 2.0);
        let wasting = (1.0 - lean_ratio).max(0.0).clamp(0.0, 1.0);
        let obesity = ((body_energy_store.fat_mass_kg / soft_fat_cap_kg) - 1.0).max(0.0).clamp(0.0, 1.0);
        let starvation_kcal_buffer = body_energy_store.stomach_kcal + body_energy_store.fat_mass_kg * KCAL_PER_KG_FAT + ((body_energy_store.lean_mass_kg - min_lean_mass).max(0.0) * KCAL_PER_KG_LEAN);
        let hours_of_reserve = if body_energy_store.burn_kcal_per_sec > 0.0 {
            starvation_kcal_buffer / (body_energy_store.burn_kcal_per_sec * 3600.0)
        } else {
            24.0
        };
        let hunger_ratio = (1.0 - (hours_of_reserve / 24.0)).clamp(0.0, 1.0);

        if let Ok(mut body_condition) = body_condition_query.get_mut(body_of.being) {
            body_condition.hunger_ratio = hunger_ratio;
            body_condition.wasting = wasting;
            body_condition.obesity = obesity;
        }
        if let Ok(mut body_strength_scale) = body_strength_scale_query.get_mut(body_of.being) {
            body_strength_scale.0 = ((lean_ratio * (1.0 - obesity * 0.15)).clamp(0.25, 1.35)).max(0.0);
        }

        trace!(
            target: BODY_ENERGY_SYSTEM,
            "BodyEnergy body={:?} being={:?} burn={:.3}kcal/s net={:.3} lean_kg={:.3} fat_kg={:.3} hunger={:.3} wasting={:.3} obesity={:.3}",
            body_ent,
            body_of.being,
            body_energy_store.burn_kcal_per_sec,
            net_kcal,
            body_energy_store.lean_mass_kg,
            body_energy_store.fat_mass_kg,
            hunger_ratio,
            wasting,
            obesity,
        );
    }
}
