use ::being_shared::*;
use faction_shared::Faction;
use ::tilemap_shared::GlobalTilePos;
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::common_components::{DisplayName, HashId, StrId};
use common::log_targets::BEING_SYSTEM;
use faction::faction_resources::{FactionEntityMap, FactionRef};
use game_common::Templ;
use game_common::game_common_components::TemplEntiRef;
use crate::being_resources::BeingEntityMap;

fn missing_gpos_entity_label(
    ent: Entity,
    str_id: Option<&StrId>,
    hash_id: Option<&HashId>,
    display_name: Option<&DisplayName>,
) -> String {
    let mut label = format!("Being {:?}", ent);
    if let Some(str_id) = str_id {
        label.push_str(&format!(" StrId={}", str_id.as_str()));
    }
    if let Some(hash_id) = hash_id {
        label.push_str(&format!(" HashId={}", hash_id));
    }
    if let Some(display_name) = display_name {
        label.push_str(&format!(" DisplayName={}", display_name));
    }
    label
}

#[allow(unused_parens, )]
pub fn assign_being_hash_ids(
    mut cmd: Commands,
    mut being_entity_map: ResMut<BeingEntityMap>,
    mut next_hash_id: Local<u64>,
    query: Query<(Entity, Option<&HashId>, ), (Added<Being>, )>,
) {
    if *next_hash_id == 0 {
        *next_hash_id = 1;
    }

    for (being_ent, existing_hash_id) in query.iter() {
        let hash_id = if let Some(&hash_id) = existing_hash_id.filter(|hash_id| **hash_id != HashId::default()) {
            hash_id
        } else {
            let mut candidate = HashId::new(*next_hash_id);
            while being_entity_map.0.contains_key(candidate) {
                *next_hash_id = (*next_hash_id).saturating_add(1);
                candidate = HashId::new(*next_hash_id);
            }
            cmd.entity(being_ent).insert(candidate);
            *next_hash_id = (*next_hash_id).saturating_add(1);
            candidate
        };

        if let Some(prev_ent) = being_entity_map.insert(hash_id, being_ent) {
            if prev_ent != being_ent {
                error!(
                    target: BEING_SYSTEM,
                    "Duplicate stable HashId {:?} for beings {:?} and {:?}",
                    hash_id,
                    prev_ent,
                    being_ent,
                );
            }
        }
    }
}

#[allow(unused_parens, )]
pub fn validate_added_beings_have_gpos(
    query: Query<(Entity, Option<&StrId>, Option<&HashId>, Option<&DisplayName>, Has<GlobalTilePos>, ), (LoadedBeing, ),>,
    time: Res<Time>,
    mut missing_gpos_timers: Local<EntityHashMap<Timer>>,
    mut loaded_beings: Local<EntityHashSet>,
) {
    loaded_beings.clear();

    let iter = query.iter();
    let loaded_count = iter.size_hint().1.unwrap_or(iter.size_hint().0);
    loaded_beings.reserve(loaded_count);

    for (ent, str_id, hash_id, display_name, has_gpos, ) in iter {
        loaded_beings.insert(ent);

        if has_gpos {
            missing_gpos_timers.remove(&ent);
            continue;
        }

        let timer = missing_gpos_timers
            .entry(ent)
            .or_insert_with(|| Timer::from_seconds(4.0, TimerMode::Once));
        if timer.is_finished() {
            continue;
        }
        timer.tick(time.delta());
        if timer.is_finished() {
            error!(
                target: BEING_SYSTEM,
                "{} still missing GlobalTilePos after 4s",
                missing_gpos_entity_label(ent, str_id, hash_id, display_name),
            );
        }
    }

    let mut stale_ents = Vec::new();
    for ent in missing_gpos_timers.keys().copied() {
        if loaded_beings.iter().any(|loaded_ent| *loaded_ent == ent) {
            continue;
        }
        stale_ents.push(ent);
    }
    for ent in stale_ents {
        missing_gpos_timers.remove(&ent);
    }
}

