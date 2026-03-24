use ::being_shared::*;
use faction_shared::Faction;
use ::tilemap_shared::GlobalTilePos;
use bevy::{ecs::{entity::{EntityHashMap, EntityHashSet}, entity_disabling::Disabled}, prelude::*};
use common::common_components::StrId;
use common::log_targets::BEING_SYSTEM;
use faction::faction_resources::FactionRef;

#[allow(unused_imports, )] use crate::being_components::*;

#[allow(unused_parens, )]
pub fn validate_added_beings_have_gpos(
    query: Query<(Entity, Option<&StrId>, Has<GlobalTilePos>, ), (LoadedBeing, ),>,
    added_being: Query<Entity, (Added<Being>, )>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut removed_unloaded: RemovedComponents<Unloaded>,
    mut to_iter: Local<EntityHashSet>,
) {
    to_iter.extend(added_being.iter());
    to_iter.extend(removed_disabled.read());
    to_iter.extend(removed_unloaded.read());
    for (ent, str_id, has_gpos, ) in query.iter_many(to_iter.drain()) {
        if has_gpos {
            continue;
        }
        error_once!(
            target: BEING_SYSTEM,
            "Added Being {:?} {} missing required components: GlobalTilePos={}",
            ent,
            str_id.map(StrId::as_str).unwrap_or("<no-strid>"),
            has_gpos,
        );
    }
}

#[allow(unused_parens, )]
pub fn sync_group_members_from_member_of(
    mut cmd: Commands,
    mut being_query: Query<(Entity, Option<Mut<JoinedGroups>>, Option<Mut<FactionRef>>), (With<Being>, )>,
    mut removed_member_of: RemovedComponents<JoinedGroups>,
    mut removed_faction_ref: RemovedComponents<FactionRef>,
    faction_query: Query<(), (With<Faction>, )>,
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

        let mut member_faction = None;
        for group_ent in current_member_of.iter() {
            if faction_query.get(group_ent).is_err() {
                continue;
            }
            member_faction = Some(group_ent);
            break;
        }

        let current_faction = faction_ref.as_ref().map(|faction_ref| faction_ref.0);
        if let Some(faction_ent) = current_faction {
            if faction_query.get(faction_ent).is_err() {
                error!(target: BEING_SYSTEM, "Being {:?} points at non-faction entity {:?} in FactionRef", being_ent, faction_ent);
            }
        }
        let current_faction = current_faction.filter(|faction_ent| faction_query.get(*faction_ent).is_ok());
        let desired_faction = member_faction.or(current_faction);
        if let Some(desired_faction) = desired_faction {
            if let Some(mut faction_ref) = faction_ref {
                if faction_ref.0 != desired_faction {
                    faction_ref.0 = desired_faction;
                }
            } else {
                cmd.entity(being_ent).try_insert(FactionRef(desired_faction));
            }
            if let Some(member_of) = member_of.as_mut() {
                if !member_of.contains(desired_faction) {
                    member_of.insert(desired_faction);
                }
                let faction_groups_to_remove: Vec<_> = member_of
                    .iter()
                    .filter(|group_ent| *group_ent != desired_faction && faction_query.get(*group_ent).is_ok())
                    .collect();
                for group_ent in faction_groups_to_remove {
                    member_of.remove(group_ent);
                }
                current_member_of = (**member_of).clone();
            }
            if !current_member_of.contains(desired_faction) {
                current_member_of.insert(desired_faction);
            }
            let faction_groups_to_remove: Vec<_> = current_member_of
                .iter()
                .filter(|group_ent| *group_ent != desired_faction && faction_query.get(*group_ent).is_ok())
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
        let Some((leader_ent, leader_rank)) = best_member else {
            if led_by.is_some() {
                cmd.entity(group_ent).try_remove::<LedBy>();
            }
            continue;
        };
        if led_by.map(|led_by| led_by.leader != leader_ent).unwrap_or(true) {
            cmd.entity(group_ent).try_insert(LedBy { leader: leader_ent });
        }
        debug!(target: BEING_SYSTEM, "Selected pack leader {:?} for pack {:?} with rank {}", leader_ent, group_ent, leader_rank);
    }
}
