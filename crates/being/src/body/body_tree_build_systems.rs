use bevy::prelude::*;
use game_common::game_common_samplers::SpriteGlobalNormalDistResult;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use modifier::modifier_components::{ModifierTarget, BaseValue, CurrEffectiveValue, ApplyMode};
use modifier::modifier_types::*;
use crate::being_components::*;
use crate::body::{body_tree_components::*, body_part::body_part_components::*};

pub fn build_body_tree(
    mut cmd: Commands,
    query: Query<
        (Entity, &BodyTreeToBuild, Option<&SpriteGlobalNormalDistResult>),
        (With<Being>, Added<BodyTreeToBuild>, Without<EntityZero>),
    >,
    tree_mass_query: Query<&BodyTreeMassKg>,
    tree_totals_query: Query<&BodyTreeDistributedTotals>,
    part_mass_weight_query: Query<&BodyPartMassWeight>,
    forced_query: Query<&BodyPartForcedDistribution>,
    weighted_query: Query<&BodyPartWeightedDistribution>,
    toclone_query: Query<(&EntityZeroRef, &BodyPartOf, Option<&BodyPartChildren>)>,
    children_query: Query<&Children>,
    modifier_target_query: Query<&ModifierTarget>,
) {
    for (being_ent, tree_to_build, global_size) in query.iter() {
        let tree_mass = tree_mass_query
            .get(tree_to_build.0)
            .map(|m| m.0)
            .unwrap_or(0.0);
        let size_mult = global_size.map(|s| s.0).unwrap_or(1.0).max(0.01);
        let total_mass = tree_mass * size_mult;
        let totals = tree_totals_query.get(tree_to_build.0).copied().unwrap_or_default();

        let mut cloned_parts = Vec::new();

        if let Some(new_root_ent) = walk_and_clone_tree(
            &mut cmd,
            tree_to_build.0,
            &toclone_query,
            &children_query,
            None,
            &modifier_target_query,
            being_ent,
            &part_mass_weight_query,
            &mut cloned_parts,
        ) {
            cmd.entity(new_root_ent)
                .try_insert(BodyPartOf { body: being_ent });
        }
        apply_distributions(
            &mut cmd,
            &cloned_parts,
            being_ent,
            total_mass,
            totals,
            &forced_query,
            &weighted_query,
        );
        cmd.entity(being_ent).try_insert(BeingMassKg(total_mass));

        cmd.entity(being_ent).remove::<BodyTreeToBuild>();
    }
}

fn walk_and_clone_tree(
    cmd: &mut Commands,
    ezerotree_curr_node_ent: Entity,
    ref_of_bpart_toclone_query: &Query<(&EntityZeroRef, &BodyPartOf, Option<&BodyPartChildren>)>,
    bodypart_modifiers_query: &Query<&Children>,
    parent_cloned_ent: Option<Entity>,
    modifier_target_query: &Query<&ModifierTarget>,
    being_ent: Entity,
    part_mass_weight_query: &Query<&BodyPartMassWeight>,
    cloned_parts: &mut Vec<Entity>,
) -> Option<Entity> {
    let Ok((ezero_ref, _ezero_body_part_of, bodypart_children)) =
        ref_of_bpart_toclone_query.get(ezerotree_curr_node_ent)
    else {
        return None;
    };
    let bodypart_2b_cloned_ent = ezero_ref.0;

    let cloned_bodypart_ent = cmd
        .entity(bodypart_2b_cloned_ent)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<EntityZero>();
        })
        .id();
    let _ = part_mass_weight_query.get(bodypart_2b_cloned_ent);
    cloned_parts.push(cloned_bodypart_ent);

    if let Ok(children) = bodypart_modifiers_query.get(bodypart_2b_cloned_ent) {
        for modifier_ent in children.iter() {
            let cloned_modifier_ent = cmd.entity(modifier_ent)
                .clone_and_spawn_with_opt_out(|builder| {
                    builder.deny::<(EntityZero, ModifierTarget)>();
                })
                .try_insert(ChildOf(cloned_bodypart_ent))
                .id();
            if let Ok(modifier_target) = modifier_target_query.get(modifier_ent) {
                let target = if modifier_target.0 == bodypart_2b_cloned_ent {
                    cloned_bodypart_ent
                } else {
                    being_ent
                };

                cmd.entity(cloned_modifier_ent).try_insert(ModifierTarget(target));
            }
        }
    }

    if let Some(parent_cloned) = parent_cloned_ent {
        cmd.entity(cloned_bodypart_ent).insert((
            BodyPartParent {
                parent: parent_cloned,
            },
            ChildOf(parent_cloned),
        ));
    }
    if let Some(bodypart_children) = bodypart_children {
        for ezero_child_bodypart_ent in bodypart_children.iter() {
            walk_and_clone_tree(
                cmd,
                ezero_child_bodypart_ent,
                ref_of_bpart_toclone_query,
                &bodypart_modifiers_query,
                Some(cloned_bodypart_ent),
                &modifier_target_query,
                being_ent,
                part_mass_weight_query,
                cloned_parts,
            );
        }
    }

    Some(cloned_bodypart_ent)
}

