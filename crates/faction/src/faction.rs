use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt};
use game_common::game_common::StatefulSessionSystems;
use faction_shared::*;

use crate::{faction_resources::*, faction_systems::*, };


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (
            update_player_members_of_groups,
            set_stuff_as_self_faction.after(update_player_members_of_groups),
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
    .replicate::<PlayerMembers>()
    ;
}
