
use crate::being_nav::AiNavGrids;
use crate::body::{HeldBody, BodySums};
use ::being_shared::body_energy::*;
use ::being_shared::*;
use bevy::{
    ecs::{
        entity::{EntityHashMap, EntityHashSet},
        system::SystemParam,
    },
    prelude::*,
};
use common::common_tag_components::TagSet;
use tilemap_shared::DimensionEntityMap;
use tilemap_shared::GlobalTilePos;

const HUNTERS_PER_PREY_TARGET: usize = 4;
const HUNT_RETARGET_HYSTERESIS_TILES: f32 = 3.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PredatorCfgSource {
    Bit(BitRef),
    Race(RaceRef),
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

fn resolve_predator_cfg(
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
    bit_cfg_query: &Query<&PredatorCfg>,
) -> Option<(PredatorCfg, PredatorCfgSource)> {
    if let Some(bit_ref) = bit_ref
        && let Ok(bit_ent) = bit_map.0.get_cloned(bit_ref.0)
        && let Ok(cfg) = bit_cfg_query.get(bit_ent)
    {
        return Some((cfg.clone(), PredatorCfgSource::Bit(*bit_ref)));
    }
    if let Some(race_ref) = race_ref
        && let Ok(race_ent) = race_map.0.get_cloned(race_ref.0)
        && let Ok(cfg) = bit_cfg_query.get(race_ent)
    {
        return Some((cfg.clone(), PredatorCfgSource::Race(*race_ref)));
    }
    None
}

fn prey_matches_predator_cfg(
    prey_ent: Entity,
    predator_cfg: &PredatorCfg,
    predator_cfg_source: PredatorCfgSource,
    bit_ref_query: &Query<&BitRef, ()>,
    race_ref_query: &Query<&RaceRef, ()>,
    tagset_query: &Query<&TagSet, >,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
) -> bool {
    if let Ok(prey_tags) = tagset_query.get(prey_ent)
        && predator_cfg.do_not_hunt_tags.intersects(prey_tags)
    {
        return true;
    }

    if predator_cfg.do_not_hunt_same_kind {
        match predator_cfg_source {
            PredatorCfgSource::Bit(predator_bit_ref) => {
                if bit_ref_query.get(prey_ent).ok().copied() == Some(predator_bit_ref) {
                    return true;
                }
            }
            PredatorCfgSource::Race(predator_race_ref) => {
                if let Some(prey_race_ref) = race_ref_query.get(prey_ent).ok().copied()
                    && prey_race_ref == predator_race_ref
                {
                    return true;
                }
                let Some(prey_bit_ref) = bit_ref_query.get(prey_ent).ok().copied() else {
                    return false;
                };
                let Ok(prey_bit_ent) = bit_map.0.get_cloned(prey_bit_ref.0) else {
                    return false;
                };
                let Ok(prey_race_ref) = race_ref_query.get(prey_bit_ent) else {
                    return false;
                };
                if *prey_race_ref == predator_race_ref {
                    return true;
                }
            }
        }
    }

    let prey_bit_ref = bit_ref_query.get(prey_ent).ok();
    if let Some(prey_bit_ref) = prey_bit_ref
        && let Ok(prey_bit_ent) = bit_map.0.get_cloned(prey_bit_ref.0)
        && let Ok(prey_bit_tags) = tagset_query.get(prey_bit_ent)
        && predator_cfg.do_not_hunt_tags.intersects(prey_bit_tags)
    {
        return true;
    }

    let mut prey_race_ref = race_ref_query.get(prey_ent).ok().copied();
    if prey_race_ref.is_none()
        && let Some(prey_bit_ref) = prey_bit_ref
        && let Ok(prey_bit_ent) = bit_map.0.get_cloned(prey_bit_ref.0)
        && let Ok(bit_race_ref) = race_ref_query.get(prey_bit_ent)
    {
        prey_race_ref = Some(*bit_race_ref);
    }
    if let Some(prey_race_ref) = prey_race_ref
        && let Ok(prey_race_ent) = race_map.0.get_cloned(prey_race_ref.0)
        && let Ok(prey_race_tags) = tagset_query.get(prey_race_ent)
        && predator_cfg.do_not_hunt_tags.intersects(prey_race_tags)
    {
        return true;
    }

    false
}

fn euclidean_dist(a: GlobalTilePos, b: GlobalTilePos) -> f32 {
    (a.0 - b.0).as_vec2().length()
}

#[allow(unused_parens, )]
pub fn update_squad_weight_sum(
    mut cmd: Commands,
    mut squads_query: Query<(Entity, &'static SquadMembers, Option<&'static mut SquadWeightSum>, ), (With<Predator>, )>,
    body_weight_query: Query<&'static BodyWeightSum, >,
) {
    for (squad_ent, squad_members, squad_weight_sum, ) in squads_query.iter_mut() {
        let mut total_weight = 0.0;
        for member_ent in squad_members.iter() {
            total_weight += body_weight_query.get(member_ent).map(|sum| sum.0).unwrap_or_default();
        }
        if let Some(mut squad_weight_sum) = squad_weight_sum {
            squad_weight_sum.0 = total_weight;
        } else {
            cmd.entity(squad_ent).try_insert(SquadWeightSum(total_weight));
        }
    }
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
#[derive(SystemParam)]
pub struct UpdatePredatorHuntingTargetsQueries<'w, 's> {
    pub bodies_query: Query<'w, 's, &'static HeldBody, >,
    pub body_health_query: Query<'w, 's, &'static BodySums, >,
    pub predators: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static SquadMemberOf>,
            Option<&'static Hunting>,
            &'static BodyCondition,
        ),
        (With<Predator>, LocalAiControlled),
    >,
    pub beings_query: Query<'w, 's, Entity, (With<Being>, )>,
    pub pos_dim_query: Query<'w, 's, (&'static ::tilemap_shared::DimensionRef, &'static GlobalTilePos), >,
    pub dim_map: Res<'w, DimensionEntityMap>,
    pub bit_map: Res<'w, BeingInstTemplateEntityMap>,
    pub race_map: Res<'w, RaceEntityMap>,
    pub body_weight_query: Query<'w, 's, &'static BodyWeightSum, >,
    pub squad_weight_query: Query<'w, 's, &'static SquadWeightSum, (With<Predator>, )>,
    pub tagset_query: Query<'w, 's, &'static TagSet, >,
    pub squad_member_of_query: Query<'w, 's, &'static SquadMemberOf, >,
    pub squad_members_query: Query<'w, 's, (Entity, &'static SquadMembers, ), (With<Predator>, )>,
    pub bit_ref_query: Query<'w, 's, &'static BitRef, ()>,
    pub race_ref_query: Query<'w, 's, &'static RaceRef, ()>,
    pub predator_cfg_query: Query<'w, 's, &'static PredatorCfg, >,
}

#[derive(SystemParam)]
pub struct UpdatePredatorHuntingTargetsLocals<'s> {
    active_member_ents: Local<'s, Vec<(Entity, GlobalTilePos, ::tilemap_shared::DimensionRef, Option<Entity>)>>,
    target_list: Local<'s, Vec<Entity>>,
    reachable_squad_targets: Local<'s, Vec<(Entity, f32)>>,
}
#[allow(unused_parens, )]
pub fn update_predator_hunting_targets(
    mut cmd: Commands,
    params: UpdatePredatorHuntingTargetsQueries,
    grids: Res<AiNavGrids>,
    mut locals: UpdatePredatorHuntingTargetsLocals,
) {
    let resolve_dim_ent = |dim_ref: &::tilemap_shared::DimensionRef| params.dim_map.0.get_opt(dim_ref.0).copied();
    for (pred_ent, squad_member_of, hunting, body_condition) in params.predators.iter() {
        if squad_member_of.is_some() {
            continue;
        }
        let bit_ref = params.bit_ref_query.get(pred_ent).ok();
        let race_ref = params.race_ref_query.get(pred_ent).ok();
        let Some((predator_cfg, predator_cfg_source)) = resolve_predator_cfg(bit_ref, race_ref, &params.bit_map, &params.race_map, &params.predator_cfg_query) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };
        if let Some(hunting) = hunting.filter(|hunting| hunting.retaliating) {
            let Ok((pred_dim, pred_pos)) = params.pos_dim_query.get(pred_ent) else {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                continue;
            };
            let current_prey = hunting.prey;
            let Some(pred_dim_ent) = resolve_dim_ent(pred_dim) else {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                continue;
            };
            let Some((current_prey_dim, current_prey_pos)) = params.pos_dim_query.get(current_prey).ok() else {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                continue;
            };
            if *current_prey_dim != *pred_dim
                || !grids.can_pathfind_between(*pred_pos, *current_prey_pos, pred_dim_ent)
                || euclidean_dist(*pred_pos, *current_prey_pos) >= hunting.retaliation_stop_distance_tiles.max(0.0)
            {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
            }
            continue;
        }
        let hp = health_ratio(pred_ent, &params.bodies_query, &params.body_health_query);
        let hunger_ratio = body_condition.hunger_ratio.max(0.0);
        let min_hunger_ratio_to_hunt = predator_cfg.min_hunger_to_hunt.clamp(0.0, 1.0);
        if hunger_ratio < min_hunger_ratio_to_hunt || hp.is_some_and(|hp| hp <= predator_cfg.min_hp_ratio_to_hunt) {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        }

        let Ok((pred_dim, pred_pos)) = params.pos_dim_query.get(pred_ent) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };
        let pred_weight_newtons = params.body_weight_query.get(pred_ent).map(|sum| sum.0).unwrap_or_default();

        let mut closest: Option<(Entity, f32)> = None;
        for prey_ent in params.beings_query.iter() {
            if prey_ent == pred_ent {
                continue;
            }
            let Ok((prey_dim, prey_pos)) = params.pos_dim_query.get(prey_ent) else {
                continue;
            };
            if prey_dim != pred_dim {
                continue;
            }
            if prey_matches_predator_cfg(
                prey_ent,
                &predator_cfg,
                predator_cfg_source,
                &params.bit_ref_query,
                &params.race_ref_query,
                &params.tagset_query,
                &params.bit_map,
                &params.race_map,
            ) {
                continue;
            }

            let prey_weight_newtons = params.body_weight_query.get(prey_ent).map(|sum| sum.0).unwrap_or_default();
            if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                && pred_weight_newtons > 0.0
                && prey_weight_newtons > pred_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
            {
                continue;
            }

            let dist = euclidean_dist(*prey_pos, *pred_pos);
            let Some((_, curr_best)) = closest else {
                closest = Some((prey_ent, dist));
                continue;
            };
            if dist < curr_best {
                closest = Some((prey_ent, dist));
            }
        }

        let Some((closest_target, closest_dist)) = closest else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            continue;
        };
        let mut chosen_target = closest_target;
        if let Some(current_prey) = hunting.map(|hunting| hunting.prey) {
            if let Ok((current_dim, current_pos)) = params.pos_dim_query.get(current_prey) {
                let Some(pred_dim_ent) = resolve_dim_ent(pred_dim) else {
                    cmd.entity(pred_ent).try_remove::<Hunting>();
                    continue;
                };
                let Some(current_dim_ent) = resolve_dim_ent(current_dim) else {
                    continue;
                };
                if current_dim_ent == pred_dim_ent && grids.can_pathfind_between(*pred_pos, *current_pos, pred_dim_ent) {
                    let current_dist = euclidean_dist(*current_pos, *pred_pos);
                    if current_dist <= closest_dist + HUNT_RETARGET_HYSTERESIS_TILES {
                        chosen_target = current_prey;
                    }
                }
            }
        }
        cmd.entity(pred_ent).try_insert(Hunting::new(chosen_target));
    }

    for (squad_ent, squad_members, ) in params.squad_members_query.iter() {
        if squad_members.iter().next().is_none() {
            continue;
        }

        let squad_weight_newtons = params.squad_weight_query.get(squad_ent).map(|sum| sum.0).unwrap_or_default();

        locals.active_member_ents.clear();
        locals.target_list.clear();
        locals.reachable_squad_targets.clear();
        let mut squad_dim = None;
        let mut anchor_sum = IVec2::ZERO;
        let mut active_member_count = 0;
        let mut pack_predator_cfg: Option<PredatorCfg> = None;
        let mut pack_predator_cfg_source: Option<PredatorCfgSource> = None;
        for member_ent in squad_members.iter() {
            let Ok((_, _, hunting, body_condition)) = params.predators.get(member_ent) else {
                continue;
            };

            let member_bit_ref = params.bit_ref_query.get(member_ent).ok();
            let member_race_ref = params.race_ref_query.get(member_ent).ok();
            let Some((predator_cfg, predator_cfg_source)) = resolve_predator_cfg(member_bit_ref, member_race_ref, &params.bit_map, &params.race_map, &params.predator_cfg_query) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            if pack_predator_cfg.is_none() {
                pack_predator_cfg = Some(predator_cfg.clone());
                pack_predator_cfg_source = Some(predator_cfg_source);
            }
            let hp = health_ratio(member_ent, &params.bodies_query, &params.body_health_query);
            let hunger_ratio = body_condition.hunger_ratio.max(0.0);
            let min_hunger_ratio_to_hunt = predator_cfg.min_hunger_to_hunt.clamp(0.0, 1.0);
            if hunger_ratio < min_hunger_ratio_to_hunt || hp.is_some_and(|hp| hp <= predator_cfg.min_hp_ratio_to_hunt) {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            }

            let Ok((member_dim, member_pos)) = params.pos_dim_query.get(member_ent) else {
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
            locals.active_member_ents.push((member_ent, *member_pos, *member_dim, hunting.map(|hunting| hunting.prey)));
        }

        let Some(squad_dim) = squad_dim else {
            continue;
        };
        if active_member_count == 0 {
            continue;
        }
        let anchor_pos = GlobalTilePos(anchor_sum / active_member_count as i32);

        let mut committed_counts: EntityHashMap<u32> = EntityHashMap::default();
        for (_, member_pos, member_dim, current_prey) in locals.active_member_ents.iter().copied() {
            let Some(current_prey) = current_prey else {
                continue;
            };
            let Ok((current_dim, current_pos)) = params.pos_dim_query.get(current_prey) else {
                continue;
            };
            let Some(member_dim_ent) = resolve_dim_ent(&member_dim) else {
                continue;
            };
            let Some(current_dim_ent) = resolve_dim_ent(current_dim) else {
                continue;
            };
            if current_dim_ent != member_dim_ent || !grids.can_pathfind_between(member_pos, *current_pos, member_dim_ent) {
                continue;
            }
            *committed_counts.entry(current_prey).or_insert(0) += 1;
        }

        let mut base_target = None;
        let mut best_base_dist = f32::MAX;
        for prey_ent in params.beings_query.iter() {
            if squad_members.iter().any(|member_ent| member_ent == prey_ent) {
                continue;
            }
            let Ok((prey_dim, prey_pos)) = params.pos_dim_query.get(prey_ent) else {
                continue;
            };
            if *prey_dim != *squad_dim {
                continue;
            }
            if let Some(predator_cfg) = pack_predator_cfg.as_ref()
                && let Some(predator_cfg_source) = pack_predator_cfg_source
                && prey_matches_predator_cfg(
                    prey_ent,
                    predator_cfg,
                    predator_cfg_source,
                    &params.bit_ref_query,
                    &params.race_ref_query,
                    &params.tagset_query,
                    &params.bit_map,
                    &params.race_map,
                )
            {
                continue;
            }
            if let Some(predator_cfg) = pack_predator_cfg.as_ref() {
                let prey_weight_newtons = params.body_weight_query.get(prey_ent).map(|sum| sum.0).unwrap_or_default();
                if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                    && squad_weight_newtons > 0.0
                    && prey_weight_newtons > squad_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
                {
                    continue;
                }
            }
            let dist = euclidean_dist(*prey_pos, anchor_pos);
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
            if let Ok((_, committed_pos)) = params.pos_dim_query.get(committed_target) {
                let committed_dist = euclidean_dist(*committed_pos, anchor_pos);
                if committed_dist <= best_base_dist + HUNT_RETARGET_HYSTERESIS_TILES {
                    base_target = committed_target;
                }
            }
        }

            if let Ok(target_member_of) = params.squad_member_of_query.get(base_target) {
                if let Ok((_, target_squad_members, )) = params.squad_members_query.get(target_member_of.0) {
                    for target_member in target_squad_members.iter() {
                    if squad_members.iter().any(|member_ent| member_ent == target_member) {
                        continue;
                    }
                    let Ok((member_dim, member_pos)) = params.pos_dim_query.get(target_member) else {
                        continue;
                    };
                    if *member_dim != *squad_dim {
                        continue;
                    }

                    let mut reachable = false;
                    for member_ent in squad_members.iter() {
                        let Ok((pred_dim, pred_pos)) = params.pos_dim_query.get(member_ent) else {
                            continue;
                        };
                        let Some(pred_dim_ent) = resolve_dim_ent(pred_dim) else {
                            continue;
                        };
                        let Some(member_dim_ent) = resolve_dim_ent(member_dim) else {
                            continue;
                        };
                        if !grids.can_pathfind_between(*pred_pos, *member_pos, pred_dim_ent) {
                            continue;
                        }
                        if pred_dim_ent != member_dim_ent {
                            continue;
                        }
                        reachable = true;
                        break;
                    }
                    if !reachable {
                        continue;
                    }
                    let dist = euclidean_dist(*member_pos, anchor_pos);
                    locals.reachable_squad_targets.push((target_member, dist));
                }
                locals.reachable_squad_targets.sort_by(|(_, a), (_, b)| a.total_cmp(b));
                if locals.reachable_squad_targets.is_empty() {
                    locals.target_list.push(base_target);
                } else {
                    locals.target_list.extend(locals.reachable_squad_targets.iter().copied().map(|(target, _)| target));
                }
            } else {
                locals.target_list.push(base_target);
            }
        } else {
            locals.target_list.push(base_target);
        }

        for (ix, member_ent) in squad_members.iter().enumerate() {
            let Ok((member_dim, member_pos)) = params.pos_dim_query.get(member_ent) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            let Some(member_dim_ent) = resolve_dim_ent(member_dim) else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            };
            let mut target_ix = (ix / HUNTERS_PER_PREY_TARGET).min(locals.target_list.len().saturating_sub(1));
            let mut assigned_target = None;
            while target_ix < locals.target_list.len() {
                let target = locals.target_list[target_ix];
                let Some((_, target_pos)) = params.pos_dim_query.get(target).ok() else {
                    target_ix += 1;
                    continue;
                };
                if !grids.can_pathfind_between(*member_pos, *target_pos, member_dim_ent) {
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
            cmd.entity(member_ent).try_insert(Hunting::new(assigned_target));
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
