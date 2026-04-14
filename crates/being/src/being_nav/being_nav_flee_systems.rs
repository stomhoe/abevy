use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use param_sets::BlockingTileParamSet;

use crate::being_messages::NavOrder;
use crate::body::BodySums;

fn choose_flee_target_pos(
    blocking_tiles: &mut BlockingTileParamSet,
    being_ent: Entity,
    being_dim: ::tilemap_shared::DimensionRef,
    being_gpos: GlobalTilePos,
    flee_from_gpos: GlobalTilePos,
    _desired_distance_tiles: f32,
    avoid_tile_tags: &BlacklistedTags,
) -> Option<(GlobalTilePos, i32, u8)> {
    let empty_whitelist = WhitelistedTags::default();
    let empty_whitelist = WhitelistedSpawnTileTagsRef(&empty_whitelist);
    let avoid_tile_tags = BlacklistedSpawnTileTagsRef(avoid_tile_tags);

    fn tile_is_valid_flee_step(
        blocking_tiles: &mut BlockingTileParamSet,
        being_ent: Entity,
        being_dim: ::tilemap_shared::DimensionRef,
        candidate: GlobalTilePos,
        avoid_tile_tags: &BlacklistedSpawnTileTagsRef<'_>,
        empty_whitelist: &WhitelistedSpawnTileTagsRef<'_>,
    ) -> bool {
        if blocking_tiles.is_blocked_at(being_dim, candidate, being_ent, ) {
            return false;
        }
        if avoid_tile_tags.0.is_empty() {
            return true;
        }
        blocking_tiles.allowed_at_refs(
            being_dim,
            candidate,
            being_ent,
            empty_whitelist,
            avoid_tile_tags,
        )
    }

    fn candidate_escape_score(
        blocking_tiles: &mut BlockingTileParamSet,
        being_ent: Entity,
        being_dim: ::tilemap_shared::DimensionRef,
        being_gpos: GlobalTilePos,
        flee_from_gpos: GlobalTilePos,
        candidate: GlobalTilePos,
        step_dir: IVec2,
        avoid_tile_tags: &BlacklistedSpawnTileTagsRef<'_>,
        empty_whitelist: &WhitelistedSpawnTileTagsRef<'_>,
    ) -> Option<(i32, u8)> {
        if !tile_is_valid_flee_step(
            blocking_tiles,
            being_ent,
            being_dim,
                candidate,
                avoid_tile_tags,
                empty_whitelist,
        ) {
            return None;
        }

        let base_dist = (being_gpos.0 - flee_from_gpos.0).abs().element_sum() as f32;
        let candidate_dist = (candidate.0 - flee_from_gpos.0).abs().element_sum() as f32;
        if candidate_dist <= base_dist {
            return None;
        }
        let dist_gain = candidate_dist - base_dist;

        let mut open_exits = 0u8;
        for neighbor_dir in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let neighbor = GlobalTilePos(candidate.0 + neighbor_dir);
            if tile_is_valid_flee_step(
                blocking_tiles,
                being_ent,
                being_dim,
                neighbor,
                avoid_tile_tags,
                empty_whitelist,
            ) {
                open_exits += 1;
            }
        }

        let mut forward_run = 0i32;
        if step_dir != IVec2::ZERO {
            for run_dist in 1..=3 {
                let forward = GlobalTilePos(candidate.0 + step_dir * run_dist);
                if tile_is_valid_flee_step(
                    blocking_tiles,
                    being_ent,
                    being_dim,
                    forward,
                    avoid_tile_tags,
                    empty_whitelist,
                ) {
                    forward_run += 1;
                } else {
                    break;
                }
            }
        }

        let mut score = candidate_dist as i32 * 28
            + dist_gain as i32 * 80
            + (open_exits as i32) * 26
            + forward_run * 18;
        if dist_gain < 0.0 {
            score -= 220;
        }
        if open_exits <= 1 {
            score -= 260;
        }
        if open_exits == 0 {
            score -= 400;
        }
        Some((score, open_exits))
    }

    let away = being_gpos.0 - flee_from_gpos.0;
    let primary_step = if away == IVec2::ZERO {
        IVec2::X
    } else if away.x.abs() >= away.y.abs() {
        IVec2::new(away.x.signum(), 0)
    } else {
        IVec2::new(0, away.y.signum())
    };
    let lateral_step = IVec2::new(-primary_step.y, primary_step.x);
    let mut best: Option<(GlobalTilePos, i32, u8)> = None;
    for step in [primary_step, lateral_step, -lateral_step, -primary_step] {
        if step == IVec2::ZERO {
            continue;
        }
        for dist in 1..=8 {
            let candidate = GlobalTilePos(being_gpos.0 + step * dist);
            if blocking_tiles.is_blocked_at(being_dim, candidate, being_ent, ) {
                break;
            }
            let Some((score, open_exits)) = candidate_escape_score(
                blocking_tiles,
                being_ent,
                being_dim,
                being_gpos,
                flee_from_gpos,
                candidate,
                step,
                &avoid_tile_tags,
                &empty_whitelist,
            ) else {
                continue;
            };
            if best
                .map(|(_, best_score, best_open_exits)| {
                    score > best_score || (score == best_score && open_exits > best_open_exits)
                })
                .unwrap_or(true)
            {
                best = Some((candidate, score, open_exits));
            }
        }
    }
    best
}

