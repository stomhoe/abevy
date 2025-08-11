#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use crate::{being_components::Being, faction_components::*, player::{OfSelf, Player}};

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn set_as_self_faction(mut cmd: Commands, //NO USAR CHANGED O ADDED EN NINGUNA DE ESTAS QUERIES
    others_query: Query<(Entity, &BelongsToFaction), (Changed<BelongsToFaction>)>,
    selfplayer_query: Single<(&BelongsToFaction), (With<Player>, With<OfSelf>, With<BelongsToFaction>,)>,
) {
    let selfplayer_faction = selfplayer_query;
    for (thing_ent, otherthing_faction) in others_query.iter() {
        if otherthing_faction.0 == selfplayer_faction.0 {
            cmd.entity(thing_ent).insert_if_new(BelongsToSelfPlayerFaction);
        } else {
            cmd.entity(thing_ent).remove::<BelongsToSelfPlayerFaction>();
        }
    }

}

#[allow(unused_parens)]
pub fn update_ofself_faction(mut cmd: Commands, //NO USAR CHANGED O ADDED EN NINGUNA DE ESTAS QUERIES
    fac_query: Query<(Entity, ), (With<Faction>, With<OfSelf>)>,
    selfplayer_query: Single<(&BelongsToFaction), (With<Player>, With<OfSelf>, Changed<BelongsToFaction>,)>,
) {
    for (fac_ent, ) in fac_query.iter() { cmd.entity(fac_ent).remove::<OfSelf>(); }
    let &BelongsToFaction(selfplayer_faction) = selfplayer_query.into_inner();
    cmd.entity(selfplayer_faction).insert(OfSelf);
}


// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
// A L CENTRO DE LA BASE VA A HABER Q PONERLE UNO DE ALGUNA FORMA
pub fn add_activates_chunks(mut cmd: Commands, 
    mut query: Query<(Entity),(With<Being>, With<BelongsToSelfPlayerFaction>)>,
) {
    
    for ent in query.iter_mut() {
        
    }
}