#[allow(unused_parens, )]
pub fn sync_group_members_from_member_of(
    mut cmd: Commands,
    mut being_query: Query<(Entity, Option<Mut<JoinedGroups>>, Option<Mut<FactionRef>>), (With<Being>, )>,
    mut removed_member_of: RemovedComponents<JoinedGroups>,
    mut removed_faction_ref: RemovedComponents<FactionRef>,
    faction_map: Res<FactionEntityMap>,
    faction_query: Query<(), (With<Faction>, )>,
    faction_hash_query: Query<&HashId, (With<Faction>, )>,
    mut group_members_query: Query<&mut BeingMembers, >,
    mut groups_by_being: Local<EntityHashMap<EntityHashSet>>,
    mut touched_beings: Local<EntityHashSet>,
    mut next_groups: Local<EntityHashSet>,
    mut initialized: Local<bool>,
) {
    touched_beings.clear();

    if !*initialized {
        groups_by_being.clear();
        let iter = being_query.iter();
        touched_beings.reserve(iter.size_hint().1.unwrap_or(iter.size_hint().0));
        for (being_ent, _, _) in iter {
            touched_beings.insert(being_ent);
        }
        *initialized = true;
    } else {
        let iter = being_query.iter();
        touched_beings.reserve(iter.size_hint().1.unwrap_or(iter.size_hint().0));
        for (being_ent, member_of, faction_ref) in iter {
            if member_of.as_ref().is_some_and(|member_of| member_of.is_changed())
                || faction_ref.as_ref().is_some_and(|faction_ref| faction_ref.is_changed())
            {
                touched_beings.insert(being_ent);
            }
        }

        let removed_member_of = removed_member_of.read();
        touched_beings.reserve(removed_member_of.size_hint().1.unwrap_or(removed_member_of.size_hint().0));
        for being_ent in removed_member_of {
            touched_beings.insert(being_ent);
        }
        let removed_faction_ref = removed_faction_ref.read();
        touched_beings.reserve(removed_faction_ref.size_hint().1.unwrap_or(removed_faction_ref.size_hint().0));
        for being_ent in removed_faction_ref {
            touched_beings.insert(being_ent);
        }
    }

    if touched_beings.is_empty() {
        return;
    }

    next_groups.clear();
    for being_ent in touched_beings.drain() {
        let Ok((_, mut member_of, faction_ref, )) = being_query.get_mut(being_ent) else {
            continue;
        };
        let mut current_member_of: JoinedGroups = member_of
            .as_ref()
            .map(|member_of| (**member_of).clone())
            .unwrap_or_else(|| JoinedGroups(EntityHashSet::default()));
        let current_groups = groups_by_being.entry(being_ent).or_default();
        let previous_groups = current_groups.clone();

        let mut member_faction_ent = None;
        for group_ent in current_member_of.iter() {
            if faction_query.get(group_ent).is_err() {
                continue;
            }
            member_faction_ent = Some(group_ent);
            break;
        }

        let current_faction_ent = faction_ref
            .as_ref()
            .and_then(|faction_ref| faction_map.0.get_cloned(faction_ref.0).ok());
        let desired_faction_ent = member_faction_ent.or(current_faction_ent);
        if let Some(desired_faction_ent) = desired_faction_ent {
            let Ok(&desired_faction_hash) = faction_hash_query.get(desired_faction_ent) else {
                error!(target: BEING_SYSTEM, "Faction entity {:?} missing HashId while syncing being {:?}", desired_faction_ent, being_ent);
                continue;
            };
            if let Some(mut faction_ref) = faction_ref {
                if faction_ref.0 != desired_faction_hash {
                    faction_ref.0 = desired_faction_hash;
                }
            } else {
                cmd.entity(being_ent).try_insert(FactionRef(desired_faction_hash));
            }
            if let Some(member_of) = member_of.as_mut() {
                if !member_of.contains(desired_faction_ent) {
                    member_of.insert(desired_faction_ent);
                }
                let faction_groups_to_remove: Vec<_> = member_of
                    .iter()
                    .filter(|group_ent| *group_ent != desired_faction_ent && faction_query.get(*group_ent).is_ok())
                    .collect();
                for group_ent in faction_groups_to_remove {
                    member_of.remove(group_ent);
                }
                current_member_of = (**member_of).clone();
            }
            if !current_member_of.contains(desired_faction_ent) {
                current_member_of.insert(desired_faction_ent);
            }
            let faction_groups_to_remove: Vec<_> = current_member_of
                .iter()
                .filter(|group_ent| *group_ent != desired_faction_ent && faction_query.get(*group_ent).is_ok())
                .collect();
            for group_ent in faction_groups_to_remove {
                current_member_of.remove(group_ent);
            }
        } else if faction_ref.is_some() {
            cmd.entity(being_ent).try_remove::<FactionRef>();
        }

        for group_ent in current_member_of.iter() {
            next_groups.insert(group_ent);
            if previous_groups.contains(&group_ent) {
                continue;
            }
            if let Ok(mut group_members) = group_members_query.get_mut(group_ent) {
                group_members.0.insert(being_ent);
            }
        }
        for group_ent in previous_groups.iter().copied() {
            if next_groups.contains(&group_ent) {
                continue;
            }
            let Ok(mut group_members) = group_members_query.get_mut(group_ent) else {
                continue;
            };
            group_members.0.remove(&being_ent);
            if group_members.is_empty() {
                cmd.entity(group_ent).try_remove::<BeingMembers>();
            }
        }
        *current_groups = core::mem::take(&mut *next_groups);
    }
}