fn resolve_flee_wander_cfg(
    member_of: Option<&SquadMemberOf>,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
    wander_cfg_query: &Query<&WanderSeri>,
) -> WanderSeri {
    if let Some(member_of) = member_of {
        if let Ok(cfg) = wander_cfg_query.get(member_of.0) {
            return cfg.clone();
        }
    }
    if let Some(bit_ref) = bit_ref {
        if let Ok(bit_ent) = bit_map.0.get_cloned(bit_ref.0)
            && let Ok(cfg) = wander_cfg_query.get(bit_ent)
        {
            return cfg.clone();
        }
    }
    if let Some(race_ref) = race_ref {
        if let Ok(race_ent) = race_map.0.get_cloned(race_ref.0)
            && let Ok(cfg) = wander_cfg_query.get(race_ent)
        {
            return cfg.clone();
        }
    }
    WanderSeri::default()
}

fn resolve_flee_avoid_tile_tags(
    cfg: &WanderSeri,
    has_avoid_blacklisted_spawn_tiles: bool,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
    blacklisted_spawn_tile_tags_query: &Query<&::tilemap_shared::BlacklistedSpawnTileTags>,
) -> BlacklistedTags {
    let mut avoid_tile_tags = BlacklistedTags::new(&cfg.avoid_tile_tags);
    if !has_avoid_blacklisted_spawn_tiles {
        return avoid_tile_tags;
    }
    if let Some(bit_ref) = bit_ref {
        if let Ok(bit_ent) = bit_map.0.get_cloned(bit_ref.0)
            && let Ok(bit_blacklisted_spawn_tile_tags) = blacklisted_spawn_tile_tags_query.get(bit_ent)
        {
            if !bit_blacklisted_spawn_tile_tags.0.is_empty() {
                avoid_tile_tags.extend_from(&bit_blacklisted_spawn_tile_tags.0);
                return avoid_tile_tags;
            }
        }
    }
    if let Some(race_ref) = race_ref {
        if let Ok(race_ent) = race_map.0.get_cloned(race_ref.0)
            && let Ok(race_blacklisted_spawn_tile_tags) = blacklisted_spawn_tile_tags_query.get(race_ent)
        {
            avoid_tile_tags.extend_from(&race_blacklisted_spawn_tile_tags.0);
        }
    }
    avoid_tile_tags
}