fn weighted_or_forced(forced: f32, weight: f32, total_free: f32, weight_sum: f32) -> f32 {
    if forced > 0.0 {
        forced
    } else if weight > 0.0 && total_free > 0.0 && weight_sum > 0.0 {
        (weight / weight_sum) * total_free
    } else {
        0.0
    }
}

fn apply_distributions(
    cmd: &mut Commands,
    parts: &[Entity],
    being_ent: Entity,
    total_mass: f32,
    totals: BodyTreeDistributedTotals,
    forced_query: &Query<&BodyPartForcedDistribution>,
    weighted_query: &Query<&BodyPartWeightedDistribution>,
) {
    let mut sum_w_mass = 0.0;
    let mut sum_w_hp = 0.0;
    let mut sum_w_regen = 0.0;
    let mut sum_w_blood = 0.0;
    let mut sum_w_walk = 0.0;
    let mut sum_w_swim = 0.0;
    let mut sum_w_fly = 0.0;
    let mut sum_w_manip = 0.0;
    let mut sum_w_vision = 0.0;
    let mut sum_w_pain = 0.0;

    let mut forced_mass = 0.0;
    let mut forced_hp = 0.0;
    let mut forced_regen = 0.0;
    let mut forced_blood = 0.0;
    let mut forced_walk = 0.0;
    let mut forced_swim = 0.0;
    let mut forced_fly = 0.0;
    let mut forced_manip = 0.0;
    let mut forced_vision = 0.0;
    let mut forced_pain = 0.0;

    for &part in parts {
        let forced = forced_query.get(part).copied().unwrap_or_default();
        let weights = weighted_query.get(part).copied().unwrap_or_default();
        if forced.mass_kg > 0.0 { forced_mass += forced.mass_kg; } else { sum_w_mass += weights.mass_kg.max(0.0); }
        if forced.hp_capacity > 0.0 { forced_hp += forced.hp_capacity; } else { sum_w_hp += weights.hp_capacity.max(0.0); }
        if forced.hp_regen_rate > 0.0 { forced_regen += forced.hp_regen_rate; } else { sum_w_regen += weights.hp_regen_rate.max(0.0); }
        if forced.blood_capacity > 0.0 { forced_blood += forced.blood_capacity; } else { sum_w_blood += weights.blood_capacity.max(0.0); }
        if forced.walk_speed > 0.0 { forced_walk += forced.walk_speed; } else { sum_w_walk += weights.walk_speed.max(0.0); }
        if forced.swim_speed > 0.0 { forced_swim += forced.swim_speed; } else { sum_w_swim += weights.swim_speed.max(0.0); }
        if forced.fly_speed > 0.0 { forced_fly += forced.fly_speed; } else { sum_w_fly += weights.fly_speed.max(0.0); }
        if forced.manipulation > 0.0 { forced_manip += forced.manipulation; } else { sum_w_manip += weights.manipulation.max(0.0); }
        if forced.vision > 0.0 { forced_vision += forced.vision; } else { sum_w_vision += weights.vision.max(0.0); }
        if forced.pain_sensitivity > 0.0 { forced_pain += forced.pain_sensitivity; } else { sum_w_pain += weights.pain_sensitivity.max(0.0); }
    }

    let free_mass = (total_mass - forced_mass).max(0.0);
    let free_hp = (totals.hp_capacity - forced_hp).max(0.0);
    let free_regen = (totals.hp_regen_rate - forced_regen).max(0.0);
    let free_blood = (totals.blood_capacity - forced_blood).max(0.0);
    let free_walk = (totals.walk_speed - forced_walk).max(0.0);
    let free_swim = (totals.swim_speed - forced_swim).max(0.0);
    let free_fly = (totals.fly_speed - forced_fly).max(0.0);
    let free_manip = (totals.manipulation - forced_manip).max(0.0);
    let free_vision = (totals.vision - forced_vision).max(0.0);
    let free_pain = (totals.pain_sensitivity - forced_pain).max(0.0);

    for &part in parts {
        let forced = forced_query.get(part).copied().unwrap_or_default();
        let weights = weighted_query.get(part).copied().unwrap_or_default();

        let mass_kg = weighted_or_forced(forced.mass_kg, weights.mass_kg, free_mass, sum_w_mass);
        cmd.entity(part).try_insert(BodyPartMassKg(mass_kg));

        let hp = weighted_or_forced(forced.hp_capacity, weights.hp_capacity, free_hp, sum_w_hp);
        if forced.hp_capacity <= 0.0 && hp > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(hp), CurrEffectiveValue(hp), ApplyMode::Add, HitpointsCapacity, ChildOf(part)));
            cmd.entity(part).try_insert(BodyPartDamage(0.0));
        }
        let regen = weighted_or_forced(forced.hp_regen_rate, weights.hp_regen_rate, free_regen, sum_w_regen);
        if forced.hp_regen_rate <= 0.0 && regen > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(regen), CurrEffectiveValue(regen), ApplyMode::Add, HitpointRegenRate, ChildOf(part)));
        }
        let blood = weighted_or_forced(forced.blood_capacity, weights.blood_capacity, free_blood, sum_w_blood);
        if forced.blood_capacity <= 0.0 && blood > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(blood), CurrEffectiveValue(blood), ApplyMode::Add, BloodCapacity, ChildOf(part)));
        }
        let walk = weighted_or_forced(forced.walk_speed, weights.walk_speed, free_walk, sum_w_walk);
        if forced.walk_speed <= 0.0 && walk > 0.0 {
            cmd.spawn((ModifierTarget(being_ent), BaseValue(walk), CurrEffectiveValue(walk), ApplyMode::Add, WalkSpeed, ChildOf(part)));
        }
        let swim = weighted_or_forced(forced.swim_speed, weights.swim_speed, free_swim, sum_w_swim);
        if forced.swim_speed <= 0.0 && swim > 0.0 {
            cmd.spawn((ModifierTarget(being_ent), BaseValue(swim), CurrEffectiveValue(swim), ApplyMode::Add, SwimSpeed, ChildOf(part)));
        }
        let fly = weighted_or_forced(forced.fly_speed, weights.fly_speed, free_fly, sum_w_fly);
        if forced.fly_speed <= 0.0 && fly > 0.0 {
            cmd.spawn((ModifierTarget(being_ent), BaseValue(fly), CurrEffectiveValue(fly), ApplyMode::Add, FlySpeed, ChildOf(part)));
        }
        let manip = weighted_or_forced(forced.manipulation, weights.manipulation, free_manip, sum_w_manip);
        if forced.manipulation <= 0.0 && manip > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(manip), CurrEffectiveValue(manip), ApplyMode::Add, Manipulation, ChildOf(part)));
        }
        let vision = weighted_or_forced(forced.vision, weights.vision, free_vision, sum_w_vision);
        if forced.vision <= 0.0 && vision > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(vision), CurrEffectiveValue(vision), ApplyMode::Add, Vision, ChildOf(part)));
        }
        let pain = weighted_or_forced(forced.pain_sensitivity, weights.pain_sensitivity, free_pain, sum_w_pain);
        if forced.pain_sensitivity <= 0.0 && pain > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(pain), CurrEffectiveValue(pain), ApplyMode::Add, PainSensitivity, ChildOf(part)));
        }
    }
}
