use crate::being_nav::AiNavGrids;
use ::being_shared::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::log_targets::BEING_SYSTEM;
use game_common::game_common_components::TemplEntiRef;
use ::tilemap_shared::{DimensionRef, GlobalTilePos};

use crate::body::body_components::*;

const TEAM_COMBAT_RADIUS_TILES: f32 = 30.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FightOrFlightSourceKind {
    Pack,
    Being,
    Bit,
    Race,
    Default,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedFightOrFlightState {
    config: FightOrFlightConfig,
    style: FightingStyle,
    source_kind: FightOrFlightSourceKind,
}

#[derive(SystemParam)]
pub struct FightOrFlightReactionQueries<'w, 's> {
    fight_or_flight_query: Query<'w, 's, &'static FightOrFlightConfig>,
    fighting_style_query: Query<'w, 's, &'static FightingStyle>,
    pack_templ_ref_query: Query<'w, 's, &'static TemplEntiRef>,
    hunting_query: Query<'w, 's, &'static Hunting>,
    squad_member_of_query: Query<'w, 's, &'static SquadMemberOf>,
    squad_members_query: Query<'w, 's, &'static SquadMembers>,
    position_query: Query<'w, 's, (&'static DimensionRef, &'static GlobalTilePos)>,
    dim_map: Res<'w, ::tilemap_shared::DimensionEntityMap>,
    bit_ref_query: Query<'w, 's, &'static BitRef>,
    race_ref_query: Query<'w, 's, &'static RaceRef>,
    body_weight_sum_query: Query<'w, 's, &'static BodyWeightSum>,
    held_body_query: Query<'w, 's, &'static HeldBody>,
    body_sums_query: Query<'w, 's, &'static BodySums>,
    speed_query: Query<'w, 's, &'static SpeedMagnitude>,
    fleeing_query: Query<'w, 's, &'static Fleeing>,
    bit_map: Res<'w, BeingInstTemplateEntityMap>,
    race_map: Res<'w, RaceEntityMap>,
}

#[derive(SystemParam)]
pub struct FightOrFlightReactionLocals<'s> {
    pack_members: Local<'s, Vec<Entity>>,
}

fn melee_combat_strength(
    being_ent: Entity,
    weight_query: &Query<&BodyWeightSum>,
    held_body_query: &Query<&HeldBody>,
    body_sums_query: &Query<&BodySums>,
) -> f32 {
    let weight = weight_query.get(being_ent).map_or(0.0, |weight_sum| weight_sum.0.max(0.0));
    let body_hp = held_body_query
        .get(being_ent)
        .ok()
        .and_then(|held_body| body_sums_query.get(held_body.entity()).ok())
        .map_or(0.0, |body_sums| body_sums.current_hp.max(0.0));
    weight + body_hp
}

fn is_within_team_combat_radius(reference_gpos: GlobalTilePos, other_gpos: GlobalTilePos) -> bool {
    let delta = other_gpos.0 - reference_gpos.0;
    delta.as_vec2().length_squared() <= TEAM_COMBAT_RADIUS_TILES * TEAM_COMBAT_RADIUS_TILES
}

fn team_combat_strength(
    being_ent: Entity,
    reference_dim: DimensionRef,
    reference_gpos: GlobalTilePos,
    queries: &FightOrFlightReactionQueries,
) -> f32 {
    let mut total_strength = melee_combat_strength(
        being_ent,
        &queries.body_weight_sum_query,
        &queries.held_body_query,
        &queries.body_sums_query,
    );

    let Ok(squad_member_of) = queries.squad_member_of_query.get(being_ent) else {
        return total_strength;
    };
    let Ok(squad_members) = queries.squad_members_query.get(squad_member_of.0) else {
        return total_strength;
    };

    for member_ent in squad_members.iter() {
        if member_ent == being_ent {
            continue;
        }
        let Ok((member_dim, member_gpos)) = queries.position_query.get(member_ent) else {
            continue;
        };
        if *member_dim != reference_dim {
            continue;
        }
        if !is_within_team_combat_radius(reference_gpos, *member_gpos) {
            continue;
        }
        total_strength += melee_combat_strength(
            member_ent,
            &queries.body_weight_sum_query,
            &queries.held_body_query,
            &queries.body_sums_query,
        );
    }

    total_strength
}

