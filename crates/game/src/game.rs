use bevy::prelude::*;
use bevy_replicon::prelude::{ClientState, };
use common::common_states::{AssetLoading, GamePhase, };
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::prelude::*;



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .init_resource::<GameInitSettings>()
    .add_observer(put_player_beings_on_map)

    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (load_game_init_settings, server_or_singleplayer_setup).chain()
        .run_if(
            in_state(ClientState::Disconnected)
        )
        .in_set(GameplaySystems)
    )
    .add_systems(Update, (
        host_on_player_added.run_if(in_state(ClientState::Disconnected)),
        find_common_player_spawn_origin
            .run_if(in_state(ClientState::Disconnected))
            .in_set(GameplaySystems),
    ))
    //.add_systems(Update, ())



    ;
}