#[allow(unused_parens, )]
pub fn update_goto_from_fleeing(
    mut cmd: Commands,
    mut writer: MessageWriter<NavOrder>,
    mut blocking_tiles: BlockingTileParamSet,
    flee_query: Query<(Entity, &::tilemap_shared::DimensionRef, &Fleeing, Option<&SquadMemberOf>, Has<DoAvoidBlacklistedSpawnTilesForWander>, ), (With<Being>, LocalAiControlled, )>,
    body_weight_query: Query<&BodyWeightSum>,
    held_body_query: Query<&HeldBody>,
    body_sums_query: Query<&BodySums>,
    wander_cfg_query: Query<&WanderSeri>,
    blacklisted_spawn_tile_tags_query: Query<&::tilemap_shared::BlacklistedSpawnTileTags>,
    bit_map: Res<BeingInstTemplateEntityMap>,
    race_map: Res<RaceEntityMap>,
    mut messages: Local<Vec<NavOrder>>,
) {
    for (being_ent, &being_dim, fleeing, member_of, has_avoid_blacklisted_spawn_tiles, ) in flee_query.iter() {
        let Ok(&being_gpos) = blocking_tiles.gpos_query.get(being_ent) else {
            continue;
        };
        let mut primary_threat: Option<(Entity, GlobalTilePos, f32)> = None;
        for threat_ent in fleeing.threats.iter().copied() {
            let Ok(&flee_from_gpos) = blocking_tiles.gpos_query.get(threat_ent) else {
                continue;
            };
            let strength = body_weight_query.get(threat_ent).map_or(0.0, |weight_sum| weight_sum.0.max(0.0))
                + held_body_query
                    .get(threat_ent)
                    .ok()
                    .and_then(|held_body| body_sums_query.get(held_body.entity()).ok())
                    .map_or(0.0, |body_sums| body_sums.current_hp.max(0.0));
            let dist = (being_gpos.0 - flee_from_gpos.0).abs().element_sum().max(1) as f32;
            let threat_score = strength / dist;
            if primary_threat
                .map(|(_, _, best_score)| threat_score > best_score)
                .unwrap_or(true)
            {
                primary_threat = Some((threat_ent, flee_from_gpos, threat_score));
            }
        }
        let Some((primary_threat_ent, flee_from_gpos, _)) = primary_threat else {
            cmd.entity(being_ent).try_remove::<Fleeing>();
            continue;
        };
        let flee_dist = (being_gpos.0 - flee_from_gpos.0).abs().element_sum();
        if (flee_dist as f32) >= fleeing.desired_distance_tiles {
            cmd.entity(being_ent).try_remove::<Fleeing>();
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        }
        let bit_ref = blocking_tiles.get_being_bit_ref(being_ent);
        let race_ref = blocking_tiles.get_being_race_ref(being_ent);
        let cfg = resolve_flee_wander_cfg(member_of, bit_ref, race_ref, &bit_map, &race_map, &wander_cfg_query);
        let avoid_tile_tags = resolve_flee_avoid_tile_tags(
            &cfg,
            has_avoid_blacklisted_spawn_tiles,
            bit_ref,
            race_ref,
            &bit_map,
            &race_map,
            &blacklisted_spawn_tile_tags_query,
        );
        let Some((target_pos, score, open_exits)) = choose_flee_target_pos(
            &mut blocking_tiles,
            being_ent,
            being_dim,
            being_gpos,
            flee_from_gpos,
            fleeing.desired_distance_tiles,
            &avoid_tile_tags,
        ) else {
            cmd.entity(being_ent).try_remove::<Fleeing>();
            cmd.entity(being_ent).try_insert(Hunting::new(primary_threat_ent));
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        };
        trace!(
            target: BEING_SYSTEM,
            "Flee target selected for {:?}: from={:?} threat={:?} target={:?} score={} exits={}",
            being_ent,
            being_gpos,
            flee_from_gpos,
            target_pos,
            score,
            open_exits
        );
        messages.push(NavOrder::new(
            being_ent,
            255,
            NavOrderSource::Fleeing,
            // Fleeing is already gated by the threat distance check above; the
            // escape target itself should be pursued normally instead of being
            // treated as an additional stop-radius.
            Some(GoTo::new(target_pos, 0.0)),
        ));
    }
    writer.write_batch(messages.drain(..));
}
