#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use common::common_id_components::{HashId, HashIdMap};
use common::log_targets::BODY_BUILD;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use modifier_shared::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierSynergies, ModifierTarget};
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_types::*;
use ::being_shared::*;

use crate::body::{
    body_tree_components::*, bodypart::bodypart_resources::*,
    body_tree_resources::*,
};

const STAT_BLEED_RATE: HashId = HashId::hash("bleed_rate");

#[allow(unused_parens)]
pub fn init_ezero_body_trees(
    mut cmd: Commands,
    body_map: Res<BodyTreeEntityMap>,
    part_map: Res<BodypartEntityMap>,
) {
    if !body_map.0.is_empty() {
        return;
    }

    for mut seri in load_body_tree_seri_defs() {

        let body_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for BodyConfig: {}", e));
                error!(target: "body_init", "{}", err);
                continue;
            }
        };
        let body_tree_ent = cmd.spawn_empty().id();
        cmd.entity(body_tree_ent).insert((
            body_id.clone(),
            BodyTree,
            EntityZero,
        ));

        if seri.name.trim().is_empty() {
            cmd.entity(body_tree_ent)
                .insert(DisplayName::trunc(body_id.as_str()));
        } else {
            cmd.entity(body_tree_ent)
                .insert(DisplayName::trunc(seri.name));
        }

        if !seri.tags.is_empty() {
            cmd.entity(body_tree_ent).insert(TagSet::new(&seri.tags));
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
            totals.overwrite(BodypartStat::STAT_MASS_KG, seri.mass_kg.max(0.0));
        }
        let totals_to_distribute = StatBudgetsToDistribute(totals.clone());
        cmd.entity(body_tree_ent).insert((
            totals_to_distribute.clone(),
            BodyTreeSexes(seri.sexes.clone()),
            CaloricBurnRateMultiplier(seri.caloric_burn_rate_multiplier),
        ));

        let root_node = std::mem::take(&mut seri.root);
        let root_id = StrId::trunc(root_node.part_id.as_str());

        let root_ent = rec_build_ezero_body_tree(
            &mut cmd,
            &part_map,
            body_tree_ent,
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

#[allow(unused_parens, )]
pub fn distribute_ezero_body_tree_modifiers(
    mut cmd: Commands,
    body_map: Res<BodyTreeEntityMap>,
    body_tree_query: Query<(Entity, &StrId, &StatBudgetsToDistribute, ), (With<BodyTree>, With<EntityZero>, )>,
    ezero_tree_bodyparts_query: Query<(&BodypartChildrenBodyparts, ), (With<EntityZero>, )>,
    forced_query: Query<&BodypartForcedStats, >,
    weighted_query: Query<&BodypartWeightedDistribution, >,
    synergy_query: Query<&ModifierSynergies, >,
    ezero_bodypart_refs_query: Query<&EntityZeroRef, (With<EntityZero>, )>,
    mut ezeros_mapped_to_bodyparts: Local<Vec<(Entity, Entity)>>,
) {
    if !body_map.0.is_empty() {
        return;
    }

    for (body_tree_ent, body_id, totals_to_distribute, ) in body_tree_query.iter() {
        let Ok((bodyparts_list, )) = ezero_tree_bodyparts_query.get(body_tree_ent) else {
            error!(target: BODY_BUILD, "BodyTree {} has no bodypart children after deferred tree build; skipping distribution", body_id);
            continue;
        };
        ezeros_mapped_to_bodyparts.clear();
        for bodypart_ent in bodyparts_list.iter() {
            let Ok(ezero_ref) = ezero_bodypart_refs_query.get(bodypart_ent) else {
                error!(target: BODY_BUILD, "BodyTree {} bodypart node {:?} is missing EntityZeroRef; skipping node", body_id, bodypart_ent);
                continue;
            };
            ezeros_mapped_to_bodyparts.push((bodypart_ent, ezero_ref.0));
        }
        if ezeros_mapped_to_bodyparts.is_empty() {
            error!(target: BODY_BUILD, "BodyTree {} produced zero ezero bodypart references; skipping distribution", body_id);
            continue;
        }
        apply_distributions(
            &mut cmd,
            body_id,
            &ezeros_mapped_to_bodyparts,
            body_tree_ent,
            stat_from_hashid_map(&totals_to_distribute.0, BodypartStat::STAT_MASS_KG),
            totals_to_distribute.clone(),
            &forced_query,
            &weighted_query,
            &synergy_query,
        );
    }
}
//dont alter this
fn rec_build_ezero_body_tree(
    cmd: &mut Commands,
    part_map: &Res<BodypartEntityMap>,
    ezero_body_ent: Entity,
    body_id: &StrId,
    node: BodypartNodeSeri,
    parent_node_ent: Option<Entity>,
) -> Option<Entity> {
    let node_bodypart_id = StrId::trunc(node.part_id.as_str());
    let Ok(source_part_ent) = part_map.0.get_cloned(&node_bodypart_id) else {
        error!(target: BODY_BUILD, "Bodypart '{}' not found in BodypartCfgEntityMap for body '{}', skipping", node_bodypart_id, body_id);
        return None;
    };

    let parent_bodypart = parent_node_ent.unwrap_or(ezero_body_ent);
    let node_ent = cmd.entity(source_part_ent).clone_and_spawn_with_opt_out(|builder| {
        builder.deny::<(
            EntityZero,
            ChildOf,
            Children,
            BodypartChildrenBodyparts,
        )>();
    }).id();
    cmd.entity(node_ent).insert((
        BodypartChildOfBodypart { parent_bodypart },
        ChildOf(ezero_body_ent),
        EntityZeroRef(source_part_ent),
        EntityZero,
        Name::default(),
    ));

    let override_label = node.label_override.trim();
    if !override_label.is_empty() {
        cmd.entity(node_ent).insert(DisplayName::trunc(override_label));
    }

    for child in node.children {
        rec_build_ezero_body_tree(cmd, part_map, ezero_body_ent, body_id, child, Some(node_ent));
    }

    Some(node_ent)
}

fn forced_or_weighted(forced: f32, weight: f32, total_free: f32, weight_sum: f32) -> f32 {
    if forced > 0.0 {
        forced
    } else if weight > 0.0 && total_free > 0.0 && weight_sum > 0.0 {
        (weight / weight_sum) * total_free
    } else {
        0.0
    }
}

fn stat_from_hashid_map(map: &HashIdMap<f32>, stat: HashId) -> f32 {
    map.get_opt(stat).copied().unwrap_or(0.0).max(0.0)
}

fn apply_distributions(
    cmd: &mut Commands,
    body_id: &StrId,
    parts: &[(Entity, Entity)],
    body_ent: Entity,
    total_mass: f32,
    totals: StatBudgetsToDistribute,
    forced_query: &Query<&BodypartForcedStats>,
    weighted_query: &Query<&BodypartWeightedDistribution>,
    synergy_query: &Query<&ModifierSynergies>,
) {
    if parts.is_empty() {
        error!(target: BODY_BUILD, "BodyTree {} has no parts to distribute stats onto", body_id);
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

    let mut forced_mass = 0.0;
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
        let mass = stat_from_hashid_map(forced, BodypartStat::STAT_MASS_KG);
        if mass > 0.0 { forced_mass += mass; } else { sum_w_mass += stat_from_hashid_map(weights, BodypartStat::STAT_MASS_KG); }
        let hp = stat_from_hashid_map(forced, BodypartStat::STAT_HP_CAPACITY);
        if hp > 0.0 { forced_hp += hp; } else { sum_w_hp += stat_from_hashid_map(weights, BodypartStat::STAT_HP_CAPACITY); }
        let regen = stat_from_hashid_map(forced, BodypartStat::STAT_HP_REGEN_RATE);
        if regen > 0.0 { forced_regen += regen; } else { sum_w_regen += stat_from_hashid_map(weights, BodypartStat::STAT_HP_REGEN_RATE); }
        let blood = stat_from_hashid_map(forced, BodypartStat::STAT_BLOOD_CAPACITY);
        if blood > 0.0 { forced_blood += blood; } else { sum_w_blood += stat_from_hashid_map(weights, BodypartStat::STAT_BLOOD_CAPACITY); }
        let walk = stat_from_hashid_map(forced, BodypartStat::STAT_WALK_SPEED);
        if walk > 0.0 { forced_walk += walk; } else { sum_w_walk += stat_from_hashid_map(weights, BodypartStat::STAT_WALK_SPEED); }
        let swim = stat_from_hashid_map(forced, BodypartStat::STAT_SWIM_SPEED);
        if swim > 0.0 { forced_swim += swim; } else { sum_w_swim += stat_from_hashid_map(weights, BodypartStat::STAT_SWIM_SPEED); }
        let fly = stat_from_hashid_map(forced, BodypartStat::STAT_FLY_SPEED);
        if fly > 0.0 { forced_fly += fly; } else { sum_w_fly += stat_from_hashid_map(weights, BodypartStat::STAT_FLY_SPEED); }
        let manip = stat_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_DEXTERITY);
        if manip > 0.0 { forced_manip += manip; } else { sum_w_manip += stat_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_DEXTERITY); }
        let manip_strength = stat_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_STRENGTH);
        if manip_strength > 0.0 { forced_manip_strength += manip_strength; } else { sum_w_manip_strength += stat_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_STRENGTH); }
        let vision = stat_from_hashid_map(forced, BodypartStat::STAT_VISION);
        if vision > 0.0 { forced_vision += vision; } else { sum_w_vision += stat_from_hashid_map(weights, BodypartStat::STAT_VISION); }
        let pain = stat_from_hashid_map(forced, BodypartStat::STAT_PAIN_SENSITIVITY);
        if pain > 0.0 { forced_pain += pain; } else { sum_w_pain += stat_from_hashid_map(weights, BodypartStat::STAT_PAIN_SENSITIVITY); }
        let bleed = stat_from_hashid_map(forced, STAT_BLEED_RATE);
        if bleed > 0.0 { forced_bleed += bleed; } else { sum_w_bleed += stat_from_hashid_map(weights, STAT_BLEED_RATE); }
    }

    let free_mass = (total_mass - forced_mass).max(0.0);
    let free_hp = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_HP_CAPACITY) - forced_hp).max(0.0);
    let free_regen = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_HP_REGEN_RATE) - forced_regen).max(0.0);
    let free_blood = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_BLOOD_CAPACITY) - forced_blood).max(0.0);
    let free_walk = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_WALK_SPEED) - forced_walk).max(0.0);
    let free_swim = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_SWIM_SPEED) - forced_swim).max(0.0);
    let free_fly = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_FLY_SPEED) - forced_fly).max(0.0);
    let free_manip = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_MANIPULATION_DEXTERITY) - forced_manip).max(0.0);
    let free_manip_strength = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_MANIPULATION_STRENGTH) - forced_manip_strength).max(0.0);
    let free_vision = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_VISION) - forced_vision).max(0.0);
    let free_pain = (stat_from_hashid_map(&totals.0, BodypartStat::STAT_PAIN_SENSITIVITY) - forced_pain).max(0.0);
    let free_bleed = (stat_from_hashid_map(&totals.0, STAT_BLEED_RATE) - forced_bleed).max(0.0);

    for &(part, source_part) in parts {
        let Ok(forced_stats) = forced_query.get(source_part) else {
            error!(target: BODY_BUILD, "BodyTree {} source part {:?} is missing BodypartForcedStats", body_id, source_part);
            continue;
        };
        let Ok(weighted_stats) = weighted_query.get(source_part) else {
            error!(target: BODY_BUILD, "BodyTree {} source part {:?} is missing BodypartWeightedDistribution", body_id, source_part);
            continue;
        };
        let forced = &forced_stats.0;
        let weights = &weighted_stats.0;
        let synergies = synergy_query.get(source_part).ok();
        if synergies.is_none() {
            debug!(target: BODY_BUILD, "BodyTree {} source part {:?} has no ModifierSynergies", body_id, source_part);
        }

        let mut spawned_modifiers = 0;
        let mass_kg = forced_or_weighted(
            stat_from_hashid_map(forced, BodypartStat::STAT_MASS_KG),
            stat_from_hashid_map(weights, BodypartStat::STAT_MASS_KG),
            free_mass,
            sum_w_mass,
        );
        if mass_kg > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(mass_kg), CurrEffectiveValue(mass_kg), ApplyMode::Add, MassKg, ChildOf(part)));
        }

        let forced_hp_capacity = stat_from_hashid_map(forced, BodypartStat::STAT_HP_CAPACITY);
        let hp = forced_or_weighted(forced_hp_capacity, stat_from_hashid_map(weights, BodypartStat::STAT_HP_CAPACITY), free_hp, sum_w_hp);
        if hp > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(hp), CurrEffectiveValue(hp), ApplyMode::Add, HitpointsCapacity, ChildOf(part)));
        }
        let forced_regen_rate = stat_from_hashid_map(forced, BodypartStat::STAT_HP_REGEN_RATE);
        let regen = forced_or_weighted(forced_regen_rate, stat_from_hashid_map(weights, BodypartStat::STAT_HP_REGEN_RATE), free_regen, sum_w_regen);
        if regen > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(regen), CurrEffectiveValue(regen), ApplyMode::Add, HitpointRegenRate, ChildOf(part)));
        }
        let forced_blood_capacity = stat_from_hashid_map(forced, BodypartStat::STAT_BLOOD_CAPACITY);
        let blood = forced_or_weighted(forced_blood_capacity, stat_from_hashid_map(weights, BodypartStat::STAT_BLOOD_CAPACITY), free_blood, sum_w_blood);
        if blood > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(blood), CurrEffectiveValue(blood), ApplyMode::Add, BloodCapacity, ChildOf(part)));
        }
        let forced_walk_speed = stat_from_hashid_map(forced, BodypartStat::STAT_WALK_SPEED);
        let walk = forced_or_weighted(forced_walk_speed, stat_from_hashid_map(weights, BodypartStat::STAT_WALK_SPEED), free_walk, sum_w_walk);
        if walk > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(walk), CurrEffectiveValue(walk), ApplyMode::Add, WalkSpeed, ChildOf(part)));
        }
        let forced_swim_speed = stat_from_hashid_map(forced, BodypartStat::STAT_SWIM_SPEED);
        let swim = forced_or_weighted(forced_swim_speed, stat_from_hashid_map(weights, BodypartStat::STAT_SWIM_SPEED), free_swim, sum_w_swim);
        if swim > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(swim), CurrEffectiveValue(swim), ApplyMode::Add, SwimSpeed, ChildOf(part)));
        }
        let forced_fly_speed = stat_from_hashid_map(forced, BodypartStat::STAT_FLY_SPEED);
        let fly = forced_or_weighted(forced_fly_speed, stat_from_hashid_map(weights, BodypartStat::STAT_FLY_SPEED), free_fly, sum_w_fly);
        if fly > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(body_ent), BaseValue(fly), CurrEffectiveValue(fly), ApplyMode::Add, FlySpeed, ChildOf(part)));
        }
        let forced_manipulation = stat_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_DEXTERITY);
        let manip = forced_or_weighted(forced_manipulation, stat_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_DEXTERITY), free_manip, sum_w_manip);
        if manip > 0.0 {
            spawned_modifiers += 1;
            let modifier_ent = cmd.spawn((ModifierTarget(part), BaseValue(manip), CurrEffectiveValue(manip), ApplyMode::Add, ManipulationDexterity, ChildOf(part))).id();
            if let Some(synergies) = synergies {
                cmd.entity(modifier_ent).insert(synergies.clone());
            }
        }
        let forced_manipulation_strength = stat_from_hashid_map(forced, BodypartStat::STAT_MANIPULATION_STRENGTH);
        let manip_strength = forced_or_weighted(forced_manipulation_strength, stat_from_hashid_map(weights, BodypartStat::STAT_MANIPULATION_STRENGTH), free_manip_strength, sum_w_manip_strength);
        if manip_strength > 0.0 {
            spawned_modifiers += 1;
            let modifier_ent = cmd.spawn((ModifierTarget(part), BaseValue(manip_strength), CurrEffectiveValue(manip_strength), ApplyMode::Add, ManipulationStrength, ChildOf(part))).id();
            if let Some(synergies) = synergies {
                cmd.entity(modifier_ent).insert(synergies.clone());
            }
        }
        let forced_vision = stat_from_hashid_map(forced, BodypartStat::STAT_VISION);
        let vision = forced_or_weighted(forced_vision, stat_from_hashid_map(weights, BodypartStat::STAT_VISION), free_vision, sum_w_vision);
        if vision > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(vision), CurrEffectiveValue(vision), ApplyMode::Add, Vision, ChildOf(part)));
        }
        let forced_pain_sensitivity = stat_from_hashid_map(forced, BodypartStat::STAT_PAIN_SENSITIVITY);
        let pain = forced_or_weighted(forced_pain_sensitivity, stat_from_hashid_map(weights, BodypartStat::STAT_PAIN_SENSITIVITY), free_pain, sum_w_pain);
        if pain > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(pain), CurrEffectiveValue(pain), ApplyMode::Add, PainSensitivity, ChildOf(part)));
        }
        let forced_bleed_rate = stat_from_hashid_map(forced, STAT_BLEED_RATE);
        let bleed = forced_or_weighted(forced_bleed_rate, stat_from_hashid_map(weights, STAT_BLEED_RATE), free_bleed, sum_w_bleed);
        if bleed > 0.0 {
            spawned_modifiers += 1;
            cmd.spawn((ModifierTarget(part), BaseValue(bleed), CurrEffectiveValue(bleed), ApplyMode::Add, BleedRate, ChildOf(part)));
        }

        if spawned_modifiers == 0 {
            error!(target: BODY_BUILD, "BodyTree {} source part {:?} produced no modifiers; forced and weighted distributions may be empty", body_id, source_part);
        } else {
            debug!(target: BODY_BUILD, "BodyTree {} source part {:?} spawned {} modifiers", body_id, source_part, spawned_modifiers);
        }
    }
}
