use std::net::Ipv4Addr;

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeDisconnectReason::*}, renet::RenetClient};

use crate::{common::common_components::DisplayName, game::multiplayer::{multiplayer_events::*, multiplayer_utils, ConnectionAttempt}, pregame_menus::main_menu::main_menu_components::MainMenuIpLineEdit, ui::ui_components::CurrentText, AppState};

pub fn attempt_join(
    mut commands: Commands, 
    channels: Res<RepliconChannels>,
    mut lobby_state: ResMut<NextState<ConnectionAttempt>>,
    line_edit_query: Single<&CurrentText, With<MainMenuIpLineEdit>>,
) -> Result {
    let ip_port_str = line_edit_query.0.trim();
    let mut split = ip_port_str.split(':');
    let ip_str = split.next().ok_or("Missing IP address")?;
    let ip: Ipv4Addr = ip_str.parse().map_err(|e| format!("Invalid IP address: {}", e))?;

    let port = if let Some(port_str) = split.next() {
        Some(port_str.parse::<u16>().map_err(|e| format!("Invalid port: {}", e))?)
    } else {
        None
    };
    multiplayer_utils::join_server(&mut commands, channels, ip, port)?;

    lobby_state.set(ConnectionAttempt::PostAttempt);

    Ok(())
}

pub fn client_on_connect_successful(
    mut commands: Commands, 
    mut app_state: ResMut<NextState<AppState>>,
) {
    app_state.set(AppState::StatefulGameSession);

    let name = format!("Player-{}", nano_id::base64::<6>());
    info!("connected as Client {name}");
    
    commands.client_trigger(SendPlayerName(DisplayName(name)));
}

pub fn client_on_connect_failed(
    mut commands: Commands,
    //client: Res<RenetClient>,
) {

    info!("Couldn't connect to server, returning to main menu");
   
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn client_on_disconnect(
    mut commands: Commands, 
    mut app_state: ResMut<NextState<AppState>>,
    netcode_client_transport: Option<Res<NetcodeClientTransport>>,
) {
    info!("Client (self) has disconnected, cleaning up resources...");

    if let Some(transport) = netcode_client_transport {
        match transport.disconnect_reason() {
            Some(reason) => 
            {
                info!("Client (self) has disconnected with reason: {:?}", reason);
                match reason{
                    DisconnectedByClient => {
                        app_state.set(AppState::PreGame);
                    },//LO DEJÉ ASÍ POR SI SE QUIERE VOLVER A INTENTAR CONECTAR A LA IP EN NETCODECLIENTTRANSPORT
                    // ConnectTokenExpired => todo!(),
                    // ConnectionTimedOut => todo!(),
                    // ConnectionResponseTimedOut => todo!(),
                    // ConnectionRequestTimedOut => todo!(),
                    // ConnectionDenied => todo!(),
                    // DisconnectedByServer => todo!(),
                    _ => {},
                }
                app_state.set(AppState::PreGame);//provisorio
            },
            None => info!("Client (self) has disconnected without a reason"),
        }
    }
}