fn current_hp_ratio(
    being_ent: Entity,
    held_body_query: &Query<&HeldBody>,
    body_sums_query: &Query<&BodySums>,
) -> f32 {
    let Some(current_hp) = held_body_query
        .get(being_ent)
        .ok()
        .and_then(|held_body| body_sums_query.get(held_body.entity()).ok())
        .map(|body_sums| (body_sums.current_hp, body_sums.total_hp))
    else {
        return 1.0;
    };
    let (curr_hp, total_hp) = current_hp;
    if total_hp <= 0.0 {
        return 1.0;
    }
    (curr_hp / total_hp).clamp(0.0, 1.0)
}

fn euclidean_dist(a: GlobalTilePos, b: GlobalTilePos) -> f32 {
    (a.0 - b.0).as_vec2().length()
}

fn speed_ratio_over_attacker(
    victim_ent: Entity,
    attacker_ent: Entity,
    speed_query: &Query<&SpeedMagnitude>,
) -> f32 {
    let victim_speed = speed_query.get(victim_ent).map_or(1.0, |speed| speed.0.max(0.0));
    let attacker_speed = speed_query.get(attacker_ent).map_or(1.0, |speed| speed.0.max(0.0));
    if attacker_speed <= f32::EPSILON {
        return victim_speed.max(0.0);
    }
    victim_speed / attacker_speed
}

fn resolve_template_entity_config(
    source_ent: Entity,
    fight_or_flight_query: &Query<&FightOrFlightConfig>,
    fighting_style_query: &Query<&FightingStyle>,
) -> Option<(FightOrFlightConfig, FightingStyle)> {
    let config = fight_or_flight_query.get(source_ent).ok().copied();
    let style = fighting_style_query.get(source_ent).ok().copied();
    if config.is_some() || style.is_some() {
        return Some((config.unwrap_or_default(), style.unwrap_or_default()));
    }
    None
}

fn resolve_template_entity_config_with_pack_fallback(
    source_ent: Entity,
    queries: &FightOrFlightReactionQueries,
) -> Option<(FightOrFlightConfig, FightingStyle)> {
    let pack_template_ent = queries.pack_templ_ref_query.get(source_ent).ok().map(|templ_ref| templ_ref.0);
    let config = queries
        .fight_or_flight_query
        .get(source_ent)
        .ok()
        .copied()
        .or_else(|| pack_template_ent.and_then(|pack_template_ent| queries.fight_or_flight_query.get(pack_template_ent).ok().copied()));
    let style = queries
        .fighting_style_query
        .get(source_ent)
        .ok()
        .copied()
        .or_else(|| pack_template_ent.and_then(|pack_template_ent| queries.fighting_style_query.get(pack_template_ent).ok().copied()));
    if config.is_some() || style.is_some() {
        return Some((config.unwrap_or_default(), style.unwrap_or_default()));
    }
    None
}

fn resolve_fight_or_flight_state(
    being_ent: Entity,
    queries: &FightOrFlightReactionQueries,
) -> ResolvedFightOrFlightState {
    if let Ok(squad_member_of) = queries.squad_member_of_query.get(being_ent) {
        let pack_ent = squad_member_of.0;
        if let Some((config, style)) = resolve_template_entity_config_with_pack_fallback(pack_ent, queries) {
            return ResolvedFightOrFlightState {
                config,
                style,
                source_kind: FightOrFlightSourceKind::Pack,
            };
        }
    }

    if let Some((config, style)) = resolve_template_entity_config(being_ent, &queries.fight_or_flight_query, &queries.fighting_style_query) {
        return ResolvedFightOrFlightState {
            config,
            style,
            source_kind: FightOrFlightSourceKind::Being,
        };
    }

    let bit_ent = queries.bit_ref_query.get(being_ent).ok().and_then(|bit_ref| queries.bit_map.0.get_cloned(bit_ref.0).ok());
    if let Some(bit_ent) = bit_ent {
        if let Some((config, style)) = resolve_template_entity_config(bit_ent, &queries.fight_or_flight_query, &queries.fighting_style_query) {
            return ResolvedFightOrFlightState {
                config,
                style,
                source_kind: FightOrFlightSourceKind::Bit,
            };
        }
    }

    let race_ent = queries
        .race_ref_query
        .get(being_ent)
        .ok()
        .and_then(|race_ref| queries.race_map.0.get_cloned(race_ref.0).ok())
        .or_else(|| {
            bit_ent.and_then(|bit_ent| {
                queries
                    .race_ref_query
                    .get(bit_ent)
                    .ok()
                    .and_then(|race_ref| queries.race_map.0.get_cloned(race_ref.0).ok())
            })
        });
    if let Some(race_ent) = race_ent {
        if let Some((config, style)) = resolve_template_entity_config(race_ent, &queries.fight_or_flight_query, &queries.fighting_style_query) {
            return ResolvedFightOrFlightState {
                config,
                style,
                source_kind: FightOrFlightSourceKind::Race,
            };
        }
    }

    ResolvedFightOrFlightState {
        config: FightOrFlightConfig::default(),
        style: FightingStyle::default(),
        source_kind: FightOrFlightSourceKind::Default,
    }
}

