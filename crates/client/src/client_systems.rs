
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeDisconnectReason::{self, *}}, renet::RenetClient};
use common::{common_states::*};
use dimension::dimension_resources::DimensionEntityMap;
use multiplayer_shared::{multiplayer_events::*, multiplayer_resources::TargetJoinServer, };
use player::{player_components::*, player_resources::PlayerData};

// Import CameraTarget if it exists in your project, adjust the path as necessary
use tilemap::terrain_gen::terrgen_resources::{OpListEntityMap, TerrGenEntityMap};

use crate::{client_functions::*, };



pub fn attempt_join(
    event: On<JoinServer>,
    mut cmd: Commands, 
    channels: Res<RepliconChannels>,
    target_join_server: Option<Res<TargetJoinServer>>,
) -> Result {


    let Some(joined_server) = target_join_server else {
        error!("No address was specified for joining, aborting attempt_join");
        return Ok(());
    };


    join_server(&mut cmd, channels, joined_server.ip(), joined_server.port())?;


    Ok(())
}

pub fn client_on_connect_succesful(
    mut cmd: Commands, 
    mut app_state: ResMut<NextState<AppState>>,
    player_data: Res<PlayerData>,
    mut game_phase: ResMut<NextState<GamePhase>>,

    
) {

    app_state.set(AppState::StatefulGameSession);
    let name = player_data.username.clone();
    info!("connected as Client {name}");
    game_phase.set(GamePhase::Setup);


    cmd.client_trigger(SendUsername(name));

}

pub fn client_on_connect_failed(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
) {
    app_state.set(AppState::NoSession);

    warn!("Couldn't connect to server, returning to main menu");
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn client_on_disconnect(
    mut app_state: ResMut<NextState<AppState>>,
    netcode_client_transport: Option<Res<NetcodeClientTransport>>,
) {
    info!("We disconnected our client, cleaning up resources...");

    if let Some(transport) = netcode_client_transport {
        match transport.disconnect_reason() {
            Some(reason) => 
            {
                info!("Client (self) has disconnected with reason: {:?}", reason);
                match reason{
                    NetcodeDisconnectReason::DisconnectedByClient => {
                        app_state.set(AppState::NoSession);
                    },//LO DEJÉ ASÍ POR SI SE QUIERE VOLVER A INTENTAR CONECTAR A LA IP EN NETCODECLIENTTRANSPORT
                    // ConnectTokenExpired => todo!(),
                    // ConnectionTimedOut => todo!(),
                    // ConnectionResponseTimedOut => todo!(),
                    // ConnectionRequestTimedOut => todo!(),
                    // ConnectionDenied => todo!(),
                    // DisconnectedByServer => todo!(),
                    _ => {},
                }
                app_state.set(AppState::NoSession);//provisorio
            },
            None => warn!("Client (self) has disconnected without a reason"),
        }
    }
}

#[allow(unused_parens)]
pub fn client_on_game_started(trigger: On<HostStartedGameplay>, mut state: ResMut<NextState<GamePhase>>, ) {

    info!(target: "lobby", "Host started game event received, transitioning to GamePhase::ActiveGame");
    state.set(GamePhase::ActiveGame);

}


pub fn client_cleanup(
    mut commands: Commands,
    client: Option<ResMut<RenetClient>>,
){
    trace!("Cleaning up client resources...");
    if let Some(mut client) = client {
        debug!("Client disconnecting...");
        client.disconnect();
    } else {
        trace!("Client was not connected, no need to disconnect");
    }

    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();

}




// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
// #[allow(unused_parens)]
// pub fn set_activates_chunk_on_camera_target(mut cmd: Commands, 
//     mut query: Query<(Entity),(Added<CameraTarget>)>,
//     mut removed_camera_targets: RemovedComponents<CameraTarget>,
// ) {
//     // for (ent) in query.iter_mut() {
//     //     cmd.entity(ent).insert(ActivatingChunks::default());
//     // }
//     // for ent in removed_camera_targets.read() {
//     //     cmd.entity(ent).remove::<ActivatingChunks>();
//     // }no hay q borrar activatingchunks si es controlledlocally
// }


// HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

// PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES
 