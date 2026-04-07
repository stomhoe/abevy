#[allow(unused_imports)] use bevy::prelude::*;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use ::being_shared::*;
use common::log_targets::FACTION_SYSTEM;
use game_common::Templ;
use player_shared::player_components::*;

#[allow(unused_imports, )]use faction_shared::*;
#[allow(unused_imports, )]use crate::{faction_resources::*, };


#[allow(unused_parens)]///TODO hacer con un paramset en vez de guardar tanto estado via marker components?
pub fn set_stuff_as_self_faction(mut cmd: Commands,
    things_query: Query<
        (
            Entity,
            Ref<JoinedGroups>,
            Has<BelongsToAPlayerFaction>,
            Has<IsAffiliatedToMyFaction>,
        ),
        Without<Player>,
    >,
    selfplayer_faction_query: Query<Ref<FactionRef>, (MyPlayer)>,
    added_mine_query: Query<(), (With<Player>, Added<Mine>)>,
    player_factions: Query<(), With<PlayerMembers>>,
    player_faction_changes: Query<(), (With<Player>, Changed<FactionRef>)>,
    mut removed_player_factions: RemovedComponents<PlayerMembers>,
) {
    let had_removed_player_factions = removed_player_factions.read().next().is_some();
    let rerun_all = !added_mine_query.is_empty()
        || selfplayer_faction_query
            .single()
            .is_ok_and(|selfplayer_faction| selfplayer_faction.is_changed())
        || !player_faction_changes.is_empty()
        || had_removed_player_factions;
    if !rerun_all && things_query.iter().all(|(_, member_of, _, _)| !member_of.is_changed()) {
        return;
    }
    let Ok(selfplayer_faction) = selfplayer_faction_query.single() else {
        error!("Failed to get my player faction");
        return;
    };
    for (thing_ent, member_of, has_player_faction, is_affiliated_to_my_faction) in things_query.iter() {
        if !rerun_all && !member_of.is_changed() {
            continue;
        }
        let belongs_to_player_faction = member_of.iter().any(|group_ent| player_factions.get(group_ent).is_ok());
        let belongs_to_self_faction = member_of.contains(selfplayer_faction.0);
        if belongs_to_player_faction {
            if !has_player_faction {
                cmd.entity(thing_ent).try_insert(BelongsToAPlayerFaction);
            }

            if belongs_to_self_faction {
                if !is_affiliated_to_my_faction {
                    cmd.entity(thing_ent).try_insert(IsAffiliatedToMyFaction);
                }
            } else if is_affiliated_to_my_faction {
                cmd.entity(thing_ent).try_remove::<IsAffiliatedToMyFaction>();
            }
        } else {
            if has_player_faction {
                cmd.entity(thing_ent).try_remove::<BelongsToAPlayerFaction>();
            }
            if is_affiliated_to_my_faction {
                cmd.entity(thing_ent).try_remove::<IsAffiliatedToMyFaction>();
            }
        }
    }

}

