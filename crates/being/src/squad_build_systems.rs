use bevy::ecs::entity_disabling::Disabled;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use ::being_shared::*;
use common::log_targets::BEING_SYSTEM;
use common::common_components::{HashId, StrId};
use game_common::Templ;
use game_common::game_common_components::TemplEntiRef;
use param_sets::BlockingTileParamSet;
use ::tilemap_shared::*;

use crate::being_bundles::BeingBundle;

#[derive(Default)]
struct SpawnTagsGroup {
    whitelist: WhitelistedTags,
    blacklist: BlacklistedTags,
    spawn_targets: Vec<Entity>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstancePackSourceKind {
    Pack,
    Race,
    Bit,
}

#[derive(SystemParam)]
pub(crate) struct InstancePackQueries<'w, 's> {
    pack_query: Query<'w, 's, (&'static PackRaceOrBitSampler, Option<&'static PackRaceOrBitSpawnQuotas>, Option<&'static StrId>), (With<Pack>, With<Templ>, )>,
    race_query: Query<'w, 's, (), (With<Race>, With<Templ>, )>,
    bit_query: Query<'w, 's, (), (With<BeingInstTemplate>, With<Templ>, )>,
    whitelist_query: Query<'w, 's, &'static WhitelistedSpawnTileTags, (With<Templ>, )>,
    blacklist_query: Query<'w, 's, &'static BlacklistedSpawnTileTags, (With<Templ>, )>,
    pack_spawn_radius_query: Query<'w, 's, &'static PackSpawnRadius>,
    no_spawn_squad_query: Query<'w, 's, (), With<NoSpawnSquadEntity>>,
    spawn_count_query: Query<'w, 's, (&'static PackInitialSizeSampler, )>,
    pack_predator_cfg_query: Query<'w, 's, &'static PredatorCfg, (With<Pack>, With<Templ>, )>,
    hash_query: Query<'w, 's, &'static HashId>,
    race_map: Res<'w, RaceEntityMap>,
    bit_map: Res<'w, BeingInstTemplateEntityMap>,
    dim_map: Res<'w, DimensionEntityMap>,
}

#[derive(SystemParam)]
pub(crate) struct InstancePackLocals<'s> {
    sampled_beings: Local<'s, Vec<Entity>>,
    spawn_positions: Local<'s, Vec<GlobalTilePos>>,
    grouped_spawn_targets: Local<'s, Vec<SpawnTagsGroup>>,
    spawn_assignments: Local<'s, Vec<(Entity, GlobalTilePos)>>,
    claimed_positions: Local<'s, HashSet<GlobalTilePos>>,
}

#[allow(unused_parens, )]
pub fn instance_pack_entities(
    mut cmd: Commands,
    mut requested_instances: MessageReader<InstantiateTemplPackEntity>,
    queries: InstancePackQueries,
    mut blocking_tiles: BlockingTileParamSet,
    mut to_enable_map: ResMut<BeingsToEnableOnChunkLoad>,
    mut next_spawn_group_id: ResMut<NextPendingNaturalSpawnGroupId>,
    mut locals: InstancePackLocals,
) {
    let mut rng = rand::rng();
    for msg in requested_instances.read() {
        let source_ent = msg.source_ent;
        let mut source_kind = None;
        let mut is_race_target = false;
        let mut is_bit_target = false;
        let mut density_tiles = PackSpawnRadius::default().as_i32();
        let mut from_no_spawn_squad = false;
        locals.sampled_beings.clear();

        if let Ok((being_sampler, forced_bounds, pack_str_id)) = queries.pack_query.get(source_ent) {
            source_kind = Some(InstancePackSourceKind::Pack);
            density_tiles = msg.resolved_pack_spawn_radius_tiles(
                queries.pack_spawn_radius_query.get(source_ent).ok().copied(),
            );
            from_no_spawn_squad = queries.no_spawn_squad_query.get(source_ent).is_ok();
            let sampled_count: usize = msg
                .override_being_count
                .map(usize::from)
                .unwrap_or_else(|| {
                    let sampled_count: usize = queries.spawn_count_query
                        .get(source_ent)
                        .ok()
                        .map(|(sampler, )| sampler.sample_count(&mut rng))
                        .unwrap_or(1);
                    let sampled_count_mult: f32 = msg.sampled_count_mult.unwrap_or(1.).max(0.);
                    ((sampled_count as f32) * sampled_count_mult).round().max(0.0) as usize
                });
            let forced_total = forced_bounds
                .map(|forced_bounds| {
                    forced_bounds
                        .0
                        .values()
                        .map(|(min_count, _)| usize::try_from(*min_count).unwrap_or(usize::MAX))
                        .sum::<usize>()
                })
                .unwrap_or_default();
            let final_count = sampled_count.max(forced_total);
            if final_count == 0 {
                trace!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: sampled final count was 0", source_ent);
                continue;
            }
            sample_pack_members(
                being_sampler,
                forced_bounds,
                final_count,
                &mut rng,
                &mut locals.sampled_beings,
                pack_str_id,
                &queries.race_map,
                &queries.bit_map,
            );
            if locals.sampled_beings.is_empty() {
                error!(target: BEING_SYSTEM, "Failed to sample members from pack template {:?} while handling InstancePackEntity", source_ent);
                continue;
            }
        } else if queries.race_query.get(source_ent).is_ok() {
            source_kind = Some(InstancePackSourceKind::Race);
            is_race_target = true;
            density_tiles = msg.resolved_pack_spawn_radius_tiles(
                queries.pack_spawn_radius_query.get(source_ent).ok().copied(),
            );
            from_no_spawn_squad = queries.no_spawn_squad_query.get(source_ent).is_ok();
            let final_count = msg
                .override_being_count
                .map(usize::from)
                .unwrap_or_else(|| {
                    let sampled_count = queries.spawn_count_query
                        .get(source_ent)
                        .ok()
                        .map(|(sampler, )| sampler.sample_count(&mut rng))
                        .unwrap_or(1);
                    let sampled_count_multiplier = msg.sampled_count_mult.unwrap_or(1.0).max(0.0);
                    ((sampled_count as f32) * sampled_count_multiplier).round().max(0.0) as usize
                });
            if final_count == 0 {
                trace!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: sampled final count was 0", source_ent);
                continue;
            }
            locals.sampled_beings.resize(final_count, source_ent);
        } else if queries.bit_query.get(source_ent).is_ok() {
            source_kind = Some(InstancePackSourceKind::Bit);
            is_bit_target = true;
            density_tiles = msg.resolved_pack_spawn_radius_tiles(
                queries.pack_spawn_radius_query.get(source_ent).ok().copied(),
            );
            from_no_spawn_squad = queries.no_spawn_squad_query.get(source_ent).is_ok();
            let final_count = msg
                .override_being_count
                .map(usize::from)
                .unwrap_or_else(|| {
                    let sampled_count = queries.spawn_count_query
                        .get(source_ent)
                        .ok()
                        .map(|(sampler, )| sampler.sample_count(&mut rng))
                        .unwrap_or(1);
                    let sampled_count_multiplier = msg.sampled_count_mult.unwrap_or(1.0).max(0.0);
                    ((sampled_count as f32) * sampled_count_multiplier).round().max(0.0) as usize
                });
            if final_count == 0 {
                trace!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: sampled final count was 0", source_ent);
                continue;
            }
            locals.sampled_beings.resize(final_count, source_ent);
        }

        let Some(source_kind) = source_kind else {
            error!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: source is not a Pack, Race, or BeingInstTemplate template entity", source_ent);
            continue;
        };
        let Some(anchor_gpos) = msg.member_gpos.first().copied() else {
            error!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: member_gpos was empty", source_ent);
            continue;
        };
        locals.spawn_positions.clear();
        let Some(_) = locals.sampled_beings.first() else {
            trace!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: no sampled beings", source_ent);
            continue;
        };
        locals.grouped_spawn_targets.clear();
        locals.grouped_spawn_targets.reserve(locals.sampled_beings.len());
        for &spawn_target in locals.sampled_beings.iter() {
            let whitelist = queries.whitelist_query
                .get(spawn_target)
                .map(|whitelist| whitelist.0.clone())
                .unwrap_or_default();
            let blacklist = queries.blacklist_query
                .get(spawn_target)
                .map(|blacklist| blacklist.0.clone())
                .unwrap_or_default();
            let mut matching_group_idx = None;
            for (idx, group) in locals.grouped_spawn_targets.iter().enumerate() {
                if group.whitelist == whitelist && group.blacklist == blacklist {
                    matching_group_idx = Some(idx);
                    break;
                }
            }
            if let Some(group_idx) = matching_group_idx {
                locals.grouped_spawn_targets[group_idx].spawn_targets.push(spawn_target);
                continue;
            }
            let mut new_group = SpawnTagsGroup {
                whitelist,
                blacklist,
                spawn_targets: Vec::with_capacity(1),
            };
            new_group.spawn_targets.push(spawn_target);
            locals.grouped_spawn_targets.push(new_group);
        }
        let preferred_radius_tiles = u16::try_from(density_tiles.max(0)).ok();
        locals.spawn_assignments.clear();
        locals.spawn_assignments.reserve(locals.sampled_beings.len());
        locals.claimed_positions.clear();
        locals.claimed_positions.reserve(locals.sampled_beings.len());
        for group in locals.grouped_spawn_targets.iter() {
            let Some(&group_entity) = group.spawn_targets.first() else {
                continue;
            };
            let needed_count = group.spawn_targets.len();
            let hard_max_radius_tiles = preferred_radius_tiles
                .map(|radius| radius.saturating_add(needed_count.saturating_sub(1) as u16));
            blocking_tiles.find_allowed_gposes_in_area(
                msg.dim_ref,
                anchor_gpos,
                needed_count,
                preferred_radius_tiles,
                hard_max_radius_tiles,
                true,
                msg.only_same_island,
                group_entity,
                &group.whitelist,
                &group.blacklist,
                &mut locals.spawn_positions,
            );
            let mut assigned_count = 0usize;
            for &gpos in locals.spawn_positions.iter() {
                if !locals.claimed_positions.insert(gpos) {
                    continue;
                }
                locals.spawn_assignments.push((group.spawn_targets[assigned_count], gpos));
                assigned_count += 1;
                if assigned_count >= needed_count {
                    break;
                }
            }
            if assigned_count < needed_count {
                trace!(target: BEING_SYSTEM, "Instancing source {:?} in {:?} found only {} allowed positions for tag-group size {} near {}", source_ent, msg.dim_ref, assigned_count, needed_count, anchor_gpos);
            }
        }
        if locals.spawn_assignments.is_empty() {
            trace!(target: BEING_SYSTEM, "Ignored InstancePackEntity for {:?}: no allowed spawn positions near {}", source_ent, anchor_gpos);
            continue;
        }
        if locals.spawn_assignments.len() < locals.sampled_beings.len() {
            trace!(target: BEING_SYSTEM, "Instancing source {:?} in {:?} found only {} allowed positions for {} sampled beings near {}", source_ent, msg.dim_ref, locals.spawn_assignments.len(), locals.sampled_beings.len(), anchor_gpos);
        }

        let spawns_squad = match msg.squad_spawn_mode {
            SquadSpawnMode::AutoFromTemplateFlag => !from_no_spawn_squad,
            SquadSpawnMode::ForceSpawn => true,
            SquadSpawnMode::DontSpawn => false,
        };
        let pack_predator_cfg = if matches!(source_kind, InstancePackSourceKind::Pack) {
            queries.pack_predator_cfg_query.get(source_ent).ok().cloned()
        } else {
            None
        };
        let squad_ent = spawns_squad.then(|| {
            let pack_entity = cmd.spawn((Pack, )).id();
            if matches!(source_kind, InstancePackSourceKind::Pack) {
                cmd.entity(pack_entity).insert(TemplEntiRef(source_ent));
            }
            pack_entity
        });
        let spawn_group_id = PendingNaturalSpawnGroupId(next_spawn_group_id.next());
        let spawn_policy = PendingNaturalSpawnPlacementPolicy {
            anchor_gpos,
            preferred_radius_tiles: preferred_radius_tiles.unwrap_or_default(),
            only_same_island: msg.only_same_island,
        };
        let mut spawned_members = 0usize;
        for (spawn_target, gpos) in locals.spawn_assignments.drain(..) {
            let gpos_chunk = gpos.to_chunkpos();
            let Ok(dim_ent) = queries.dim_map.0.get_cloned(msg.dim_ref.0) else {
                error!(target: BEING_SYSTEM, "Could not resolve DimensionRef hash {:?} when instancing source {:?}", msg.dim_ref.0, source_ent);
                continue;
            };
            let being_ent = cmd
                .spawn((
                    BeingBundle::new(msg.dim_ref, dim_ent, gpos),
                    Disabled,
                    NaturalSpawnOrigin(gpos_chunk),
                    spawn_group_id,
                    spawn_policy,
                ))
                .id();
            if let Some(squad_ent) = squad_ent {
                cmd.entity(being_ent).try_insert(SquadMemberOf(squad_ent));
            }
            if let Some(pack_predator_cfg) = pack_predator_cfg.as_ref() {
                cmd.entity(being_ent).try_insert(pack_predator_cfg.clone());
            }
            to_enable_map.insert(being_ent, msg.dim_ref, gpos_chunk);
            let target_is_bit = is_bit_target || queries.bit_query.get(spawn_target).is_ok();
            let target_is_race = is_race_target || queries.race_query.get(spawn_target).is_ok();
            if target_is_bit {
                let Ok(&spawn_target_hash) = queries.hash_query.get(spawn_target) else {
                    error!(target: BEING_SYSTEM, "Sampled BIT target {:?} is missing HashId", spawn_target);
                    continue;
                };
                cmd.entity(being_ent).insert(BitRef(spawn_target_hash));
            } else if target_is_race {
                let Ok(&spawn_target_hash) = queries.hash_query.get(spawn_target) else {
                    error!(target: BEING_SYSTEM, "Sampled Race target {:?} is missing HashId", spawn_target);
                    continue;
                };
                cmd.entity(being_ent).insert(RaceRef(spawn_target_hash));
            } else {
                error!(target: BEING_SYSTEM, "Sampled spawn target {:?} for source {:?} is neither BIT nor Race template", spawn_target, source_ent);
            }
            spawned_members += 1;
        }
        trace!(
            target: BEING_SYSTEM,
            "Instanced source {:?} (kind {:?}) with {} beings in {:?}, squad {:?}, radius {}, solo_member_predator_cfg={}",
            source_ent,
            source_kind,
            spawned_members,
            msg.dim_ref,
            squad_ent,
            density_tiles,
            pack_predator_cfg.is_some(),
        );
    }
}

fn sample_pack_members(
    being_sampler: &PackRaceOrBitSampler,
    forced_bounds: Option<&PackRaceOrBitSpawnQuotas>,
    target_count: usize,
    rng: &mut impl rand::Rng,
    out: &mut Vec<Entity>,
    pack_str_id: Option<&StrId>,
    race_map: &RaceEntityMap,
    bit_map: &BeingInstTemplateEntityMap,
) {
    out.clear();
    let Some(forced_bounds) = forced_bounds else {
        let mut sampled_hash_ids = Vec::with_capacity(target_count);
        being_sampler.0.sample_n_with_rng(target_count, rng, &mut sampled_hash_ids);
        out.reserve(sampled_hash_ids.len());
        for sampled_hash_id in sampled_hash_ids {
            if let Some(sampled_ent) = resolve_pack_hash_id(sampled_hash_id, race_map, bit_map) {
                out.push(sampled_ent);
            }
        }
        return;
    };

    let mut counts: EntityHashMap<usize> = EntityHashMap::default();
    let mut max_counts: EntityHashMap<usize> = EntityHashMap::default();
    let mut forced_total = 0usize;
    out.reserve(target_count.max(forced_bounds.0.len()));

    for (&ent, &(min_count, max_count)) in forced_bounds.0.iter() {
        let min_count = usize::try_from(min_count).unwrap_or(usize::MAX);
        let max_count = if max_count == u32::MAX {
            None
        } else {
            Some(usize::try_from(max_count).unwrap_or(usize::MAX).max(min_count))
        };
        forced_total = forced_total.saturating_add(min_count);
        counts.insert(ent, min_count);
        if let Some(max_count) = max_count {
            max_counts.insert(ent, max_count);
        }
        for _ in 0..min_count {
            out.push(ent);
        }
    }

    let target_count = target_count.max(forced_total);
    if out.len() >= target_count {
        return;
    }

    let mut attempts_left = target_count.saturating_mul(16).max(32);
    while out.len() < target_count && attempts_left > 0 {
        attempts_left -= 1;
        let Some(sampled_hash_id) = being_sampler.0.sample_with_rng(rng) else {
            break;
        };
        let Some(sampled_ent) = resolve_pack_hash_id(sampled_hash_id, race_map, bit_map) else {
            continue;
        };
        let current_count = counts.get(&sampled_ent).copied().unwrap_or_default();
        if let Some(&max_count) = max_counts.get(&sampled_ent) {
            if current_count >= max_count {
                continue;
            }
        }
        counts.insert(sampled_ent, current_count.saturating_add(1));
        out.push(sampled_ent);
    }

    if out.len() < target_count {
        trace!(
            target: BEING_SYSTEM,
            "Pack {:?} could only sample {} of {} requested members after applying forced min/max bounds",
            pack_str_id.map(StrId::as_str).unwrap_or("<no-strid>"),
            out.len(),
            target_count,
        );
    }
}

fn resolve_pack_hash_id(
    sampled_hash_id: HashId,
    race_map: &RaceEntityMap,
    bit_map: &BeingInstTemplateEntityMap,
) -> Option<Entity> {
    race_map
        .0
        .get_opt(sampled_hash_id)
        .copied()
        .or_else(|| bit_map.0.get_opt(sampled_hash_id).copied())
}