#[allow(unused_parens, )]
pub fn refresh_leader_on_member_rank_change(
    mut cmd: Commands,
    changed_member_ranks_query: Query<
        (Entity, &SquadMembers, &MemberRanks, Option<&LedBy>, ),
        (Or<(Changed<MemberRanks>, Changed<SquadMembers>)>, ),
    >,
) {
    for (group_ent, members, member_ranks, led_by, ) in changed_member_ranks_query.iter() {
        let mut best_member: Option<(Entity, f32)> = None;
        for member_ent in members.iter() {
            let Some(&candidate_rank) = member_ranks.0.get(&member_ent) else {
                continue;
            };
            if best_member
                .map(|(_, best_rank)| candidate_rank > best_rank)
                .unwrap_or(true)
            {
                best_member = Some((member_ent, candidate_rank));
            }
        }
        //ta bien
        let Some((leader_ent, _)) = best_member
        else {
            cmd.entity(group_ent).try_remove::<LedBy>();
            continue;
        };
        if led_by.map(|led_by| led_by.leader != leader_ent).unwrap_or(true) {
            cmd.entity(group_ent).try_insert(LedBy { leader: leader_ent });
        }
    }
}

#[allow(unused_parens, )]
pub fn assign_member_ranks_on_joined_squad(
    mut cmd: Commands,
    joined_members: Query<
        (Entity, &SquadMemberOf, Option<&BitRef>, Option<&RaceRef>, ),
        (Added<SquadMemberOf>, With<Being>, ),
    >,
    mut squads_query: Query<
        (
            Option<&PackMemberRankSampler>,
            Option<&TemplEntiRef>,
            Option<&mut MemberRanks>,
        ),
        (Without<Templ>, ),
    >,
    bit_map: Res<BeingInstTemplateEntityMap>,
    race_map: Res<RaceEntityMap>,
    templ_rank_sampler_query: Query<&PackMemberRankSampler, (With<Templ>, )>,
) {
    let mut rng = rand::rng();
    for (being_ent, squad_member_of, bit_ref, race_ref, ) in joined_members.iter() {
        let squad_ent = squad_member_of.0;
        let Ok((rank_sampler_on_squad, templ_ref, member_ranks, )) = squads_query.get_mut(squad_ent) else {
            continue;
        };
        let rank_sampler_from_templ = templ_ref
            .and_then(|templ_ref| templ_rank_sampler_query.get(templ_ref.0).ok());
        let rank_sampler = rank_sampler_on_squad.or(rank_sampler_from_templ);
        let bit_ent = bit_ref.and_then(|bit_ref| bit_map.0.get_cloned(bit_ref.0).ok());
        let race_ent = race_ref.and_then(|race_ref| race_map.0.get_cloned(race_ref.0).ok());
        let rank_dist = rank_sampler.and_then(|rank_sampler| {
            bit_ent
                .and_then(|bit_ent| rank_sampler.0.get(&bit_ent))
                .or_else(|| race_ent.and_then(|race_ent| rank_sampler.0.get(&race_ent)))
        });
        let sampled_rank = rank_dist
            .map(|rank_dist| rank_dist.sample(&mut rng))
            .unwrap_or(0.0);
        if let Some(mut member_ranks) = member_ranks {
            member_ranks.0.insert(being_ent, sampled_rank);
        } else {
            let mut new_member_ranks = EntityHashMap::default();
            new_member_ranks.insert(being_ent, sampled_rank);
            cmd.entity(squad_ent).try_insert(MemberRanks(new_member_ranks));
        }
        trace!(target: BEING_SYSTEM, "assign_member_ranks_on_joined_squad: squad={:?} member={:?} rank={}", squad_ent, being_ent, sampled_rank);
    }
}
