#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use common::common_id_components::{HashId, HashIdMap};
use common::log_targets::BODY_BUILD;
use game_common::game_common_components::{Templ, TemplEntiRef};
use modifier_shared::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierSynergies, ModifierTarget};
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_types::*;
use ::being_shared::*;
use crate::being_interaction_zone_helper::build_being_interaction_zones;

use crate::body::body_hp_systems::UserBodypartInstances;
use crate::body::{
    body_components::*, bodypart::bodypart_resources::*,
    body_resources::*,
};

const STAT_BLEED_RATE: HashId = HashId::hash("bleed_rate");

#[allow(unused_parens)]
pub fn init_templ_bodys(
    mut cmd: Commands,
    body_map: Res<BodyEntityMap>,
    part_map: Res<BodypartEntityMap>,
) {
    if !body_map.0.is_empty() {
        return;
    }
    for mut seri in load_body_seri_defs() {

        let body_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for BodyConfig: {}", e));
                error!(target: "body_init", "{}", err);
                continue;
            }
        };
        let body_ent = cmd.spawn_empty().id();
        cmd.entity(body_ent).insert((
            body_id.clone(),
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
        if !totals.contains_key(BodypartStat::STAT_WALK_SPEED) {
            totals.overwrite(BodypartStat::STAT_WALK_SPEED, 300.);
        }
        if !totals.contains_key(BodypartStat::STAT_MASS_KG) {
            error!(target: BODY_BUILD, "Body '{}' is missing distributed_totals.mass_kg; skipping", body_id);
            continue;
        }
        let totals_to_distribute = StatBudgetsToDistributeAmongBodyPartsOfTemplBody(totals.clone());
        cmd.entity(body_ent).insert((
            totals_to_distribute.clone(),
            BodySexes(seri.sexes.clone()),
            CaloricBurnRateMultiplier(seri.caloric_burn_rate_multiplier),
        ));
        let root_node = std::mem::take(&mut seri.root);
        let root_id = StrId::trunc(root_node.part_id.as_str());

        let root_ent = rec_build_templ_body(
            &mut cmd,
            &part_map,
            body_ent,
            &body_id,
            root_node,
            None,
        );

        if let Some(root_ent) = root_ent {
            cmd.entity(root_ent).insert(TreeRoot);
        } else {
            warn!(target: "body_init", "BodyConfig '{}' root part '{}' not found", body_id, root_id);
            continue;
        }

    }
}

fn rec_build_templ_body(
    cmd: &mut Commands,
    part_map: &Res<BodypartEntityMap>,
    templ_body_ent: Entity,
    body_id: &StrId,
    node: BodypartNodeSeri,
    parent_node_ent: Option<Entity>,
) -> Option<Entity> {
    let node_bodypart_id = StrId::trunc(node.part_id.as_str());
    let Ok(source_part_ent) = part_map.0.get_cloned(&node_bodypart_id) else {
        error!(target: BODY_BUILD, "Bodypart '{}' not found in BodypartCfgEntityMap for body '{}', skipping", node_bodypart_id, body_id);
        return None;
    };

    let parent_bodypart = parent_node_ent.unwrap_or(templ_body_ent);
    let node_ent = cmd.entity(source_part_ent).clone_and_spawn_with_opt_out(|builder| {
        builder.deny::<(
            Templ,
            ChildOf,
            Children,
            BodypartChildrenBodyparts,
        )>();
    }).id();
    cmd.entity(node_ent).insert((
        BodypartChildOfBodypart { parent_bodypart },
        ChildOf(templ_body_ent),
        TemplEntiRef(source_part_ent),
        UserBodypartInstances::default(),
        Templ,
        Name::default(),
    ));

    let override_label = node.label_override.trim();
    if !override_label.is_empty() {
        cmd.entity(node_ent).insert(DisplayName::trunc(override_label));
    }

    for child in node.children {
        rec_build_templ_body(cmd, part_map, templ_body_ent, body_id, child, Some(node_ent));
    }

    Some(node_ent)
}



