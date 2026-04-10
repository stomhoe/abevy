#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use common::common_id_components::{HashId, HashIdMap};
use common::log_targets::BODY_BUILD;
use game_common::game_common_components::Templ;
use modifier_shared::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierSynergies, ModifierTarget};
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_types::*;
use ::being_shared::*;
use crate::being_interaction_zone_helper::build_being_interaction_zones;

use crate::body::{
    body_components::*,
    body_resources::*,
    bodytree::*,
};
use ::being_shared::body_energy::*;

const STAT_BLEED_RATE: HashId = HashId::hash("bleed_rate");

#[allow(unused_parens)]
pub fn init_templ_bodys(
    mut cmd: Commands,
    body_map: Res<BodyEntityMap>,
    bodytree_map: Res<BodyTreeTemplateEntityMap>,
    bodytree_abstract_query: Query<Has<BodyTreeAbstract>, (With<Templ>, )>,
) {
    if !body_map.0.is_empty() {
        return;
    }
    for seri in load_body_seri_defs() {

        let body_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for BodyConfig: {}", e));
                error!(target: BODY_BUILD, "{}", err);
                continue;
            }
        };
        let body_ent = cmd.spawn_empty().id();
        cmd.entity(body_ent).insert((
            body_id.clone(),
            body_id.hash_id(),
            Body,
            Templ,
            build_being_interaction_zones(
                seri.melee_interaction_zone.clone(),
                seri.collision_zone.clone(),
            ),
        ));

        if seri.name.trim().is_empty() {
            cmd.entity(body_ent)
                .insert(DisplayName::trunc(body_id.as_str()));
        } else {
            cmd.entity(body_ent)
                .insert(DisplayName::trunc(seri.name));
        }

        if !seri.tags.is_empty() {
            cmd.entity(body_ent).insert(TagSet::new(&seri.tags));
        }

        let mut totals = HashIdMap::default();
        for (key, val) in &seri.distributed_totals {
            totals.overwrite(HashId::from(key), val.max(0.0));
        }
        if !totals.contains_key(BodypartStat::STAT_HP_CAPACITY) {
            totals.overwrite(BodypartStat::STAT_HP_CAPACITY, 1.0);
        }
        if !totals.contains_key(BodypartStat::STAT_HP_REGEN_RATE) {
            totals.overwrite(BodypartStat::STAT_HP_REGEN_RATE, 1.0);
        }
        if !totals.contains_key(BodypartStat::STAT_BLOOD_CAPACITY) {
            totals.overwrite(BodypartStat::STAT_BLOOD_CAPACITY, 1.0);
        }
        if !totals.contains_key(BodypartStat::STAT_VISION) {
            totals.overwrite(BodypartStat::STAT_VISION, 1.0);
        }
        if !totals.contains_key(BodypartStat::STAT_CALORIC_BURN_RATE) {
            totals.overwrite(BodypartStat::STAT_CALORIC_BURN_RATE, 1.0);
        }
        if !totals.contains_key(BodypartStat::STAT_WALK_STRENGTH) {
            totals.overwrite(BodypartStat::STAT_WALK_STRENGTH, 300.);
        }
        if !totals.contains_key(BodypartStat::STAT_MASS_KG) {
            error!(target: BODY_BUILD, "Body '{}' is missing distributed_totals.mass_kg; skipping", body_id);
            continue;
        }
        let totals_to_distribute = StatBudgetsToDistributeAmongBodyPartsOfTemplBody(totals.clone());
        let burn_rate_multiplier = if seri.caloric_burn_rate_multiplier <= 0.0 {
            1.0
        } else {
            seri.caloric_burn_rate_multiplier
        };
        let wasting_rate_multiplier = if seri.wasting_rate_multiplier <= 0.0 {
            1.0
        } else {
            seri.wasting_rate_multiplier
        };
        let healthy_fat_capacity_multiplier = if seri.healthy_fat_capacity_multiplier <= 0.0 {
            1.0
        } else {
            seri.healthy_fat_capacity_multiplier
        };
        let max_fat_mobilization_kcal_per_sec = seri.max_fat_mobilization_kcal_per_sec.max(0.0);
        let max_lean_catabolism_kcal_per_sec = seri.max_lean_catabolism_kcal_per_sec.max(0.0);
        let damage_per_sec_at_zero_lean = seri.damage_per_sec_at_zero_lean.max(0.0);
        cmd.entity(body_ent).insert((
            totals_to_distribute,
            BodySexes(seri.sexes.clone()),
            BodyEnergyProfile {
                burn_rate_multiplier,
                wasting_rate_multiplier,
                healthy_fat_capacity_multiplier,
            },
            StarvationConfig {
                max_fat_mobilization_kcal_per_sec,
                max_lean_catabolism_kcal_per_sec,
                damage_per_sec_at_zero_lean,
            },
        ));
        let bodytree_id = seri.bodytree_id.trim();
        if bodytree_id.is_empty() {
            error!(target: BODY_BUILD, "Body '{}' is missing bodytree_id", body_id);
            continue;
        }
        let bodytree_str_id = match StrId::new_with_result(bodytree_id.to_string(), 3) {
            Ok(id) => id,
            Err(err) => {
                error!(target: BODY_BUILD, "Body '{}' has invalid bodytree_id '{}': {}", body_id, bodytree_id, err);
                continue;
            }
        };
        let Ok(bodytree_ent) = bodytree_map.0.get_cloned(&bodytree_str_id) else {
            error!(target: BODY_BUILD, "Body '{}' references missing bodytree '{}'", body_id, bodytree_str_id);
            continue;
        };
        let Ok(is_abstract) = bodytree_abstract_query.get(bodytree_ent) else {
            error!(target: BODY_BUILD, "Body '{}' could not inspect bodytree '{}'", body_id, bodytree_str_id);
            continue;
        };
        if is_abstract {
            error!(target: BODY_BUILD, "Body '{}' references abstract bodytree '{}'; use a derived concrete tree instead", body_id, bodytree_str_id);
            continue;
        }
        cmd.entity(body_ent)
            .insert(BodyTreeRef(HashId::from(bodytree_str_id.as_str())));
        trace!(target: BODY_BUILD, "Body '{}' uses shared bodytree '{}'", body_id, bodytree_str_id);
    }
}

