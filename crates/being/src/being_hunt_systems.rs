
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

use crate::being_nav::being_nav_debug::*;

const NAV_SYSTEM: &str = "being_hunt_systems";

const HUNTERS_PER_PREY_TARGET: usize = 4;
const HUNT_RETARGET_HYSTERESIS_TILES: f32 = 3.0;
const PACK_HUNT_NEARBY_RADIUS_TILES: f32 = 100.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PredatorCfgSource {
    Being,
    Bit(BitRef),
    Race(RaceRef),
}

#[derive(Clone)]
struct HuntTargetCandidate {
    ent: Entity,
    pos: GlobalTilePos,
    bit_ref: Option<BitRef>,
    race_ref: Option<RaceRef>,
    tags: Option<TagSet>,
    bit_tags: Option<TagSet>,
    race_tags: Option<TagSet>,
    weight: f32,
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
    source_ent: Entity,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
    bit_cfg_query: &Query<&PredatorCfg>,
) -> Option<(PredatorCfg, PredatorCfgSource)> {
    if let Ok(cfg) = bit_cfg_query.get(source_ent) {
        return Some((cfg.clone(), PredatorCfgSource::Being));
    }
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

fn prey_matches_predator_cfg_cached(
    prey: &HuntTargetCandidate,
    predator_cfg: &PredatorCfg,
    predator_cfg_source: PredatorCfgSource,
) -> bool {
    if let Some(prey_tags) = prey.tags.as_ref()
        && predator_cfg.do_not_hunt_tags.intersects(prey_tags)
    {
        return true;
    }

    if predator_cfg.do_not_hunt_same_kind {
        match predator_cfg_source {
            PredatorCfgSource::Being => {}
            PredatorCfgSource::Bit(predator_bit_ref) => {
                if prey.bit_ref == Some(predator_bit_ref) {
                    return true;
                }
            }
            PredatorCfgSource::Race(predator_race_ref) => {
                if prey.race_ref == Some(predator_race_ref) {
                    return true;
                }
            }
        }
    }

    if let Some(prey_bit_tags) = prey.bit_tags.as_ref()
        && predator_cfg.do_not_hunt_tags.intersects(prey_bit_tags)
    {
        return true;
    }

    if let Some(prey_race_tags) = prey.race_tags.as_ref()
        && predator_cfg.do_not_hunt_tags.intersects(prey_race_tags)
    {
        return true;
    }

    false
}

fn find_hunt_candidate<'a>(
    candidates: &'a [HuntTargetCandidate],
    ent: Entity,
) -> Option<&'a HuntTargetCandidate> {
    candidates.iter().find(|candidate| candidate.ent == ent)
}

fn euclidean_dist(a: GlobalTilePos, b: GlobalTilePos) -> f32 {
    (a.0 - b.0).as_vec2().length()
}

