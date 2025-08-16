#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use player::player_components::*;
use tilemap::chunking_components::ActivatingChunks;

use crate::{faction_components::*, };

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn set_stuff_as_self_faction(mut cmd: Commands, 
    thingss_query: Query<(Entity, &BelongsToFaction), (Changed<BelongsToFaction>, )>,
    selfplayer_query: Single<(&BelongsToFaction), (With<Player>, With<OfSelf>, )>,
    player_factions: Query<(&PlayerMembers),>,

) {
    let selfplayer_faction = selfplayer_query.into_inner();
    for (thing_ent, otherthing_faction) in thingss_query.iter() {
        if player_factions.get(otherthing_faction.0).is_ok() {
            cmd.entity(thing_ent).try_insert(BelongsToAPlayerFaction);

            if otherthing_faction.0 == selfplayer_faction.0 {
                cmd.entity(thing_ent).try_insert_if_new(IsAffiliatedToMyFaction);
            } else {
                cmd.entity(thing_ent).try_remove::<IsAffiliatedToMyFaction>();
            }
        } else{
            cmd.entity(thing_ent).try_remove::<BelongsToAPlayerFaction>();
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
        for &thing_ent in faction_things.entities() {
            cmd.entity(thing_ent).try_insert(BelongsToAPlayerFaction);
            debug!("Entity {:?} now has BelongsToAPlayerFaction", thing_ent);
        }
    }
    for ent in removed_player_factions.read() {
        if let Ok(faction_things) = faction_things.get(ent) {
            for &thing_ent in faction_things.entities() {
                cmd.entity(thing_ent).try_remove::<BelongsToAPlayerFaction>();
            }
        }
    }
}

#[allow(unused_parens)]
pub fn update_ofself_faction(mut cmd: Commands, //EL SINGLE ASE Q NO SE EJECUTE ESTE SISTEMA SI NO CAMBIÓ ASÍ Q TA BIEN
    selfplayer_query: Single<(&BelongsToFaction), (With<Player>, With<OfSelf>, Changed<BelongsToFaction>,)>,
    fac_query: Single<(Entity, ), (With<Faction>, With<OfSelf>)>,
) {
    cmd.entity(fac_query.0).try_remove::<OfSelf>();
    let selfplayer_faction = selfplayer_query.into_inner();
    cmd.entity(selfplayer_faction.0).insert(OfSelf);
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
