#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy_replicon::prelude::server_or_singleplayer;
use common::common_states::{AppState, GamePhase};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{game_systems::*,};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(OnEnter(AppState::StatefulGameSession), (
        (server_or_singleplayer_setup,).run_if(in_state(ClientState::Disconnected)),
    ))
    .add_systems(OnEnter(GamePhase::ActiveGame), (
        (spawn_player_beings,).run_if(in_state(ClientState::Disconnected)),
    ))
    //.add_systems(Update, ())

 

    ;
}


