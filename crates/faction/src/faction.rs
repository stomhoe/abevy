use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{faction_resources::*, faction_systems::*, faction_components::*};


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (set_stuff_as_self_faction, update_ofself_faction, update_as_belonging_to_player_faction, set_player_of_faction, 
        
        ).in_set(StatefulSessionSystems)
    ))
 
    .add_plugins((
        plugin_faction,
    ))

    .replicate::<Faction>()
    .replicate::<BelongsToFaction>()

    .register_type::<BelongsToFaction>()
    .register_type::<Faction>()
    .register_type::<PlayerOfFaction>()
    .register_type::<PlayerMembers>()
    .register_type::<FactionThings>()
    ;
}