pub(crate) fn distribute_budgets_among_bodyparts_based_on_weights_and_forcings(
    cmd: &mut Commands,
    body_id: &StrId,
    parts: &[(Entity, Entity)],
    body_ent: Entity,
    totals: &StatBudgetsToDistributeAmongBodyPartsOfTemplBody,
    forced_query: &Query<&BodypartForcedStats>,
    weighted_query: &Query<&BodypartWeightedDistribution>,
    synergy_query: &Query<&ModifierSynergies>,
) {
    if parts.is_empty() {
        error!(target: BODY_BUILD, "Body {} has no parts to distribute stats onto", body_id);
        return;
    }

    let empty_stats = HashIdMap::default();
    let mut sum_w_mass = 0.0;
    let mut sum_w_hp = 0.0;
    let mut sum_w_regen = 0.0;
    let mut sum_w_blood = 0.0;
    let mut sum_w_walk_strength = 0.0;
    let mut sum_w_swim_strength = 0.0;
    let mut sum_w_fly_strength = 0.0;
    let mut sum_w_manip = 0.0;
    let mut sum_w_manip_strength = 0.0;
    let mut sum_w_vision = 0.0;
    let mut sum_w_pain = 0.0;
    let mut sum_w_bleed = 0.0;

    let mut forced_mass_kg = 0.0;
    let mut forced_hp = 0.0;
    let mut forced_regen = 0.0;
    let mut forced_blood = 0.0;
    let mut forced_walk_strength = 0.0;
    let mut forced_swim_strength = 0.0;
    let mut forced_fly_strength = 0.0;
    let mut forced_manip = 0.0;
    let mut forced_manip_strength = 0.0;
    let mut forced_vision = 0.0;
    let mut forced_pain = 0.0;
    let mut forced_bleed = 0.0;

    for &(_, source_part) in parts {
        let forced = forced_query.get(source_part).map(|x| &x.0).unwrap_or(&empty_stats);
        let weights = weighted_query.get(source_part).map(|x| &x.0).unwrap_or(&empty_stats);
        let mass = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_MASS_KG);
        if mass > 0.0 { forced_mass_kg += mass; } else { sum_w_mass += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_MASS_KG); }
        let hp = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_HP_CAPACITY);
        if hp > 0.0 { forced_hp += hp; } else { sum_w_hp += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_HP_CAPACITY); }
        let regen = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_HP_REGEN_RATE);
        if regen > 0.0 { forced_regen += regen; } else { sum_w_regen += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_HP_REGEN_RATE); }
        let blood = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_BLOOD_CAPACITY);
        if blood > 0.0 { forced_blood += blood; } else { sum_w_blood += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_BLOOD_CAPACITY); }
        let walk_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_WALK_STRENGTH);
        if walk_strength > 0.0 { forced_walk_strength += walk_strength; } else { sum_w_walk_strength += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_WALK_STRENGTH); }
        let swim_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_SWIM_STRENGTH);
        if swim_strength > 0.0 { forced_swim_strength += swim_strength; } else { sum_w_swim_strength += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_SWIM_STRENGTH); }
        let fly_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_FLY_STRENGTH);
        if fly_strength > 0.0 { forced_fly_strength += fly_strength; } else { sum_w_fly_strength += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_FLY_STRENGTH); }
        let manip = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_DEXTERITY);
        if manip > 0.0 { forced_manip += manip; } else { sum_w_manip += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_DEXTERITY); }
        let manip_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_STRENGTH);
        if manip_strength > 0.0 { forced_manip_strength += manip_strength; } else { sum_w_manip_strength += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_STRENGTH); }
        let vision = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_VISION);
        if vision > 0.0 { forced_vision += vision; } else { sum_w_vision += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_VISION); }
        let pain = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_PAIN_SENSITIVITY);
        if pain > 0.0 { forced_pain += pain; } else { sum_w_pain += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_PAIN_SENSITIVITY); }
        let bleed = get_stat_value_from_hashid_map(forced, STAT_BLEED_RATE);
        if bleed > 0.0 { forced_bleed += bleed; } else { sum_w_bleed += get_stat_value_from_hashid_map(weights, STAT_BLEED_RATE); }
    }

    let free_mass_kg = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_MASS_KG) - forced_mass_kg).max(0.0);
    let free_hp = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_HP_CAPACITY) - forced_hp).max(0.0);
    let free_regen = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_HP_REGEN_RATE) - forced_regen).max(0.0);
    let free_blood = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_BLOOD_CAPACITY) - forced_blood).max(0.0);
    let free_walk_strength = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_WALK_STRENGTH) - forced_walk_strength).max(0.0);
    let free_swim_strength = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_SWIM_STRENGTH) - forced_swim_strength).max(0.0);
    let free_fly_strength = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_FLY_STRENGTH) - forced_fly_strength).max(0.0);
    let free_manip = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_MANIPULATION_DEXTERITY) - forced_manip).max(0.0);
    let free_manip_strength = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_MANIPULATION_STRENGTH) - forced_manip_strength).max(0.0);
    let free_vision = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_VISION) - forced_vision).max(0.0);
    let free_pain = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_PAIN_SENSITIVITY) - forced_pain).max(0.0);
    let free_bleed = (get_stat_value_from_hashid_map(&totals.0, STAT_BLEED_RATE) - forced_bleed).max(0.0);

    for &(part, source_part) in parts {
        let Ok(forced_stats) = forced_query.get(source_part) else {
            error!(target: BODY_BUILD, "Body {} source part {:?} is missing BodypartForcedStats", body_id, source_part);
            continue;
        };
        let Ok(weighted_stats) = weighted_query.get(source_part) else {
            error!(target: BODY_BUILD, "Body {} source part {:?} is missing BodypartWeightedDistribution", body_id, source_part);
            continue;
        };
        let forced = &forced_stats.0;
        let weights = &weighted_stats.0;
        let synergies = synergy_query.get(source_part).ok();

        let mut spawned_modifiers = 0;
        let mass_kg = forced_or_weighted(
            get_stat_value_from_hashid_map(forced, BodypartStat::STAT_MASS_KG),
            get_stat_value_from_hashid_map(weights, BodypartStat::STAT_MASS_KG),
            free_mass_kg,
            sum_w_mass,
        );
        if mass_kg > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(mass_kg), CurrEffectiveValue(mass_kg), ApplyMode::Add, MassKg, ChildOf(part)));
        }

        let forced_hp_capacity = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_HP_CAPACITY);
        let hp = forced_or_weighted(forced_hp_capacity, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_HP_CAPACITY), free_hp, sum_w_hp);
        if hp > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(hp), CurrEffectiveValue(hp), ApplyMode::Add, HitpointsCapacity, ChildOf(part)));
        }
        let forced_regen_rate = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_HP_REGEN_RATE);
        let regen = forced_or_weighted(forced_regen_rate, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_HP_REGEN_RATE), free_regen, sum_w_regen);
        if regen > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(regen), CurrEffectiveValue(regen), ApplyMode::Add, HitpointRegenRate, ChildOf(part)));
        }
        let forced_blood_capacity = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_BLOOD_CAPACITY);
        let blood = forced_or_weighted(forced_blood_capacity, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_BLOOD_CAPACITY), free_blood, sum_w_blood);
        if blood > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(blood), CurrEffectiveValue(blood), ApplyMode::Add, BloodCapacity, ChildOf(part)));
        }
        let forced_walk_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_WALK_STRENGTH);
        let walk_strength = forced_or_weighted(forced_walk_strength, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_WALK_STRENGTH), free_walk_strength, sum_w_walk_strength);
        if walk_strength > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(walk_strength), CurrEffectiveValue(walk_strength), ApplyMode::Add, WalkStrength, ChildOf(part)));
        }
        let forced_swim_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_SWIM_STRENGTH);
        let swim_strength = forced_or_weighted(forced_swim_strength, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_SWIM_STRENGTH), free_swim_strength, sum_w_swim_strength);
        if swim_strength > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(swim_strength), CurrEffectiveValue(swim_strength), ApplyMode::Add, SwimStrength, ChildOf(part)));
        }
        let forced_fly_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_FLY_STRENGTH);
        let fly_strength = forced_or_weighted(forced_fly_strength, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_FLY_STRENGTH), free_fly_strength, sum_w_fly_strength);
        if fly_strength > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(fly_strength), CurrEffectiveValue(fly_strength), ApplyMode::Add, FlyStrength, ChildOf(part)));
        }
        let forced_manipulation = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_DEXTERITY);
        let manip = forced_or_weighted(forced_manipulation, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_DEXTERITY), free_manip, sum_w_manip);
        if manip > 0.0 {
            spawned_modifiers += 1;
            let modifier_ent = cmd.spawn((ModifierTarget(part), BaseValue(manip), CurrEffectiveValue(manip), ApplyMode::Add, ManipulationDexterity, ChildOf(part))).id();
            if let Some(synergies) = synergies {
                cmd.entity(modifier_ent).insert(synergies.clone());
            }
        }
        let forced_manipulation_strength = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_STRENGTH);
        let manip_strength = forced_or_weighted(forced_manipulation_strength, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_STRENGTH), free_manip_strength, sum_w_manip_strength);
        if manip_strength > 0.0 {
            spawned_modifiers += 1;
            let modifier_ent = cmd.spawn((ModifierTarget(part), BaseValue(manip_strength), CurrEffectiveValue(manip_strength), ApplyMode::Add, ManipulationStrength, ChildOf(part))).id();
            if let Some(synergies) = synergies {
                cmd.entity(modifier_ent).insert(synergies.clone());
            }
        }
        let forced_vision = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_VISION);
        let vision = forced_or_weighted(forced_vision, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_VISION), free_vision, sum_w_vision);
        if vision > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(vision), CurrEffectiveValue(vision), ApplyMode::Add, Vision, ChildOf(part)));
        }
        let forced_pain_sensitivity = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_PAIN_SENSITIVITY);
        let pain = forced_or_weighted(forced_pain_sensitivity, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_PAIN_SENSITIVITY), free_pain, sum_w_pain);
        if pain > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(pain), CurrEffectiveValue(pain), ApplyMode::Add, PainSensitivity, ChildOf(part)));
        }
        let forced_bleed_rate = get_stat_value_from_hashid_map(forced, STAT_BLEED_RATE);
        let bleed = forced_or_weighted(forced_bleed_rate, get_stat_value_from_hashid_map(weights, STAT_BLEED_RATE), free_bleed, sum_w_bleed);
        if bleed > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(bleed), CurrEffectiveValue(bleed), ApplyMode::Add, BleedRate, ChildOf(part)));
        }

        if spawned_modifiers == 0 {
            error!(target: BODY_BUILD, "Body {} source part {:?} produced no modifiers; forced and weighted distributions may be empty", body_id, source_part);
        } else {
            trace!(target: BODY_BUILD, "Body {} source part {:?} spawned {} modifiers", body_id, source_part, spawned_modifiers);
        }
    }
}

/// Pick forced value if nonzero, otherwise take my portion of the budget based on my weight relative to the sum of all weights.
fn forced_or_weighted(forced: f32, weight: f32, total_free: f32, weight_sum: f32) -> f32 {
    if forced > 0.0 {
        forced
    } else if weight > 0.0 && total_free > 0.0 && weight_sum > 0.0 {
        (weight / weight_sum) * total_free
    } else {
        0.0
    }
}

fn get_stat_value_from_hashid_map(map: &HashIdMap<f32>, stat: HashId) -> f32 {
    map.get_opt(stat).copied().unwrap_or_default().max(0.0)
}