fn ensure_fleeing(
    cmd: &mut Commands,
    being_ent: Entity,
    attacker_ent: Entity,
    desired_distance_tiles: f32,
    existing_fleeing: Option<&Fleeing>,
) {
    let mut fleeing = existing_fleeing
        .cloned()
        .unwrap_or_else(|| Fleeing::with_distance(attacker_ent, desired_distance_tiles));
    fleeing.add_threat(attacker_ent);
    fleeing.desired_distance_tiles = fleeing.desired_distance_tiles.max(desired_distance_tiles.max(0.0));
    cmd.entity(being_ent).try_remove::<(Hunting, Chasing, AiAutoMeleeTargets)>();
    cmd.entity(being_ent).insert(fleeing);
}

fn ensure_counterattack(
    cmd: &mut Commands,
    being_ent: Entity,
    attacker_ent: Entity,
    retaliating: bool,
    retaliation_stop_distance_tiles: f32,
) {
    cmd.entity(being_ent).try_remove::<Fleeing>();
    let hunting = if retaliating {
        Hunting::with_retaliation(attacker_ent, retaliation_stop_distance_tiles)
    } else {
        Hunting::new(attacker_ent)
    };
    cmd.entity(being_ent).try_insert(hunting);
}

fn should_preserve_current_hunt_for_pack_counterattack(
    being_ent: Entity,
    attacker_ent: Entity,
    current_hunting: &Hunting,
    queries: &FightOrFlightReactionQueries,
    grids: &AiNavGrids,
) -> bool {
    if !current_hunting.retaliating {
        return false;
    }

    let Some((being_dim, being_pos)) = queries.position_query.get(being_ent).ok() else {
        return false;
    };
    let Some(being_dim_ent) = queries.dim_map.0.get_opt(being_dim.0).copied() else {
        return false;
    };

    let Some((attacker_dim, attacker_pos)) = queries.position_query.get(attacker_ent).ok() else {
        return false;
    };
    if attacker_dim != being_dim {
        return false;
    }
    let Some(attacker_dim_ent) = queries.dim_map.0.get_opt(attacker_dim.0).copied() else {
        return false;
    };
    if attacker_dim_ent != being_dim_ent {
        return false;
    }

    let Some((current_prey_dim, current_prey_pos)) = queries.position_query.get(current_hunting.prey).ok() else {
        return false;
    };
    if current_prey_dim != being_dim {
        return false;
    }
    let Some(current_prey_dim_ent) = queries.dim_map.0.get_opt(current_prey_dim.0).copied() else {
        return false;
    };
    if current_prey_dim_ent != being_dim_ent {
        return false;
    }

    if !grids.can_pathfind_between(*being_pos, *current_prey_pos, being_dim_ent) {
        return false;
    }

    euclidean_dist(*being_pos, *current_prey_pos) <= euclidean_dist(*being_pos, *attacker_pos)
}

