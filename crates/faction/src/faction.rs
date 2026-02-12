
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::{AppRuleExt};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};
use std::time::Duration;

use crate::{faction_resources::*, faction_systems::*, faction_components::*};


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (set_stuff_as_self_faction, update_ofself_faction, update_as_belonging_to_player_faction, set_player_of_faction,
            convert_faction_strid_ref_to_ent_ref.run_if(on_timer(Duration::from_secs_f32(1.)))
        ).in_set(StatefulSessionSystems)
    ))

    .add_plugins((
        plugin_faction,
    ))

    .replicate::<Faction>()
    .replicate::<BelongsToFaction>()

    .register_type::<BelongsToFaction>()
    .register_type::<PlayerOfFaction>()
    .register_type::<PlayerMembers>()
    .register_type::<FactionThings>()
    ;
}