#[allow(unused_parens, )]
pub fn distribute_templ_body_modifiers(
    mut cmd: Commands,
    body_query: Query<(Entity, &StrId, &StatBudgetsToDistributeAmongBodyPartsOfTemplBody, &Children), (With<Body>, With<Templ>, )>,
    forced_query: Query<&BodypartForcedStats, >,
    weighted_query: Query<&BodypartWeightedDistribution, >,
    synergy_query: Query<&ModifierSynergies, >,
    templ_bodypart_refs_query: Query<&TemplEntiRef, ()>,
    mut templs_mapped_to_bodyparts: Local<Vec<(Entity, Entity)>>,
) {
    for (body_ent, body_id, budgets_to_distribute, bodyparts_list) in body_query.iter() {

        templs_mapped_to_bodyparts.clear();
        for bodypart_ent in bodyparts_list.iter() {
            let Ok(templ_ref) = templ_bodypart_refs_query.get(bodypart_ent) else {
                error!(target: BODY_BUILD, "Body {} bodypart node {:?} is missing TemplEntiRef; skipping node", body_id, bodypart_ent);
                continue;
            };
            templs_mapped_to_bodyparts.push((bodypart_ent, templ_ref.0));
        }
        if templs_mapped_to_bodyparts.is_empty() {
            error!(target: BODY_BUILD, "Body {} produced zero templ bodypart references; skipping distribution", body_id);
            continue;
        }
        distribute_budgets_among_bodyparts_based_on_weights_and_forcings(
            &mut cmd,
            body_id,
            &templs_mapped_to_bodyparts,
            body_ent,
            budgets_to_distribute,
            &forced_query,
            &weighted_query,
            &synergy_query,
        );
    }
}

fn distribute_budgets_among_bodyparts_based_on_weights_and_forcings(
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
    let mut sum_w_walk = 0.0;
    let mut sum_w_swim = 0.0;
    let mut sum_w_fly = 0.0;
    let mut sum_w_manip = 0.0;
    let mut sum_w_manip_strength = 0.0;
    let mut sum_w_vision = 0.0;
    let mut sum_w_pain = 0.0;
    let mut sum_w_bleed = 0.0;

    let mut forced_mass_kg = 0.0;
    let mut forced_hp = 0.0;
    let mut forced_regen = 0.0;
    let mut forced_blood = 0.0;
    let mut forced_walk = 0.0;
    let mut forced_swim = 0.0;
    let mut forced_fly = 0.0;
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
        let walk = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_WALK_SPEED);
        if walk > 0.0 { forced_walk += walk; } else { sum_w_walk += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_WALK_SPEED); }
        let swim = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_SWIM_SPEED);
        if swim > 0.0 { forced_swim += swim; } else { sum_w_swim += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_SWIM_SPEED); }
        let fly = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_FLY_SPEED);
        if fly > 0.0 { forced_fly += fly; } else { sum_w_fly += get_stat_value_from_hashid_map(weights, BodypartStat::STAT_FLY_SPEED); }
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
    let free_walk = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_WALK_SPEED) - forced_walk).max(0.0);
    let free_swim = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_SWIM_SPEED) - forced_swim).max(0.0);
    let free_fly = (get_stat_value_from_hashid_map(&totals.0, BodypartStat::STAT_FLY_SPEED) - forced_fly).max(0.0);
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
        let forced_walk_speed = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_WALK_SPEED);
        let walk = forced_or_weighted(forced_walk_speed, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_WALK_SPEED), free_walk, sum_w_walk);
        if walk > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(walk), CurrEffectiveValue(walk), ApplyMode::Add, WalkSpeed, ChildOf(part)));
        }
        let forced_swim_speed = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_SWIM_SPEED);
        let swim = forced_or_weighted(forced_swim_speed, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_SWIM_SPEED), free_swim, sum_w_swim);
        if swim > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(swim), CurrEffectiveValue(swim), ApplyMode::Add, SwimSpeed, ChildOf(part)));
        }
        let forced_fly_speed = get_stat_value_from_hashid_map(forced, BodypartStat::STAT_FLY_SPEED);
        let fly = forced_or_weighted(forced_fly_speed, get_stat_value_from_hashid_map(weights, BodypartStat::STAT_FLY_SPEED), free_fly, sum_w_fly);
        if fly > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(fly), CurrEffectiveValue(fly), ApplyMode::Add, FlySpeed, ChildOf(part)));
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
