use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use common::states::{AppState, ConnectionAttempt, GameSetupType};
use multiplayer_shared::{multiplayer_events::SendPlayerName, multiplayer_shared::ClientSystems};

use crate::client_systems::*;




#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((ClientPlugin, ClientEventPlugin, RepliconRenetClientPlugin, ))
    

    .add_systems(
        OnEnter(ConnectionAttempt::Triggered),
        (
            (attempt_join,).in_set(ClientSystems),
        ),
    )
    .add_systems(Update,(
            client_on_connect_succesful.run_if(client_just_connected),
            (client_on_connect_failed).run_if(
                in_state(GameSetupType::JoinerLobby)
                .and(in_state(ConnectionAttempt::PostAttempt))
                .and(not(client_connecting))
                .and(not(client_connected))
            ),
            client_on_disconnect.run_if((client_just_disconnected),),
            (
                client_change_operand_entities
            )
            .run_if(not(server_or_singleplayer)),
    ))
    .configure_sets(OnEnter(ConnectionAttempt::Triggered), (
        ClientSystems.run_if(in_state(GameSetupType::JoinerLobby)),
    ))
    .configure_sets(OnEnter(AppState::StatefulGameSession), (
        ClientSystems.run_if(in_state(GameSetupType::JoinerLobby)),
    ))
    .add_client_trigger::<SendPlayerName>(Channel::Ordered)

    .add_observer(client_map_server_tiling)

    .add_observer(client_map_server_sprite_cfgs)


    .add_observer(on_receive_moving_anim_from_server)

    ;
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Once),
    ))
*/