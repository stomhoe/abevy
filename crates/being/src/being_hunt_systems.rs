use crate::being_components::Being;
use crate::being_inst_template::being_inst_template_resources::BitRef;
use crate::being_nav::AiNavGrids;
use crate::body::{Body, BodySums};
use crate::race::race_components::Race;
use crate::race::race_resources::RaceRef;
use ::being_shared::*;
use bevy::{
    ecs::{
        entity::{EntityHashMap, EntityHashSet},
        entity_disabling::Disabled,
    },
    prelude::*,
};
use common::AnyDisabling;
use common::common_components::StrId;
use common::common_tag_components::TagSet;
use tilemap_shared::{CardinalDirection, GlobalTilePos};

const HUNTERS_PER_PREY_TARGET: usize = 4;
const HUNT_RETARGET_HYSTERESIS_TILES: i32 = 3;
const SNEAK_FRONT_PENALTY_TILES: i32 = 6;
const SNEAK_SIDE_PENALTY_TILES: i32 = 3;

#[allow(unused_parens, )]
pub fn add_predator_behavior_components(
    mut commands: Commands,
    query: Query<Entity, (With<Predator>, Without<Hunger>, )>,
) {
    for being_ent in query.iter() {
        commands
            .entity(being_ent)
            .try_insert((Hunger::default(), PredatorHuntThreshold::default()));
    }
}

#[allow(unused_parens, )]
pub fn sync_predator_config_from_sources(
    mut commands: Commands,
    changed_beings: Query<Entity, (With<Being>, Or<(Changed<BitRef>, Changed<RaceRef>)>, )>,
    beings: Query<(Option<&BitRef>, Option<&RaceRef>, ), (With<Being>, AnyDisabling, )>,
    bit_pred_cfg: Query<&Predator>,
    race_pred_cfg: Query<&Predator>,
    bit_cfg: Query<&PredatorHuntThreshold>,
    race_cfg: Query<&PredatorHuntThreshold>,
    mut removed_disabled: RemovedComponents<Disabled>,
) {
    let reenabled_beings = collect_reenabled_entities(&mut removed_disabled);
    let mut beings_to_sync = reenabled_beings;
    beings_to_sync.extend(changed_beings.iter());

    for being_ent in beings_to_sync {
        let Ok((bit_ref, race_ref, )) = beings.get(being_ent) else {
            continue;
        };
        let bit_predator = bit_ref.and_then(|r| bit_pred_cfg.get(r.0).ok()).cloned();
        let race_predator = race_ref.and_then(|r| race_pred_cfg.get(r.0).ok()).cloned();
        let bit_threshold = bit_ref.and_then(|r| bit_cfg.get(r.0).ok()).copied();
        let race_threshold = race_ref.and_then(|r| race_cfg.get(r.0).ok()).copied();

        let Some(chosen) = bit_threshold.or(race_threshold) else {
            continue;
        };
        let predator = bit_predator.or(race_predator).unwrap_or_default();
        commands.entity(being_ent).try_insert((predator, chosen));
    }
}

fn collect_reenabled_entities(removed_disabled: &mut RemovedComponents<Disabled>) -> EntityHashSet {
    let mut entities = EntityHashSet::default();
    entities.extend(removed_disabled.read());
    entities
}

#[allow(unused_parens, )]
pub fn tick_hunger(
    time: Res<Time>,
    mut query: Query<&mut Hunger, >,
) {
    let delta = time.delta_secs();
    if delta <= 0.0 {
        return;
    }
    for mut hunger in query.iter_mut() {
        hunger.curr = (hunger.curr + hunger.increase_per_sec * delta).clamp(0.0, hunger.max);
    }
}

fn health_ratio(
    being: Entity,
    bodies_query: &Query<&Body, (With<Being>, )>,
    body_health_query: &Query<&BodySums, >,
) -> Option<f32> {
    let Ok(body) = bodies_query.get(being) else {
        return None;
    };
    let Ok(health) = body_health_query.get(body.entity()) else {
        return None;
    };
    if health.total_hp <= 0.0 {
        return None;
    }
    Some((health.current_hp / health.total_hp).clamp(0.0, 1.0))
}

fn sneak_penalty_tiles(
    predator_pos: GlobalTilePos,
    prey_pos: GlobalTilePos,
    prey_facing: Option<CardinalDirection>,
) -> i32 {
    let Some(prey_facing) = prey_facing else {
        return 0;
    };
    let to_predator = (predator_pos.0 - prey_pos.0).as_vec2().normalize_or_zero();
    if to_predator == Vec2::ZERO {
        return 0;
    }
    let facing = prey_facing.to_dir_vec().as_vec2().normalize_or_zero();
    if facing == Vec2::ZERO {
        return 0;
    }
    let dot = facing.dot(to_predator);
    if dot >= 0.4 {
        SNEAK_FRONT_PENALTY_TILES
    } else if dot >= -0.2 {
        SNEAK_SIDE_PENALTY_TILES
    } else {
        0
    }
}