fn apply_reaction_to_being(
    cmd: &mut Commands,
    being_ent: Entity,
    attacker_ent: Entity,
    queries: &FightOrFlightReactionQueries,
    grids: &AiNavGrids,
) {
    let resolved = resolve_fight_or_flight_state(being_ent, queries);
    let current_hp_ratio = current_hp_ratio(being_ent, &queries.held_body_query, &queries.body_sums_query);
    let (victim_strength, attacker_strength) = if let Ok((victim_dim, victim_gpos)) = queries.position_query.get(being_ent) {
        (
            team_combat_strength(being_ent, *victim_dim, *victim_gpos, queries),
            team_combat_strength(attacker_ent, *victim_dim, *victim_gpos, queries),
        )
    } else {
        (
            melee_combat_strength(being_ent, &queries.body_weight_sum_query, &queries.held_body_query, &queries.body_sums_query),
            melee_combat_strength(attacker_ent, &queries.body_weight_sum_query, &queries.held_body_query, &queries.body_sums_query),
        )
    };
    let strength_ratio = if attacker_strength <= f32::EPSILON {
        f32::INFINITY
    } else {
        victim_strength / attacker_strength
    };
    let speed_ratio = speed_ratio_over_attacker(being_ent, attacker_ent, &queries.speed_query);
    let ranged_keep_distance = resolved.style.ranged_keep_distance_threshold().map(|threshold| speed_ratio >= threshold).unwrap_or(false);

    let should_flee = match resolved.source_kind {
        FightOrFlightSourceKind::Pack => matches!(resolved.config.reaction, FightOrFlightReaction::Flee),
        _ => resolved
            .config
            .curr_hp_ratio_over_my_max_hp_to_start_fleeing
            .is_some_and(|threshold| current_hp_ratio <= threshold)
            || matches!(resolved.config.reaction, FightOrFlightReaction::Flee)
            || strength_ratio < resolved.config.min_melee_strength_ratio_to_counterattack,
    };

    if should_flee {
        let flee_distance = if ranged_keep_distance { 10.0 } else { 20.0 };
        let existing_fleeing = queries.fleeing_query.get(being_ent).ok();
        ensure_fleeing(cmd, being_ent, attacker_ent, flee_distance, existing_fleeing);
        return;
    }

    if ranged_keep_distance {
        let existing_fleeing = queries.fleeing_query.get(being_ent).ok();
        ensure_fleeing(cmd, being_ent, attacker_ent, 10.0, existing_fleeing);
        return;
    }

    if matches!(resolved.source_kind, FightOrFlightSourceKind::Pack)
        && let Some(current_hunting) = queries.hunting_query.get(being_ent).ok()
    {
        if should_preserve_current_hunt_for_pack_counterattack(being_ent, attacker_ent, current_hunting, queries, grids) {
            debug!(
                target: BEING_SYSTEM,
                "Pack reaction kept current hunt for {:?}: current prey {:?} is retaliating and closer than attacker {:?}",
                being_ent,
                current_hunting.prey,
                attacker_ent
            );
            return;
        }

        debug!(
            target: BEING_SYSTEM,
            "Pack reaction retargeted {:?} from {:?} to attacker {:?}",
            being_ent,
            current_hunting.prey,
            attacker_ent
        );
    }

    let retaliating = !matches!(resolved.source_kind, FightOrFlightSourceKind::Pack);
    ensure_counterattack(
        cmd,
        being_ent,
        attacker_ent,
        retaliating,
        resolved.config.retaliation_chase_stop_distance_tiles,
    );
}

fn affected_beings_for_damage(
    victim_ent: Entity,
    queries: &FightOrFlightReactionQueries,
    locals: &mut FightOrFlightReactionLocals,
) -> Vec<Entity> {
    if let Ok(squad_member_of) = queries.squad_member_of_query.get(victim_ent) {
        let pack_ent = squad_member_of.0;
        let Some((config, _style)) = resolve_template_entity_config_with_pack_fallback(pack_ent, queries) else {
            return vec![victim_ent];
        };
        if config.entire_nearby_squad_counterattacks {
            if let Ok(squad_members) = queries.squad_members_query.get(pack_ent) {
                locals.pack_members.clear();
                locals.pack_members.extend(squad_members.iter());
                return locals.pack_members.clone();
            }
            return vec![victim_ent];
        }
    }
    vec![victim_ent]
}

#[allow(unused_parens, )]
pub fn handle_fight_or_flight_reactions(
    mut cmd: Commands,
    mut reader: MessageReader<IncHealthDamageOrHeal>,
    queries: FightOrFlightReactionQueries,
    grids: Res<AiNavGrids>,
    mut locals: FightOrFlightReactionLocals,
) {
    for damage in reader.read() {
        let attacker_ent = damage.source_ent;
        if attacker_ent == Entity::PLACEHOLDER {
            continue;
        }
        let affected_beings = affected_beings_for_damage(damage.target_ent, &queries, &mut locals);
        for being_ent in affected_beings {
            apply_reaction_to_being(&mut cmd, being_ent, attacker_ent, &queries, &grids);
        }
    }
}
