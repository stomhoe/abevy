#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::{faction::faction_components::{BelongsToFaction, BelongsToSelfPlayerFaction, Faction}, player::player_components::{OfSelf, Player}};

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn set_as_self_faction(mut cmd: Commands, //NO USAR CHANGED O ADDED EN NINGUNA DE ESTAS QUERIES
    selfplayer_query: Single<(&BelongsToFaction), (With<Player>, With<OfSelf>, Changed<BelongsToFaction>,)>,
    others_query: Query<(Entity, &BelongsToFaction), (Changed<BelongsToFaction>, )>,
    fac_query: Query<(Entity, ), (Changed<Faction>, )>,

) {
    let selfplayer_faction = selfplayer_query;
    info!("Setting self player faction: {:?}", selfplayer_faction.0);
    for (thing_ent, otherthing_faction) in others_query.iter() {
        if otherthing_faction.0 == selfplayer_faction.0 {
            cmd.entity(thing_ent).insert_if_new(BelongsToSelfPlayerFaction);
            info!("Entity {:?} is now part of the self player faction: {:?}", thing_ent, otherthing_faction.0);
        } else {
            cmd.entity(thing_ent).remove::<BelongsToSelfPlayerFaction>();
            info!("Entity {:?} removed from self player faction: {:?}", thing_ent, otherthing_faction.0);
        }
    }
    for (fac_ent, ) in fac_query.iter() {
        if selfplayer_faction.0 == fac_ent {
            cmd.entity(fac_ent).insert_if_new(OfSelf);
            info!("Faction {:?} set as self", fac_ent);
        } else {
            cmd.entity(fac_ent).remove::<OfSelf>();
            info!("Faction {:?} unset as self", fac_ent);
        }
    }
}
