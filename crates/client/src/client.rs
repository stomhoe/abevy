use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use common::common_states::{AppState, ConnectionAttempt, GameSetupType};
use multiplayer_shared::{multiplayer_events::SendUsername, multiplayer_shared::ClientSystems};

use crate::client_systems::*;




#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((ClientPlugin, ClientEventPlugin, RepliconRenetClientPlugin, ))
    

    .add_systems(
        OnEnter(ConnectionAttempt::Triggered),
        (
            client_init_resources.before(attempt_join),
            attempt_join,
        ).in_set(ClientSystems),
    )
    .add_systems(
        OnExit(AppState::StatefulGameSession),
        (
            client_cleanup,
        ).in_set(ClientSystems),
    )
    .add_systems(Update,(
        client_on_connect_succesful.run_if(client_just_connected),
        (client_on_connect_failed).run_if(
            in_state(GameSetupType::AsJoiner)
            .and(in_state(ConnectionAttempt::PostAttempt))
            .and(not(client_connecting))
            .and(not(client_connected))
        ),
        client_on_disconnect.run_if((client_just_disconnected),),

        // (
        // )
        // .in_set(ClientSystems),
    ))
   

    .add_observer(client_on_game_started)

    ;
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Once),
    ))
*/