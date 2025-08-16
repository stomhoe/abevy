#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy_replicon::prelude::{server_or_singleplayer, AppRuleExt};
use common::common_states::{AppState, GamePhase};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{faction_resources::*, faction_systems::*, faction_components::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .init_resource::<FactionEntityMap>()

    .add_systems(Update, (
        (set_stuff_as_self_faction, update_ofself_faction, update_as_belonging_to_player_faction, set_player_of_faction, 
        
        ).in_set(StatefulSessionSystems)
    ))
 
    .replicate::<Faction>()
    .replicate::<BelongsToFaction>()

    .register_type::<FactionEntityMap>()
    .register_type::<BelongsToFaction>()
    .register_type::<Faction>()
    .register_type::<PlayerOfFaction>()
    .register_type::<PlayerMembers>()
    .register_type::<FactionThings>()
    ;
}


