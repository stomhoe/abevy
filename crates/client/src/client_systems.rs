
use being_shared::{ControlledBy, ControlledLocally, HumanControlled};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeDisconnectReason::*}, renet::RenetClient};
use common::{common_states::*};
use dimension::dimension_resources::DimensionEntityMap;
use multiplayer_shared::{multiplayer_events::*, multiplayer_resources::TargetJoinServer, };
use player::{player_components::*, player_resources::PlayerData};
use sprite::sprite_resources::SpriteCfgEntityMap;

// Import CameraTarget if it exists in your project, adjust the path as necessary
use tilemap::terrain_gen::terrgen_resources::{OpListEntityMap, TerrGenEntityMap};

use crate::{client_functions::*, };

use game_common::{color_sampler_resources::ColorWeightedSamplersMap, };


pub fn attempt_join(
    mut cmd: Commands, 
    channels: Res<RepliconChannels>,
    mut lobby_state: ResMut<NextState<ConnectionAttempt>>,
    target_join_server: Option<Res<TargetJoinServer>>,
    //line_edit_query: Single<&CurrentText, With<MainMenuIpLineEdit>>,
) -> Result {

    let Some(joined_server) = target_join_server else {
        error!("No address was specified for joining, aborting attempt_join");
        return Ok(());
    };


    join_server(&mut cmd, channels, joined_server.ip(), joined_server.port())?;

    lobby_state.set(ConnectionAttempt::PostAttempt);

    Ok(())
}

pub fn client_on_connect_succesful(
    mut cmd: Commands, 
    mut app_state: ResMut<NextState<AppState>>,
    player_data: Res<PlayerData>,
    
) {

    app_state.set(AppState::StatefulGameSession);
    let name = player_data.username.clone();
    info!("connected as Client {name}");

    cmd.client_trigger(SendUsername(name));

}

pub fn client_on_connect_failed(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,

    //client: Res<RenetClient>,
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
                    DisconnectedByClient => {
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
pub fn client_on_game_started(trigger: Trigger<HostStartedGame>, mut state: ResMut<NextState<GamePhase>>, 
    client: Option<Res<RenetClient>>) {
    if client.is_none() {
        return;
    }
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




#[allow(unused_parens)]
pub fn client_init_resources(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<ReplicatedAssetsSession>>,
) {

    //next_state.set(ReplicatedAssetsSession::DespawnLocalAssets);

    cmd.insert_resource(TerrGenEntityMap::default());
    cmd.insert_resource(OpListEntityMap::default());
    cmd.insert_resource(DimensionEntityMap::default());

    //next_state.set(ReplicatedAssetsSession::KeepAlive);
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
 