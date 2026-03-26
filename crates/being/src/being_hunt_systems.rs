
use crate::being_nav::AiNavGrids;
use crate::body::{HeldBody, BodySums};
use ::being_shared::*;
use bevy::{
    ecs::{
        entity::{EntityHashMap, EntityHashSet},
    },
    prelude::*,
};
use common::common_tag_components::TagSet;
use tilemap_shared::{CardinalDirection, GlobalTilePos};

const HUNTERS_PER_PREY_TARGET: usize = 4;
const HUNT_RETARGET_HYSTERESIS_TILES: i32 = 3;
const SNEAK_FRONT_PENALTY_TILES: i32 = 6;
const SNEAK_SIDE_PENALTY_TILES: i32 = 3;


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
    bodies_query: &Query<&HeldBody, >,
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

fn resolve_predator_cfg(
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_cfg_query: &Query<&PredatorCfg>,
) -> Option<PredatorCfg> {
    bit_ref
        .and_then(|bit_ref| bit_cfg_query.get(bit_ref.0).ok())
        .or_else(|| race_ref.and_then(|race_ref| bit_cfg_query.get(race_ref.0).ok()))
        .cloned()
}

#[allow(unused_parens, )]
pub fn sync_predator_squad_marker(
    mut cmd: Commands,
    changed_predators: Query<&SquadMemberOf, (Or<(Added<Predator>, Changed<SquadMemberOf>)>, With<Being>, )>,
    mut removed_predator: RemovedComponents<Predator>,
    squad_members_query: Query<&SquadMembers, >,
    member_predator_query: Query<Has<Predator>, (With<Being>, )>,
    squad_member_of_query: Query<&SquadMemberOf, >,
    mut squads_to_process: Local<EntityHashSet>,
) {
    squads_to_process.clear();
    for member_of in changed_predators.iter() {
        squads_to_process.insert(member_of.0);
    }
    for pred_ent in removed_predator.read() {
        let Ok(member_of) = squad_member_of_query.get(pred_ent) else {
            continue;
        };
        squads_to_process.insert(member_of.0);
    }

    for squad_ent in squads_to_process.drain() {
        let Ok(squad_members) = squad_members_query.get(squad_ent) else {
            continue;
        };
        let has_predator = squad_members.iter().any(|member_ent| member_predator_query.get(member_ent).is_ok_and(|has_predator| has_predator));
        if has_predator {
            cmd.entity(squad_ent).try_insert_if_new(Predator);
        } else {
            cmd.entity(squad_ent).try_remove::<Predator>();
        }
    }
}

