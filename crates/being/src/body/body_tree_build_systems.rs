use being_shared::BeingInstTemplate;
use bevy::prelude::*;
use game_common::game_common_samplers::SpriteGlobalNormalDistResult;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use common::common_components::{HashId, HashIdMap};
use modifier::modifier_components::{AppliedModifiers, ApplyMode, BaseValue, CurrEffectiveValue, ModifierTarget};
use modifier::modifier_types::*;
use crate::being_components::*;
use crate::body::BodyTreeRef;
use crate::body::{body_tree_components::*, body_part::body_part_components::*};
use crate::race::race_components::Race;
use crate::race::race_resources::RaceRef;

pub fn build_body_tree(
    mut cmd: Commands,
    query: Query<
        (Entity, &BodyTreeRef, Option<&RaceRef>, Option<&SpriteGlobalNormalDistResult>),
        (With<Being>, Added<BodyTreeRef>, Without<EntityZero>, Without<Race>, Without<BeingInstTemplate>),
    >,
    tree_mass_query: Query<&BodyTreeMassKg, With<BodyTree>>,
    tree_totals_query: Query<&BodyTreeDistributedTotals, With<BodyTree>>,
    race_mass_query: Query<&BodyTreeMassKg, With<Race>>,
    race_totals_query: Query<&BodyTreeDistributedTotals, With<Race>>,
    forced_query: Query<&BodyPartForcedDistribution>,
    weighted_query: Query<&BodyPartWeightedDistribution>,
    root_parts_query: Query<(Entity, &BodyPartOf), (With<BodyRootPart>, With<EntityZero>)>,
    toclone_query: Query<(&EntityZeroRef, &BodyPartOf, Option<&BodyPartChildren>)>,
    children_query: Query<&Children>,
    modifier_target_query: Query<&ModifierTarget>,
) {
    for (being_ent, tree_to_build, race_ref, global_size) in query.iter() {
        let body_ent = cmd.spawn((
            BodyOf { being: being_ent },
            ChildOf(being_ent),
        )).id();

        let tree_mass = race_ref
            .and_then(|r| race_mass_query.get(r.0).ok().map(|m| m.0))
            .or_else(|| tree_mass_query.get(tree_to_build.0).ok().map(|m| m.0))
            .unwrap_or(0.0);
        let size_mult = global_size.map(|s| s.0).unwrap_or(1.0).max(0.01);
        let total_mass = tree_mass * size_mult;
        let totals = race_ref
            .and_then(|r| race_totals_query.get(r.0).ok().cloned())
            .or_else(|| tree_totals_query.get(tree_to_build.0).ok().cloned())
            .unwrap_or_default();

        let mut cloned_parts = Vec::new();

        let mut root_template_ent = None;
        for (part_ent, body_of) in root_parts_query.iter() {
            if body_of.body == tree_to_build.0 {
                root_template_ent = Some(part_ent);
                break;
            }
        }
        let Some(root_template_ent) = root_template_ent else {
            warn!(target: "body_build", "BodyTree {:?} has no BodyRootPart; skipping clone for being {:?}", tree_to_build.0, being_ent);
            cmd.entity(being_ent).try_insert(BeingMassKg(total_mass));
            continue;
        };

        if let Some(new_root_ent) = walk_and_clone_tree(
            &mut cmd,
            root_template_ent,
            &toclone_query,
            &children_query,
            None,
            &modifier_target_query,
            body_ent,
            being_ent,
            &mut cloned_parts,
        ) {
            cmd.entity(new_root_ent)
                .try_insert((BodyPartOf { body: body_ent }, ChildOf(body_ent)))
                .try_remove::<BodyPartParent>();
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
    }
}

fn walk_and_clone_tree(
    cmd: &mut Commands,
    ezerotree_curr_node_ent: Entity,
    ref_of_bpart_toclone_query: &Query<(&EntityZeroRef, &BodyPartOf, Option<&BodyPartChildren>)>,
    bodypart_modifiers_query: &Query<&Children>,
    parent_cloned_ent: Option<Entity>,
    modifier_target_query: &Query<&ModifierTarget>,
    body_ent: Entity,
    being_ent: Entity,
    cloned_parts: &mut Vec<(Entity, Entity)>,
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
            builder.deny::<Children>();
            builder.deny::<AppliedModifiers>();
            builder.deny::<BodyPartForcedDistribution>();
            builder.deny::<BodyPartWeightedDistribution>();
        })
        .id();
    cmd.entity(cloned_bodypart_ent)
        .try_insert(BodyPartOf { body: body_ent });
    cloned_parts.push((cloned_bodypart_ent, bodypart_2b_cloned_ent));

    if let Ok(children) = bodypart_modifiers_query.get(bodypart_2b_cloned_ent) {
        for modifier_ent in children.iter() {
            let Ok(modifier_target) = modifier_target_query.get(modifier_ent) else {
                continue;
            };

            let cloned_modifier_ent = cmd.entity(modifier_ent)
                .clone_and_spawn_with_opt_out(|builder| {
                    builder.deny::<(EntityZero, ModifierTarget)>();
                })
                .try_insert(ChildOf(cloned_bodypart_ent))
                .id();
            let target = if modifier_target.0 == bodypart_2b_cloned_ent {
                cloned_bodypart_ent
            } else {
                being_ent
            };
            cmd.entity(cloned_modifier_ent).try_insert(ModifierTarget(target));
        }
    }

    if let Some(parent_cloned) = parent_cloned_ent {
        cmd.entity(cloned_bodypart_ent).insert((
            BodyPartParent {
                parent: parent_cloned,
            },
            ChildOf(parent_cloned),
        ));
    } else {
        cmd.entity(cloned_bodypart_ent).try_remove::<BodyPartParent>();
        cmd.entity(cloned_bodypart_ent).try_insert(ChildOf(body_ent));
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
                body_ent,
                being_ent,
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

fn stat_from_hashid_map(map: &HashIdMap<f32>, stat: HashId) -> f32 {
    map.get_opt(stat).copied().unwrap_or(0.0).max(0.0)
}

fn apply_distributions(
    cmd: &mut Commands,
    parts: &[(Entity, Entity)],
    being_ent: Entity,
    total_mass: f32,
    totals: BodyTreeDistributedTotals,
    forced_query: &Query<&BodyPartForcedDistribution>,
    weighted_query: &Query<&BodyPartWeightedDistribution>,
) {
    let empty_stats = HashIdMap::default();
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

    for &(_, source_part) in parts {
        let forced = forced_query.get(source_part).map(|x| &x.0).unwrap_or(&empty_stats);
        let weights = weighted_query.get(source_part).map(|x| &x.0).unwrap_or(&empty_stats);
        let mass = stat_from_hashid_map(&forced, STAT_MASS_KG);
        if mass > 0.0 { forced_mass += mass; } else { sum_w_mass += stat_from_hashid_map(&weights, STAT_MASS_KG); }
        let hp = stat_from_hashid_map(&forced, STAT_HP_CAPACITY);
        if hp > 0.0 { forced_hp += hp; } else { sum_w_hp += stat_from_hashid_map(&weights, STAT_HP_CAPACITY); }
        let regen = stat_from_hashid_map(&forced, STAT_HP_REGEN_RATE);
        if regen > 0.0 { forced_regen += regen; } else { sum_w_regen += stat_from_hashid_map(&weights, STAT_HP_REGEN_RATE); }
        let blood = stat_from_hashid_map(&forced, STAT_BLOOD_CAPACITY);
        if blood > 0.0 { forced_blood += blood; } else { sum_w_blood += stat_from_hashid_map(&weights, STAT_BLOOD_CAPACITY); }
        let walk = stat_from_hashid_map(&forced, STAT_WALK_SPEED);
        if walk > 0.0 { forced_walk += walk; } else { sum_w_walk += stat_from_hashid_map(&weights, STAT_WALK_SPEED); }
        let swim = stat_from_hashid_map(&forced, STAT_SWIM_SPEED);
        if swim > 0.0 { forced_swim += swim; } else { sum_w_swim += stat_from_hashid_map(&weights, STAT_SWIM_SPEED); }
        let fly = stat_from_hashid_map(&forced, STAT_FLY_SPEED);
        if fly > 0.0 { forced_fly += fly; } else { sum_w_fly += stat_from_hashid_map(&weights, STAT_FLY_SPEED); }
        let manip = stat_from_hashid_map(&forced, STAT_MANIPULATION);
        if manip > 0.0 { forced_manip += manip; } else { sum_w_manip += stat_from_hashid_map(&weights, STAT_MANIPULATION); }
        let vision = stat_from_hashid_map(&forced, STAT_VISION);
        if vision > 0.0 { forced_vision += vision; } else { sum_w_vision += stat_from_hashid_map(&weights, STAT_VISION); }
        let pain = stat_from_hashid_map(&forced, STAT_PAIN_SENSITIVITY);
        if pain > 0.0 { forced_pain += pain; } else { sum_w_pain += stat_from_hashid_map(&weights, STAT_PAIN_SENSITIVITY); }
    }

    let free_mass = (total_mass - forced_mass).max(0.0);
    let free_hp = (stat_from_hashid_map(&totals.0, STAT_HP_CAPACITY) - forced_hp).max(0.0);
    let free_regen = (stat_from_hashid_map(&totals.0, STAT_HP_REGEN_RATE) - forced_regen).max(0.0);
    let free_blood = (stat_from_hashid_map(&totals.0, STAT_BLOOD_CAPACITY) - forced_blood).max(0.0);
    let free_walk = (stat_from_hashid_map(&totals.0, STAT_WALK_SPEED) - forced_walk).max(0.0);
    let free_swim = (stat_from_hashid_map(&totals.0, STAT_SWIM_SPEED) - forced_swim).max(0.0);
    let free_fly = (stat_from_hashid_map(&totals.0, STAT_FLY_SPEED) - forced_fly).max(0.0);
    let free_manip = (stat_from_hashid_map(&totals.0, STAT_MANIPULATION) - forced_manip).max(0.0);
    let free_vision = (stat_from_hashid_map(&totals.0, STAT_VISION) - forced_vision).max(0.0);
    let free_pain = (stat_from_hashid_map(&totals.0, STAT_PAIN_SENSITIVITY) - forced_pain).max(0.0);

    for &(part, source_part) in parts {
        let forced = forced_query.get(source_part).map(|x| &x.0).unwrap_or(&empty_stats);
        let weights = weighted_query.get(source_part).map(|x| &x.0).unwrap_or(&empty_stats);

        let mass_kg = weighted_or_forced(
            stat_from_hashid_map(&forced, STAT_MASS_KG),
            stat_from_hashid_map(&weights, STAT_MASS_KG),
            free_mass,
            sum_w_mass,
        );
        cmd.entity(part).try_insert(BodyPartMassResolvedKgs(mass_kg));

        let forced_hp_capacity = stat_from_hashid_map(&forced, STAT_HP_CAPACITY);
        let hp = weighted_or_forced(forced_hp_capacity, stat_from_hashid_map(&weights, STAT_HP_CAPACITY), free_hp, sum_w_hp);
        if forced_hp_capacity <= 0.0 && hp > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(hp), CurrEffectiveValue(hp), ApplyMode::Add, HitpointsCapacity, ChildOf(part)));
            cmd.entity(part).try_insert(BodyPartDamage(0.0));
        }
        let forced_regen_rate = stat_from_hashid_map(&forced, STAT_HP_REGEN_RATE);
        let regen = weighted_or_forced(forced_regen_rate, stat_from_hashid_map(&weights, STAT_HP_REGEN_RATE), free_regen, sum_w_regen);
        if forced_regen_rate <= 0.0 && regen > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(regen), CurrEffectiveValue(regen), ApplyMode::Add, HitpointRegenRate, ChildOf(part)));
        }
        let forced_blood_capacity = stat_from_hashid_map(&forced, STAT_BLOOD_CAPACITY);
        let blood = weighted_or_forced(forced_blood_capacity, stat_from_hashid_map(&weights, STAT_BLOOD_CAPACITY), free_blood, sum_w_blood);
        if forced_blood_capacity <= 0.0 && blood > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(blood), CurrEffectiveValue(blood), ApplyMode::Add, BloodCapacity, ChildOf(part)));
        }
        let forced_walk_speed = stat_from_hashid_map(&forced, STAT_WALK_SPEED);
        let walk = weighted_or_forced(forced_walk_speed, stat_from_hashid_map(&weights, STAT_WALK_SPEED), free_walk, sum_w_walk);
        if walk > 0.0 {
            cmd.spawn((ModifierTarget(being_ent), BaseValue(walk), CurrEffectiveValue(walk), ApplyMode::Add, WalkSpeed, ChildOf(part)));
        }
        let forced_swim_speed = stat_from_hashid_map(&forced, STAT_SWIM_SPEED);
        let swim = weighted_or_forced(forced_swim_speed, stat_from_hashid_map(&weights, STAT_SWIM_SPEED), free_swim, sum_w_swim);
        if forced_swim_speed <= 0.0 && swim > 0.0 {
            cmd.spawn((ModifierTarget(being_ent), BaseValue(swim), CurrEffectiveValue(swim), ApplyMode::Add, SwimSpeed, ChildOf(part)));
        }
        let forced_fly_speed = stat_from_hashid_map(&forced, STAT_FLY_SPEED);
        let fly = weighted_or_forced(forced_fly_speed, stat_from_hashid_map(&weights, STAT_FLY_SPEED), free_fly, sum_w_fly);
        if forced_fly_speed <= 0.0 && fly > 0.0 {
            cmd.spawn((ModifierTarget(being_ent), BaseValue(fly), CurrEffectiveValue(fly), ApplyMode::Add, FlySpeed, ChildOf(part)));
        }
        let forced_manipulation = stat_from_hashid_map(&forced, STAT_MANIPULATION);
        let manip = weighted_or_forced(forced_manipulation, stat_from_hashid_map(&weights, STAT_MANIPULATION), free_manip, sum_w_manip);
        if forced_manipulation <= 0.0 && manip > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(manip), CurrEffectiveValue(manip), ApplyMode::Add, Manipulation, ChildOf(part)));
        }
        let forced_vision = stat_from_hashid_map(&forced, STAT_VISION);
        let vision = weighted_or_forced(forced_vision, stat_from_hashid_map(&weights, STAT_VISION), free_vision, sum_w_vision);
        if forced_vision <= 0.0 && vision > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(vision), CurrEffectiveValue(vision), ApplyMode::Add, Vision, ChildOf(part)));
        }
        let forced_pain_sensitivity = stat_from_hashid_map(&forced, STAT_PAIN_SENSITIVITY);
        let pain = weighted_or_forced(forced_pain_sensitivity, stat_from_hashid_map(&weights, STAT_PAIN_SENSITIVITY), free_pain, sum_w_pain);
        if forced_pain_sensitivity <= 0.0 && pain > 0.0 {
            cmd.spawn((ModifierTarget(part), BaseValue(pain), CurrEffectiveValue(pain), ApplyMode::Add, PainSensitivity, ChildOf(part)));
        }
    }
}
