use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt};
use common::common_states::{AppState, GamePhase};
use common::{define_entity_map_systems, entity_map_macros::*, common_components::StrId};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{faction_resources::*, faction_systems::*, faction_components::*};

define_entity_map_systems!(
    FactionEntityMap,
    StrId,
    Faction
);

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (set_stuff_as_self_faction, update_ofself_faction, update_as_belonging_to_player_faction, set_player_of_faction, 
        
        ).in_set(StatefulSessionSystems)
    ))
 
    .add_plugins((
        plugin_faction_entity_map,
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


