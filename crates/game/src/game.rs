#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy_replicon::prelude::{ClientState, };
use common::common_states::{AppState, AssetsLoadingState, GamePhase, TerrainHotReloading};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{game_init_systems::*,};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(
        OnEnter(AssetsLoadingState::ReplicatedFinished),
        (server_or_singleplayer_setup,)
        .run_if(
            in_state(ClientState::Disconnected)
            .and(not(in_state(TerrainHotReloading::DespawnAll)))
        )
        .in_set(GameplaySystems)
    )
    .add_systems(
        OnEnter(GamePhase::ActiveGame),
        (spawn_player_beings,)
        .run_if(in_state(ClientState::Disconnected))
        .in_set(GameplaySystems)
    )
    //.add_systems(Update, ())

 

    ;
}