fn nearby_squad_body_weight(
    center_pos: GlobalTilePos,
    center_dim: &::tilemap_shared::DimensionRef,
    squad_members: &SquadMembers,
    pos_dim_query: &Query<(&::tilemap_shared::DimensionRef, &GlobalTilePos), >,
    body_weight_query: &Query<&BodyWeightSum, >,
) -> f32 {
    let mut total_weight = 0.0;
    for member_ent in squad_members.iter() {
        let Ok((member_dim, member_pos)) = pos_dim_query.get(member_ent) else {
            continue;
        };
        if *member_dim != *center_dim {
            continue;
        }
        if euclidean_dist(*member_pos, center_pos) > PACK_HUNT_NEARBY_RADIUS_TILES {
            continue;
        }
        total_weight += body_weight_query.get(member_ent).map(|sum| sum.0).unwrap_or_default();
    }
    total_weight
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
    mut nav_log: BeingNavDebugLog,
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
        nav_log.push(
            squad_ent,
            NAV_SYSTEM,
            BeingNavDebugKind::State,
            if has_predator {
                "Added Predator marker to squad"
            } else {
                "Removed Predator marker from squad"
            },
            vec![
                BeingNavDebugField::new("member_count", squad_members.iter().count() as u32),
                BeingNavDebugField::new("has_predator", has_predator),
            ],
        );
    }
    nav_log.flush();
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
            Option<&'static PredatorCfg>,
        ),
        (Or<(With<Predator>, With<PredatorCfg>)>, LocalAiControlled),
    >,
    pub beings_query: Query<'w, 's, Entity, (With<Being>, )>,
    pub pos_dim_query: Query<'w, 's, (&'static ::tilemap_shared::DimensionRef, &'static GlobalTilePos), >,
    pub dim_map: Res<'w, DimensionEntityMap>,
    pub bit_map: Res<'w, BeingInstTemplateEntityMap>,
    pub race_map: Res<'w, RaceEntityMap>,
    pub body_weight_query: Query<'w, 's, &'static BodyWeightSum, >,
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
    candidates_by_dim: Local<'s, EntityHashMap<Vec<HuntTargetCandidate>>>,
    target_list: Local<'s, Vec<Entity>>,
    reachable_squad_targets: Local<'s, Vec<(Entity, f32)>>,
}
#[allow(unused_parens, )]
pub fn update_predator_hunting_targets(
    mut cmd: Commands,
    params: UpdatePredatorHuntingTargetsQueries,
    grids: Res<AiNavGrids>,
    mut locals: UpdatePredatorHuntingTargetsLocals,
    mut nav_log: BeingNavDebugLog,
) {
    let resolve_dim_ent = |dim_ref: &::tilemap_shared::DimensionRef| params.dim_map.0.get_opt(dim_ref.0).copied();
    locals.candidates_by_dim.clear();
    let beings_iter = params.beings_query.iter();
    let beings_count = beings_iter.size_hint().1.unwrap_or(beings_iter.size_hint().0);
    locals.candidates_by_dim.reserve(beings_count);
    for prey_ent in params.beings_query.iter() {
        let Ok((prey_dim, prey_pos)) = params.pos_dim_query.get(prey_ent) else {
            continue;
        };
        let Some(prey_dim_ent) = resolve_dim_ent(prey_dim) else {
            continue;
        };
        let bit_ref = params.bit_ref_query.get(prey_ent).ok().copied();
        let race_ref = params.race_ref_query.get(prey_ent).ok().copied();
        let tags = params.tagset_query.get(prey_ent).ok().cloned();
        let bit_tags = bit_ref
            .and_then(|bit_ref| params.bit_map.0.get_cloned(bit_ref.0).ok())
            .and_then(|bit_ent| params.tagset_query.get(bit_ent).ok().cloned());
        let race_tags = race_ref
            .and_then(|race_ref| params.race_map.0.get_cloned(race_ref.0).ok())
            .and_then(|race_ent| params.tagset_query.get(race_ent).ok().cloned());
        let weight = params.body_weight_query.get(prey_ent).map(|sum| sum.0).unwrap_or_default();
        locals
            .candidates_by_dim
            .entry(prey_dim_ent)
            .or_default()
            .push(HuntTargetCandidate {
                ent: prey_ent,
                pos: *prey_pos,
                bit_ref,
                race_ref,
                tags,
                bit_tags,
                race_tags,
                weight,
            });
    }
    for (pred_ent, squad_member_of, hunting, body_condition, direct_predator_cfg) in params.predators.iter() {
        if squad_member_of.is_some() && direct_predator_cfg.is_none() {
            continue;
        }
        let bit_ref = params.bit_ref_query.get(pred_ent).ok();
        let race_ref = params.race_ref_query.get(pred_ent).ok();
        let Some((predator_cfg, predator_cfg_source)) = direct_predator_cfg
            .map(|predator_cfg| (predator_cfg.clone(), PredatorCfgSource::Being))
            .or_else(|| resolve_predator_cfg(pred_ent, bit_ref, race_ref, &params.bit_map, &params.race_map, &params.predator_cfg_query))
        else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Clear,
                "Removed hunting: missing predator cfg",
                vec![
                    BeingNavDebugField::new("entity", pred_ent),
                    BeingNavDebugField::new("hunting", hunting.map(|hunting| hunting.prey)),
                ],
            );
            continue;
        };
        if let Some(hunting) = hunting.filter(|hunting| hunting.retaliating) {
            let Ok((pred_dim, pred_pos)) = params.pos_dim_query.get(pred_ent) else {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                nav_log.push(
                    pred_ent,
                    NAV_SYSTEM,
                    BeingNavDebugKind::Clear,
                    "Removed retaliating hunting: missing position",
                    vec![
                        BeingNavDebugField::new("hunting", hunting.prey),
                    ],
                );
                continue;
            };
            let current_prey = hunting.prey;
            let Some(_pred_dim_ent) = resolve_dim_ent(pred_dim) else {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                nav_log.push(
                    pred_ent,
                    NAV_SYSTEM,
                    BeingNavDebugKind::Clear,
                    "Removed retaliating hunting: missing dimension",
                    vec![
                        BeingNavDebugField::new("hunting", current_prey),
                    ],
                );
                continue;
            };
            let Some((current_prey_dim, current_prey_pos)) = params.pos_dim_query.get(current_prey).ok() else {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                nav_log.push(
                    pred_ent,
                    NAV_SYSTEM,
                    BeingNavDebugKind::Clear,
                    "Removed retaliating hunting: prey position missing",
                    vec![
                        BeingNavDebugField::new("hunting", current_prey),
                    ],
                );
                continue;
            };
            if *current_prey_dim != *pred_dim
                || !grids.can_pathfind_between(*pred_pos, *current_prey_pos, *pred_dim)
                || euclidean_dist(*pred_pos, *current_prey_pos) >= hunting.retaliation_stop_distance_tiles.max(0.0)
            {
                cmd.entity(pred_ent).try_remove::<(Hunting, Chasing)>();
                nav_log.push(
                    pred_ent,
                    NAV_SYSTEM,
                    BeingNavDebugKind::Clear,
                    "Removed retaliating hunting: prey no longer reachable",
                    vec![
                        BeingNavDebugField::new("hunting", current_prey),
                        BeingNavDebugField::new("pred_dim", format!("{:?}", pred_dim.0)),
                        BeingNavDebugField::new("pred_gpos", *pred_pos),
                        BeingNavDebugField::new("prey_dim", format!("{:?}", current_prey_dim.0)),
                        BeingNavDebugField::new("prey_gpos", *current_prey_pos),
                    ],
                );
            }
            continue;
        }
        let hp = health_ratio(pred_ent, &params.bodies_query, &params.body_health_query);
        let hunger_ratio = body_condition.hunger_ratio.max(0.0);
        let min_hunger_ratio_to_hunt = predator_cfg.min_hunger_to_hunt.clamp(0.0, 1.0);
        if hunger_ratio < min_hunger_ratio_to_hunt || hp.is_some_and(|hp| hp <= predator_cfg.min_hp_ratio_to_hunt) {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Clear,
                "Removed hunting: hunger or health too low",
                vec![
                    BeingNavDebugField::new("hunger_ratio", hunger_ratio),
                    BeingNavDebugField::new("min_hunger_to_hunt", min_hunger_ratio_to_hunt),
                    BeingNavDebugField::new("hp_ratio", hp),
                    BeingNavDebugField::new("min_hp_ratio_to_hunt", predator_cfg.min_hp_ratio_to_hunt),
                ],
            );
            continue;
        }

        let Ok((pred_dim, pred_pos)) = params.pos_dim_query.get(pred_ent) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Clear,
                "Removed hunting: missing position",
                vec![],
            );
            continue;
        };
        let Some(pred_dim_ent) = resolve_dim_ent(pred_dim) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Clear,
                "Removed hunting: missing dimension",
                vec![],
            );
            continue;
        };
        let Some(candidates) = locals.candidates_by_dim.get(&pred_dim_ent) else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Clear,
                "Removed hunting: no candidates in dimension",
                vec![
                    BeingNavDebugField::new("dim", format!("{:?}", pred_dim.0)),
                ],
            );
            continue;
        };
        let pred_weight_newtons = params.body_weight_query.get(pred_ent).map(|sum| sum.0).unwrap_or_default();

        let mut closest: Option<(Entity, f32)> = None;
        for prey in candidates.iter() {
            if prey.ent == pred_ent {
                continue;
            }
            if prey_matches_predator_cfg_cached(prey, &predator_cfg, predator_cfg_source) {
                continue;
            }

            if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                && pred_weight_newtons > 0.0
                && prey.weight > pred_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
            {
                continue;
            }

            let dist = euclidean_dist(prey.pos, *pred_pos);
            let Some((_, curr_best)) = closest else {
                closest = Some((prey.ent, dist));
                continue;
            };
            if dist < curr_best {
                closest = Some((prey.ent, dist));
            }
        }

        let Some((closest_target, closest_dist)) = closest else {
            cmd.entity(pred_ent).try_remove::<Hunting>();
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Clear,
                "Removed hunting: no valid prey candidate",
                vec![
                    BeingNavDebugField::new("dim", format!("{:?}", pred_dim.0)),
                    BeingNavDebugField::new("gpos", *pred_pos),
                ],
            );
            continue;
        };
        let mut chosen_target = closest_target;
        if let Some(current_prey) = hunting.map(|hunting| hunting.prey) {
            if current_prey == pred_ent {
                cmd.entity(pred_ent).try_remove::<Hunting>();
                continue;
            }
            let Some(current_prey) = find_hunt_candidate(candidates, current_prey) else {
                cmd.entity(pred_ent).try_remove::<Hunting>();
                continue;
            };
            if grids.can_pathfind_between(*pred_pos, current_prey.pos, *pred_dim) {
                let current_dist = euclidean_dist(current_prey.pos, *pred_pos);
                if current_dist <= closest_dist + HUNT_RETARGET_HYSTERESIS_TILES {
                    chosen_target = current_prey.ent;
                }
            }
        }
        if hunting.is_none_or(|hunting| hunting.prey != chosen_target) {
            nav_log.push(
                pred_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Target,
                "Updated hunting target",
                vec![
                    BeingNavDebugField::new("dim", format!("{:?}", pred_dim.0)),
                    BeingNavDebugField::new("gpos", *pred_pos),
                    BeingNavDebugField::new("target", chosen_target),
                    BeingNavDebugField::new("previous", hunting.map(|hunting| hunting.prey)),
                    BeingNavDebugField::new("distance", closest_dist),
                ],
            );
        }
        cmd.entity(pred_ent).try_insert(Hunting::new(chosen_target));
    }

    for (_squad_ent, squad_members, ) in params.squad_members_query.iter() {
        if squad_members.iter().next().is_none() {
            continue;
        }

        locals.active_member_ents.clear();
        locals.target_list.clear();
        locals.reachable_squad_targets.clear();
        let mut squad_dim = None;
        let mut anchor_sum = IVec2::ZERO;
        let mut active_member_count = 0;
        let mut pack_predator_cfg: Option<PredatorCfg> = None;
        let mut pack_predator_cfg_source: Option<PredatorCfgSource> = None;
        for member_ent in squad_members.iter() {
            let Ok((_, _, hunting, body_condition, _)) = params.predators.get(member_ent) else {
                continue;
            };

            let member_bit_ref = params.bit_ref_query.get(member_ent).ok();
            let member_race_ref = params.race_ref_query.get(member_ent).ok();
            if params.predator_cfg_query.get(member_ent).is_ok() {
                continue;
            }
            let Some((predator_cfg, predator_cfg_source)) = resolve_predator_cfg(member_ent, member_bit_ref, member_race_ref, &params.bit_map, &params.race_map, &params.predator_cfg_query) else {
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
        let Some(squad_dim_ent) = resolve_dim_ent(&squad_dim) else {
            continue;
        };
        let Some(squad_candidates) = locals.candidates_by_dim.get(&squad_dim_ent) else {
            continue;
        };
        let anchor_pos = GlobalTilePos(anchor_sum / active_member_count as i32);
        let squad_weight_newtons = nearby_squad_body_weight(anchor_pos, &squad_dim, squad_members, &params.pos_dim_query, &params.body_weight_query);

        let mut committed_counts: EntityHashMap<u32> = EntityHashMap::default();
        for (member_ent, member_pos, member_dim, current_prey) in locals.active_member_ents.iter().copied() {
            let Some(current_prey) = current_prey else {
                continue;
            };
            if current_prey == member_ent {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            }
            let Some(_member_dim_ent) = resolve_dim_ent(&member_dim) else {
                continue;
            };
            let Some(current_candidate) = find_hunt_candidate(squad_candidates, current_prey) else {
                continue;
            };
            if !grids.can_pathfind_between(member_pos, current_candidate.pos, member_dim) {
                continue;
            }
            *committed_counts.entry(current_prey).or_insert(0) += 1;
        }

        let mut base_target = None;
        let mut best_base_dist = f32::MAX;
        for prey in squad_candidates.iter() {
            if squad_members.iter().any(|member_ent| member_ent == prey.ent) {
                continue;
            }
            if let Some(predator_cfg) = pack_predator_cfg.as_ref()
                && let Some(predator_cfg_source) = pack_predator_cfg_source
                && prey_matches_predator_cfg_cached(prey, predator_cfg, predator_cfg_source)
            {
                continue;
            }
            if let Some(predator_cfg) = pack_predator_cfg.as_ref() {
                if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                    && squad_weight_newtons > 0.0
                    && prey.weight > squad_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
                {
                    continue;
                }
            }
            let dist = euclidean_dist(prey.pos, anchor_pos);
            if dist < best_base_dist {
                best_base_dist = dist;
                base_target = Some(prey.ent);
            }
        }

        let Some(mut base_target) = base_target else {
            for member_ent in squad_members.iter() {
                cmd.entity(member_ent).try_remove::<Hunting>();
                nav_log.push(
                    member_ent,
                    NAV_SYSTEM,
                    BeingNavDebugKind::Clear,
                    "Removed squad hunting: no valid shared target",
                    vec![],
                );
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
                        if !grids.can_pathfind_between(*pred_pos, *member_pos, *pred_dim) {
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
            let Some(_member_dim_ent) = resolve_dim_ent(member_dim) else {
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
                if !grids.can_pathfind_between(*member_pos, *target_pos, *member_dim) {
                    target_ix += 1;
                    continue;
                }
                assigned_target = Some(target);
                break;
            }
            let Some(assigned_target) = assigned_target else {
                cmd.entity(member_ent).try_remove::<Hunting>();
                nav_log.push(
                    member_ent,
                    NAV_SYSTEM,
                    BeingNavDebugKind::Clear,
                    "Removed squad hunting: target unreachable",
                    vec![
                        BeingNavDebugField::new("leader_target", base_target),
                ],
                );
                continue;
            };
            if assigned_target == member_ent {
                cmd.entity(member_ent).try_remove::<Hunting>();
                continue;
            }
            let previous_target = locals
                .active_member_ents
                .iter()
                .find(|(ent, _, _, _)| *ent == member_ent)
                .and_then(|(_, _, _, current_prey)| *current_prey);
            nav_log.push(
                member_ent,
                NAV_SYSTEM,
                BeingNavDebugKind::Target,
                "Assigned squad hunting target",
                vec![
                    BeingNavDebugField::new("target", assigned_target),
                    BeingNavDebugField::new("previous", previous_target),
                ],
            );
            cmd.entity(member_ent).try_insert(Hunting::new(assigned_target));
        }
    }
    nav_log.flush();
}

#[allow(unused_parens, )]
pub fn sync_chasing_to_hunt(
    mut cmd: Commands,
    hunting_predators: Query<(Entity, &Hunting, Option<&Chasing>, ), (Changed<Hunting>)>,
    mut nav_log: BeingNavDebugLog,
) {
    for (pred_ent, hunting, chasing, ) in hunting_predators.iter() {
        if chasing
            .map(|chasing| chasing.target == hunting.prey)
            .unwrap_or(false)
        {
            continue;
        }
        cmd.entity(pred_ent).try_insert(Chasing::new(hunting.prey, 1.5));
        nav_log.push(
            pred_ent,
            NAV_SYSTEM,
            BeingNavDebugKind::Target,
            "Synced chasing to hunting target",
            vec![
                BeingNavDebugField::new("target", hunting.prey),
                BeingNavDebugField::new("previous", chasing.map(|chasing| chasing.target)),
            ],
        );
    }
    nav_log.flush();
}
