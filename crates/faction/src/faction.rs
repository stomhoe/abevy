
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::{AppRuleExt};
use game_common::game_common::StatefulSessionSystems;
use std::time::Duration;
use faction_shared::*;

use crate::{faction_resources::*, faction_systems::*, };


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (update_player_members_of_groups, set_stuff_as_self_faction,
            convert_faction_strid_ref_to_ent_ref.run_if(on_timer(Duration::from_secs_f32(1.)))
        ).in_set(StatefulSessionSystems)
    ))

    .add_plugins((
        plugin_faction,
        crate::culture::plugin,
        crate::faction_inst_templ::plugin,
    ))

    .replicate::<Faction>()
    .replicate::<FactionInstTempl>()
    .replicate::<FactionRef>()
    ;
}
