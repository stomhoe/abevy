use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt};

use crate::{player_components::*, player_resources::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {

    app
    .replicate::<Player>()
    .replicate::<HostPlayer>()

    .init_resource::<PlayerData>()


    ;
}
