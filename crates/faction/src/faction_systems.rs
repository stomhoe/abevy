#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use player::player_components::*;

#[allow(unused_imports, )]use crate::{faction_components::*, faction_resources::*, };


#[allow(unused_parens)]///todo arreglar, se puede ejecutar antes de que se le ponga Mine a nuestra faccion
pub fn set_stuff_as_self_faction(mut cmd: Commands,
    things_query: Query<
        (
            Entity,
            Ref<BelongsToFaction>,
            Has<BelongsToAPlayerFaction>,
            Has<IsAffiliatedToMyFaction>,
        ),
        Without<Player>,
    >,
    selfplayer_faction_query: Query<Ref<BelongsToFaction>, (With<Player>, With<Mine>)>,
    added_mine_query: Query<(), (With<Player>, Added<Mine>)>,
    player_factions: Query<(), With<PlayerMembers>>,

) {
    let rerun_all = !added_mine_query.is_empty()
        || selfplayer_faction_query
            .single()
            .is_ok_and(|selfplayer_faction| selfplayer_faction.is_changed());
    if !rerun_all && things_query.iter().all(|(_, faction, _, _)| !faction.is_changed()) {
        return;
    }
    let Ok(selfplayer_faction) = selfplayer_faction_query.single() else {
        error!("Failed to get my player faction");
        return;
    };
    for (thing_ent, otherthing_faction, has_player_faction, is_affiliated_to_my_faction) in things_query.iter() {
        if !rerun_all && !otherthing_faction.is_changed() {
            continue;
        }
        if player_factions.get(otherthing_faction.0).is_ok() {
            if !has_player_faction {
                cmd.entity(thing_ent).try_insert(BelongsToAPlayerFaction);
            }

            if otherthing_faction.0 == selfplayer_faction.0 {
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
pub fn update_as_belonging_to_player_faction(mut cmd: Commands,
    player_factions: Query<(&FactionThings), (Added<PlayerMembers>)>,
    faction_things: Query<&FactionThings>,
    mut removed_player_factions: RemovedComponents<PlayerMembers>,

) {
    for faction_things in player_factions.iter() {
        for thing_ent in faction_things.iter() {
            cmd.entity(thing_ent).try_insert(BelongsToAPlayerFaction);
            debug!("Entity {:?} now has BelongsToAPlayerFaction", thing_ent);
        }
    }
    for ent in removed_player_factions.read() {
        if let Ok(faction_things) = faction_things.get(ent) {
            for thing_ent in faction_things.iter() {
                cmd.entity(thing_ent).try_remove::<BelongsToAPlayerFaction>();
            }
        }
    }
}

#[allow(unused_parens)]
pub fn update_ofself_faction(mut cmd: Commands, //EL SINGLE ASE Q NO SE EJECUTE ESTE SISTEMA SI NO CAMBIÓ ASÍ Q TA BIEN
    selfplayer_query: Query<(&BelongsToFaction), (With<Player>, With<Mine>, Changed<BelongsToFaction>,)>,
    fac_query: Query<(Entity), (With<Faction>, With<Mine>)>,
) {
    if selfplayer_query.is_empty(){
        return;
    }
    let Ok(selfplayer_faction) = selfplayer_query.single() else {
        error!("More than one player with Mine");
        return;
    };
    for (faction_ent) in fac_query.iter() {
        if faction_ent != selfplayer_faction.0 {
            cmd.entity(faction_ent).try_remove::<Mine>();
        }
    }
    cmd.entity(selfplayer_faction.0).try_insert(Mine);
}

#[allow(unused_parens)]
pub fn set_player_of_faction(mut cmd: Commands,
    query: Query<(Entity, &BelongsToFaction, ), (With<Player>, Changed<BelongsToFaction>,)>,
    mut removed: RemovedComponents<BelongsToFaction>,
) {
    for (ent, &belonging_to_faction) in query.iter() {
        debug!("Setting PlayerOfFaction for entity {:?} to faction {:?}", ent, belonging_to_faction.0);
        cmd.entity(ent).try_insert(PlayerOfFaction::new(belonging_to_faction.0));
    }
    for ent in removed.read() {cmd.entity(ent).try_remove::<PlayerOfFaction>(); }
}