#[allow(unused_parens, )]
pub fn update_player_members_of_groups(
    mut cmd: Commands,
    mut player_query: Query<
        (
            Entity,
            Has<Mine>,
            Option<Mut<FactionRef>>,
            Option<Mut<JoinedGroups>>,
        ),
        (With<Player>, ),
    >,
    mut removed_member_of: RemovedComponents<JoinedGroups>,
    mut removed_faction_ref: RemovedComponents<FactionRef>,
    mut removed_mine: RemovedComponents<Mine>,
    faction_query: Query<(), (With<Faction>, Without<Templ>)>,
    mut player_members_query: Query<&mut PlayerMembers, >,
    mut prev_player_group: Local<EntityHashMap<Option<Entity>>>,
    mut prev_mine_groups: Local<EntityHashSet>,
    mut touched_players: Local<EntityHashSet>,
    mut initialized: Local<bool>,
) {
    let removed_faction_ref = removed_faction_ref.read();
    let removed_member_of = removed_member_of.read();
    let removed_mine = removed_mine.read();

    touched_players.clear();
    touched_players.reserve(
        removed_faction_ref.size_hint().1.unwrap_or(removed_faction_ref.size_hint().0)
        + removed_member_of.size_hint().1.unwrap_or(removed_member_of.size_hint().0)
        + removed_mine.size_hint().1.unwrap_or(removed_mine.size_hint().0)
    );
    for removed_player in removed_faction_ref {
        touched_players.insert(removed_player);
    }
    for removed_player in removed_member_of {
        touched_players.insert(removed_player);
    }
    for removed_player in removed_mine {
        touched_players.insert(removed_player);
    }

    if !*initialized {
        for (player_ent, _, _, _) in player_query.iter() {
            touched_players.insert(player_ent);
        }
        *initialized = true;
    } else {
        for (player_ent, _, faction_ref, joined_groups) in player_query.iter() {
            let faction_changed = faction_ref.is_some_and(|faction_ref| faction_ref.is_changed());
            let joined_groups_changed = joined_groups.as_ref().is_some_and(|joined_groups| joined_groups.is_changed());
            if faction_changed || joined_groups_changed {
                touched_players.insert(player_ent);
            }
        }
    }

    for player_ent in touched_players.drain() {
        if let Some(Some(old_group)) = prev_player_group.remove(&player_ent) {
            let Ok(mut members) = player_members_query.get_mut(old_group) else {
                continue;
            };
            members.remove(player_ent);
            if members.is_empty() {
                cmd.entity(old_group).try_remove::<PlayerMembers>();
            }
        }

        let Ok((_, _has_mine, faction_ref, mut member_of, )) = player_query.get_mut(player_ent) else {
            continue;
        };
        let faction_ref_ent = faction_ref.as_ref().map(|faction_ref| faction_ref.0);
        let mut member_faction = None;
        if let Some(member_of) = member_of.as_ref() {
            for group_ent in member_of.iter() {
                if faction_query.get(group_ent).is_err() {
                    continue;
                }
                member_faction = Some(group_ent);
                break;
            }
        }
        if let Some(faction_ent) = faction_ref_ent {
            if faction_query.get(faction_ent).is_err() {
                error!(target: FACTION_SYSTEM, "Player {:?} points at non-faction entity {:?} in FactionRef", player_ent, faction_ent);
                cmd.entity(player_ent).try_remove::<FactionRef>();
            }
        }
        let faction_ent = member_faction.or_else(|| faction_ref_ent.filter(|faction_ent| faction_query.get(*faction_ent).is_ok()));
        let Some(faction_ent) = faction_ent else {
            prev_player_group.insert(player_ent, None);
            continue;
        };
        if let Some(mut faction_ref) = faction_ref {
            if faction_ref.0 != faction_ent {
                faction_ref.0 = faction_ent;
            }
        } else {
            cmd.entity(player_ent).try_insert(FactionRef(faction_ent));
        }
        if let Some(member_of) = member_of.as_mut() {
            member_of.insert(faction_ent);
            let faction_groups_to_remove: Vec<_> = member_of
                .iter()
                .filter(|group_ent| *group_ent != faction_ent && faction_query.get(*group_ent).is_ok())
                .collect();
            for group_ent in faction_groups_to_remove {
                member_of.remove(group_ent);
            }
        } else {
            cmd.entity(player_ent).try_insert(JoinedGroups::single(faction_ent));
        }
        if let Ok(mut members) = player_members_query.get_mut(faction_ent) {
            members.insert(player_ent);
        } else {
            cmd.entity(faction_ent).try_insert(PlayerMembers(vec![player_ent]));
        }
        prev_player_group.insert(player_ent, Some(faction_ent));
    }

    let mut mine_groups = EntityHashSet::default();
    for (_player_ent, has_mine, faction_ref, member_of, ) in player_query.iter() {
        if !has_mine {
            continue;
        }
        let mut faction_ent = faction_ref.map(|faction_ref| faction_ref.0);
        if faction_ent.is_none() {
            if let Some(member_of) = member_of {
                for group_ent in member_of.iter() {
                    if faction_query.get(group_ent).is_err() {
                        continue;
                    }
                    faction_ent = Some(group_ent);
                    break;
                }
            }
        }
        let Some(faction_ent) = faction_ent else {
            continue;
        };
        if faction_query.get(faction_ent).is_err() {
            continue;
        }
        mine_groups.insert(faction_ent);
    }
    for mine_group in prev_mine_groups.iter().copied() {
        if mine_groups.contains(&mine_group) {
            continue;
        }
        cmd.entity(mine_group).try_remove::<Mine>();
    }
    for mine_group in mine_groups.iter().copied() {
        if prev_mine_groups.contains(&mine_group) {
            continue;
        }
        cmd.entity(mine_group).try_insert(Mine);
    }
    *prev_mine_groups = mine_groups;
}