#[allow(unused_parens, )]
pub fn update_predator_hunting_targets(
    bodies_query: Query<&Body, (With<Being>, )>,
    body_health_query: Query<&BodySums, >,
    predators: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Predator,
            Option<&RaceRef>,
            Option<&BodyTreeWeightSum>,
            Option<&ComputedBy>,
            Option<&SquadMemberOf>,
            Option<&Hunting>,
            Has<PredatorDetectedByPrey>,
            &Hunger,
            &PredatorHuntThreshold,
        ),
        (With<Predator>, ),
    >,
    prey_query: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&RaceRef>,
            Option<&BodyTreeWeightSum>,
            Option<&TagSet>,
            Option<&SquadMemberOf>,
            Option<&CardinalDirection>,
        ),
        (With<Being>, ),
    >,
    race_str_id_query: Query<&StrId, (With<Race>, )>,
    squads_query: Query<&SquadMembers, >,
    grids: Res<AiNavGrids>,
    mut valid_prey_positions: Local<EntityHashMap<GlobalTilePos>>,
    mut valid_prey_dims: Local<EntityHashMap<::tilemap_shared::DimensionRef>>,
    mut valid_prey_race: Local<EntityHashMap<Option<RaceRef>>>,
    mut valid_prey_weight: Local<EntityHashMap<f32>>,
    mut valid_prey_tags: Local<EntityHashMap<Option<TagSet>>>,
    mut valid_prey_squad: Local<EntityHashMap<Option<Entity>>>,
    mut valid_prey_facing: Local<EntityHashMap<Option<CardinalDirection>>>,
    mut cmd: Commands,
) {
    valid_prey_positions.clear();
    valid_prey_dims.clear();
    valid_prey_race.clear();
    valid_prey_weight.clear();
    valid_prey_tags.clear();
    valid_prey_squad.clear();
    valid_prey_facing.clear();
    for (prey_ent, prey_gpos, &prey_dim, prey_race, prey_weight_sum, prey_tags, prey_member_of, prey_facing, ) in prey_query.iter() {
        valid_prey_positions.insert(prey_ent, *prey_gpos);
        valid_prey_dims.insert(prey_ent, prey_dim);
        valid_prey_race.insert(prey_ent, prey_race.copied());
        valid_prey_weight.insert(prey_ent, prey_weight_sum.map(|sum| sum.0).unwrap_or_default());
        valid_prey_tags.insert(prey_ent, prey_tags.cloned());
        valid_prey_squad.insert(prey_ent, prey_member_of.map(|member_of| member_of.squad));
        valid_prey_facing.insert(prey_ent, prey_facing.copied());
    }

    let mut predator_squads: EntityHashMap<Vec<(Entity, GlobalTilePos, ::tilemap_shared::DimensionRef)>> =
        EntityHashMap::default();
    let mut predators_without_squad = Vec::new();
    for (
        pred_ent,
        pred_gpos,
        &pred_dim,
        predator_cfg,
        pred_race,
        pred_weight_sum,
        controlled_by,
        pred_member_of,
        hunting,
        predator_detected,
        hunger,
        hunt_threshold,
    ) in predators.iter()
    {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_dc_input {
                cmd.entity(pred_ent).try_remove::<Hunting>();
                continue;
            }
        }

        let hp = health_ratio(pred_ent, &bodies_query, &body_health_query);
        if hunger.curr < hunt_threshold.0 || hp.is_some_and(|hp| hp <= 0.9) {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        }
        if pred_member_of.is_some() && (predator_cfg.pack_size_min > 1 || predator_cfg.pack_size_max > 1) {
            let squad = pred_member_of.map_or(pred_ent, |member_of| member_of.squad);
            predator_squads
                .entry(squad)
                .or_default()
                .push((pred_ent, *pred_gpos, pred_dim));
            continue;
        }
        predators_without_squad.push((
            pred_ent,
            *pred_gpos,
            pred_dim,
            predator_cfg.clone(),
            pred_race.copied(),
            pred_weight_sum.map(|sum| sum.0).unwrap_or_default(),
            hunting.map(|hunting| hunting.prey),
            predator_detected,
        ));
    }

    for (pred_ent, pred_pos, pred_dim, predator_cfg, pred_race, pred_weight_newtons, current_prey, predator_detected, ) in predators_without_squad.into_iter() {
        let mut closest: Option<(Entity, i32)> = None;
        for (&prey_ent, prey_pos) in valid_prey_positions.iter() {
            let Some(&prey_dim) = valid_prey_dims.get(&prey_ent) else {
                continue;
            };
            if prey_ent == pred_ent || prey_dim != pred_dim {
                continue;
            }
            let prey_tags = valid_prey_tags.get(&prey_ent).and_then(|tags| tags.as_ref());
            if let Some(prey_tags) = prey_tags {
                if predator_cfg.do_not_hunt_tags.intersects(prey_tags) {
                    continue;
                }
            }
            let prey_weight_newtons = valid_prey_weight.get(&prey_ent).copied().unwrap_or_default();
            if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                && pred_weight_newtons > 0.0
                && prey_weight_newtons > pred_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
            {
                continue;
            }
            let prey_race = valid_prey_race.get(&prey_ent).copied().flatten();
            if let Some(prey_race) = prey_race {
                if let Ok(prey_race_id) = race_str_id_query.get(prey_race.0) {
                    if predator_cfg.own_races.contains(prey_race_id) {
                        continue;
                    }
                }
            }
            if let (Some(pred_race), Some(prey_race)) = (pred_race, prey_race) {
                if pred_race.0 == prey_race.0 {
                    continue;
                }
            }
            let delta = prey_pos.0 - pred_pos.0;
            let mut manhattan = delta.x.abs() + delta.y.abs();
            if !predator_detected {
                manhattan += sneak_penalty_tiles(
                    pred_pos,
                    *prey_pos,
                    valid_prey_facing.get(&prey_ent).copied().flatten(),
                );
            }
            let Some((_, curr_best)) = closest else {
                closest = Some((prey_ent, manhattan));
                continue;
            };
            if manhattan < curr_best {
                closest = Some((prey_ent, manhattan));
            }
        }

        let Some((closest_target, closest_dist)) = closest else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };
        let mut chosen_target = closest_target;
        if let Some(current_prey) = current_prey {
            if let (Some(current_pos), Some(&current_dim)) = (
                valid_prey_positions.get(&current_prey).copied(),
                valid_prey_dims.get(&current_prey),
            ) {
                if current_dim == pred_dim && grids.can_pathfind_between(pred_pos, current_pos, pred_dim) {
                    let current_dist = (current_pos.0 - pred_pos.0).abs().x + (current_pos.0 - pred_pos.0).abs().y;
                    if current_dist <= closest_dist + HUNT_RETARGET_HYSTERESIS_TILES {
                        chosen_target = current_prey;
                    }
                }
            }
        }
        cmd.entity(pred_ent).try_insert(Hunting { prey: chosen_target, });
    }

    for (_predator_squad, squad_preds, ) in predator_squads.iter() {
        if squad_preds.is_empty() {
            continue;
        }
        let pred_dim = squad_preds[0].2;
        let mut anchor_sum = IVec2::ZERO;
        for (_, gpos, _, ) in squad_preds.iter() {
            anchor_sum += gpos.0;
        }
        let anchor_pos = GlobalTilePos(anchor_sum / squad_preds.len() as i32);
        let mut prey_candidates: Vec<(Entity, i32)> = Vec::new();
        prey_candidates.reserve(valid_prey_positions.len());
        for (&prey_ent, prey_pos) in valid_prey_positions.iter() {
            let Some(&prey_dim) = valid_prey_dims.get(&prey_ent) else {
                continue;
            };
            if prey_dim != pred_dim {
                continue;
            }
            let dist = (prey_pos.0 - anchor_pos.0).abs().x + (prey_pos.0 - anchor_pos.0).abs().y;
            prey_candidates.push((prey_ent, dist));
        }
        prey_candidates.sort_by_key(|(_, dist)| *dist);

        let Some(&(closest_base_target, closest_base_dist)) = prey_candidates.first() else {
            for (pred_ent, _, _, ) in squad_preds.iter() {
                cmd.entity(*pred_ent).try_remove::<Hunting>();
            }
            continue;
        };
        let mut base_target = closest_base_target;
        if let Some((_, pred_pos, pred_dim, )) = squad_preds.first() {
            let mut current_target_counts: EntityHashMap<u32> = EntityHashMap::default();
            for (pred_ent, _, _, ) in squad_preds.iter() {
                let Ok((_pred_ent, _pred_gpos, _pred_dim, _predator_cfg, _pred_race, _pred_weight_sum, _controlled_by, _pred_member_of, hunting, _predator_detected, _hunger, _hunt_threshold)) = predators.get(*pred_ent) else {
                    continue;
                };
                let Some(hunting) = hunting else {
                    continue;
                };
                let Some(current_pos) = valid_prey_positions.get(&hunting.prey).copied() else {
                    continue;
                };
                let Some(&current_dim) = valid_prey_dims.get(&hunting.prey) else {
                    continue;
                };
                if current_dim != *pred_dim || !grids.can_pathfind_between(*pred_pos, current_pos, *pred_dim) {
                    continue;
                }
                *current_target_counts.entry(hunting.prey).or_insert(0) += 1;
            }
            if let Some((&committed_target, _)) = current_target_counts.iter().max_by_key(|(_, count)| **count) {
                if let Some(committed_pos) = valid_prey_positions.get(&committed_target).copied() {
                    let committed_dist = (committed_pos.0 - anchor_pos.0).abs().x + (committed_pos.0 - anchor_pos.0).abs().y;
                    if committed_dist <= closest_base_dist + HUNT_RETARGET_HYSTERESIS_TILES {
                        base_target = committed_target;
                    }
                }
            }
        }

        let mut target_list: Vec<Entity> = Vec::new();
        if let Some(Some(target_squad)) = valid_prey_squad.get(&base_target).copied() {
            let Ok(target_squad_members) = squads_query.get(target_squad) else {
                target_list.push(base_target);
                for (pred_ent, pred_pos, pred_dim, ) in squad_preds.iter() {
                    let Some(prey_pos) = valid_prey_positions.get(&base_target).copied() else {
                        continue;
                    };
                    if !grids.can_pathfind_between(*pred_pos, prey_pos, *pred_dim) {
                        continue;
                    }
                    cmd.entity(*pred_ent).try_insert(Hunting { prey: base_target, });
                }
                continue;
            };

            let mut reachable_squad_targets: Vec<(Entity, i32)> = Vec::new();
            for target_member in target_squad_members.iter() {
                let Some(member_pos) = valid_prey_positions.get(&target_member).copied() else {
                    continue;
                };
                let Some(&member_dim) = valid_prey_dims.get(&target_member) else {
                    continue;
                };
                if member_dim != pred_dim {
                    continue;
                }
                let mut reachable = false;
                for (_pred_ent, pred_pos, pred_dim, ) in squad_preds.iter() {
                    if !grids.can_pathfind_between(*pred_pos, member_pos, *pred_dim) {
                        continue;
                    }
                    reachable = true;
                    break;
                }
                if !reachable {
                    continue;
                }
                let dist = (member_pos.0 - anchor_pos.0).abs().x + (member_pos.0 - anchor_pos.0).abs().y;
                reachable_squad_targets.push((target_member, dist));
            }
            reachable_squad_targets.sort_by_key(|(_, dist)| *dist);
            target_list.reserve(reachable_squad_targets.len());
            target_list.extend(reachable_squad_targets.into_iter().map(|(target, _)| target));
            if target_list.is_empty() {
                target_list.push(base_target);
            }
        } else {
            target_list.push(base_target);
        }

        for (ix, (pred_ent, pred_pos, pred_dim, )) in squad_preds.iter().enumerate() {
            let mut target_ix = (ix / HUNTERS_PER_PREY_TARGET).min(target_list.len().saturating_sub(1));
            let mut assigned_target = None;
            while target_ix < target_list.len() {
                let target = target_list[target_ix];
                let Some(target_pos) = valid_prey_positions.get(&target).copied() else {
                    target_ix += 1;
                    continue;
                };
                if !grids.can_pathfind_between(*pred_pos, target_pos, *pred_dim) {
                    target_ix += 1;
                    continue;
                }
                assigned_target = Some(target);
                break;
            }
            let Some(assigned_target) = assigned_target else {
                cmd.entity(*pred_ent).try_remove::<Hunting>();
                continue;
            };
            cmd.entity(*pred_ent).try_insert(Hunting { prey: assigned_target, });
        }
    }
}

#[allow(unused_parens, )]
pub fn sync_hunting_to_chasing(
    mut cmd: Commands,
    hunting_predators: Query<(Entity, &Hunting, Option<&Chasing>, ), (With<Predator>, )>,
) {
    for (pred_ent, hunting, chasing, ) in hunting_predators.iter() {
        if chasing
            .map(|chasing| chasing.target == hunting.prey)
            .unwrap_or(false)
        {
            continue;
        }
        cmd.entity(pred_ent).try_insert(Chasing::new(hunting.prey, 1.5));
    }
}

#[allow(unused_parens, )]
pub fn clear_predator_detected_when_not_hunting(
    mut commands: Commands,
    query: Query<(Entity, Has<Hunting>, Has<PredatorDetectedByPrey>, ), (With<Predator>, )>,
) {
    for (pred_ent, has_hunting, has_detected, ) in query.iter() {
        if has_hunting || !has_detected {
            continue;
        }
        commands.entity(pred_ent).try_remove::<PredatorDetectedByPrey>();
    }
}