#[allow(unused_parens, )]
/// DON'T ALTER THIS SIGNATURE
pub fn update_predator_hunting_targets(
    mut cmd: Commands,
    bodies_query: Query<&HeldBody, >,
    body_health_query: Query<&BodySums, >,
    predators: Query<
        (
            Entity,
            Option<&SquadMemberOf>,
            Option<&Hunting>,
            Has<PredatorDetectedByPrey>,
            &Hunger,
        ),
        (With<Predator>, LocalAiControlled),
    >,
    gpos_query: Query<&GlobalTilePos, >,
    dim_query: Query<&::tilemap_shared::DimensionRef, >,
    body_tree_weight_query: Query<&BodyTreeWeightSum, >,
    tagset_query: Query<&TagSet, >,
    squad_member_of_query: Query<&SquadMemberOf, >,
    squad_members_query: Query<(&SquadMembers), (With<Predator>, )>,
    card_dir_query: Query<&CardinalDirection, >,
    bit_ref_query: Query<&BitRef, ()>,
    race_ref_query: Query<&RaceRef, ()>,
    predator_cfg_query: Query<&PredatorCfg, >,

    grids: Res<AiNavGrids>,
) {
    for (pred_ent, squad_member_of, hunting, predator_detected, hunger) in predators.iter() {
        if squad_member_of.is_some() {
            continue;
        }
        let bit_ref = bit_ref_query.get(pred_ent).ok();
        let race_ref = race_ref_query.get(pred_ent).ok();
        let Some(predator_cfg) = resolve_predator_cfg(bit_ref, race_ref, &predator_cfg_query) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };
        let hp = health_ratio(pred_ent, &bodies_query, &body_health_query);
        if hunger.curr < predator_cfg.min_hunger_to_hunt || hp.is_some_and(|hp| hp <= predator_cfg.min_hp_ratio_to_hunt) {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        }

        let Ok(&pred_pos) = gpos_query.get(pred_ent) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };
        let Ok(&pred_dim) = dim_query.get(pred_ent) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };        let pred_weight_newtons = body_tree_weight_query.get(pred_ent).map(|sum| sum.0).unwrap_or_default();

        let mut closest: Option<(Entity, i32)> = None;
        for (prey_ent, _, _, _, _) in predators.iter() {
            if prey_ent == pred_ent {
                continue;
            }
            let Ok(&prey_pos) = gpos_query.get(prey_ent) else {
                continue;
            };
            let Ok(&prey_dim) = dim_query.get(prey_ent) else {
                continue;
            };
            if prey_dim != pred_dim {
                continue;
            }
            if let Ok(prey_tags) = tagset_query.get(prey_ent) {
                if predator_cfg.do_not_hunt_tags.intersects(prey_tags) {
                    continue;
                }
            }

            let prey_weight_newtons = body_tree_weight_query.get(prey_ent).map(|sum| sum.0).unwrap_or_default();
            if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                && pred_weight_newtons > 0.0
                && prey_weight_newtons > pred_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
            {
                continue;
            }

            let delta = prey_pos.0 - pred_pos.0;
            let mut manhattan = delta.x.abs() + delta.y.abs();
            if !predator_detected {
                manhattan += sneak_penalty_tiles(pred_pos, prey_pos, card_dir_query.get(prey_ent).ok().copied());
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
        if let Some(current_prey) = hunting.map(|hunting| hunting.prey) {
            if let (Ok(&current_pos), Ok(&current_dim)) = (gpos_query.get(current_prey), dim_query.get(current_prey)) {
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

    for (squad_members) in squad_members_query.iter() {
        if squad_members.iter().next().is_none() {
            continue;
        }

        let mut squad_dim = None;
        let mut anchor_sum = IVec2::ZERO;
        let mut active_member_count = 0;
        let mut active_member_ents = Vec::new();
        for member_ent in squad_members.iter() {
            let Ok((_, _, hunting, controlled_by, hunger)) = predators.get(member_ent) else {
                continue;
            };
            if controlled_by {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            }

            let member_bit_ref = bit_ref_query.get(member_ent).ok();
            let member_race_ref = race_ref_query.get(member_ent).ok();
            let Some(predator_cfg) = resolve_predator_cfg(member_bit_ref, member_race_ref, &predator_cfg_query) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            let hp = health_ratio(member_ent, &bodies_query, &body_health_query);
            if hunger.curr < predator_cfg.min_hunger_to_hunt || hp.is_some_and(|hp| hp <= predator_cfg.min_hp_ratio_to_hunt) {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            }

            let Ok(&member_pos) = gpos_query.get(member_ent) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            let Ok(&member_dim) = dim_query.get(member_ent) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            if let Some(squad_dim) = squad_dim {
                if squad_dim != member_dim {
                    continue;
                }
            } else {
                squad_dim = Some(member_dim);
            }

            anchor_sum += member_pos.0;
            active_member_count += 1;
            active_member_ents.push((member_ent, member_pos, member_dim, hunting.map(|hunting| hunting.prey)));
        }

        let Some(squad_dim) = squad_dim else {
            continue;
        };
        if active_member_count == 0 {
            continue;
        }
        let anchor_pos = GlobalTilePos(anchor_sum / active_member_count as i32);

        let mut committed_counts: EntityHashMap<u32> = EntityHashMap::default();
        for (_, member_pos, member_dim, current_prey) in active_member_ents.iter().copied() {
            let Some(current_prey) = current_prey else {
                continue;
            };
            let Ok(&current_pos) = gpos_query.get(current_prey) else {
                continue;
            };
            let Ok(&current_dim) = dim_query.get(current_prey) else {
                continue;
            };
            if current_dim != member_dim || !grids.can_pathfind_between(member_pos, current_pos, member_dim) {
                continue;
            }
            *committed_counts.entry(current_prey).or_insert(0) += 1;
        }

        let mut base_target = None;
        let mut best_base_dist = i32::MAX;
        for (prey_ent, _, _, _, _) in predators.iter() {
            if squad_members.iter().any(|member_ent| member_ent == prey_ent) {
                continue;
            }
            let Ok(&prey_pos) = gpos_query.get(prey_ent) else {
                continue;
            };
            let Ok(&prey_dim) = dim_query.get(prey_ent) else {
                continue;
            };
            if prey_dim != squad_dim {
                continue;
            }
            let dist = (prey_pos.0 - anchor_pos.0).abs().x + (prey_pos.0 - anchor_pos.0).abs().y;
            if dist < best_base_dist {
                best_base_dist = dist;
                base_target = Some(prey_ent);
            }
        }

        let Some(mut base_target) = base_target else {
            for member_ent in squad_members.iter() {
                cmd.entity(member_ent).try_remove::<Hunting>();
            }
            continue;
        };

        if let Some((&committed_target, _)) = committed_counts.iter().max_by_key(|(_, count)| **count) {
            if let Ok(&committed_pos) = gpos_query.get(committed_target) {
                let committed_dist = (committed_pos.0 - anchor_pos.0).abs().x + (committed_pos.0 - anchor_pos.0).abs().y;
                if committed_dist <= best_base_dist + HUNT_RETARGET_HYSTERESIS_TILES {
                    base_target = committed_target;
                }
            }
        }

        let mut target_list = Vec::new();
        if let Ok(target_member_of) = squad_member_of_query.get(base_target) {
            if let Ok((target_squad_members)) = squad_members_query.get(target_member_of.0) {
                let mut reachable_squad_targets: Vec<(Entity, i32)> = Vec::new();
                for target_member in target_squad_members.iter() {
                    if squad_members.iter().any(|member_ent| member_ent == target_member) {
                        continue;
                    }
                    let Ok(&member_pos) = gpos_query.get(target_member) else {
                        continue;
                    };
                    let Ok(&member_dim) = dim_query.get(target_member) else {
                        continue;
                    };
                    if member_dim != squad_dim {
                        continue;
                    }

                    let mut reachable = false;
                    for member_ent in squad_members.iter() {
                        let Ok(&pred_pos) = gpos_query.get(member_ent) else {
                            continue;
                        };
                        let Ok(&pred_dim) = dim_query.get(member_ent) else {
                            continue;
                        };
                        if !grids.can_pathfind_between(pred_pos, member_pos, pred_dim) {
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
                if reachable_squad_targets.is_empty() {
                    target_list.push(base_target);
                } else {
                    target_list.extend(reachable_squad_targets.into_iter().map(|(target, _)| target));
                }
            } else {
                target_list.push(base_target);
            }
        } else {
            target_list.push(base_target);
        }

        for (ix, member_ent) in squad_members.iter().enumerate() {
            let Ok(&member_pos) = gpos_query.get(member_ent) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            let Ok(&member_dim) = dim_query.get(member_ent) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            let mut target_ix = (ix / HUNTERS_PER_PREY_TARGET).min(target_list.len().saturating_sub(1));
            let mut assigned_target = None;
            while target_ix < target_list.len() {
                let target = target_list[target_ix];
                let Some(target_pos) = gpos_query.get(target).ok().copied() else {
                    target_ix += 1;
                    continue;
                };
                if !grids.can_pathfind_between(member_pos, target_pos, member_dim) {
                    target_ix += 1;
                    continue;
                }
                assigned_target = Some(target);
                break;
            }
            let Some(assigned_target) = assigned_target else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            cmd.entity(member_ent).try_insert(Hunting { prey: assigned_target, });
        }
    }
}

#[allow(unused_parens, )]
pub fn sync_chasing_to_hunt(
    mut cmd: Commands,
    hunting_predators: Query<(Entity, &Hunting, Option<&Chasing>, ), (Changed<Hunting>)>,
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
    query: Query<(Entity, ), (Without<Hunting>, With<PredatorDetectedByPrey>)>,
) {
    for (pred_ent, ) in query.iter() {

        commands.entity(pred_ent).try_remove::<PredatorDetectedByPrey>();
    }
}
