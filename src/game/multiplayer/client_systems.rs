use std::net::Ipv4Addr;

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeDisconnectReason::*}, renet::RenetClient};

use crate::{common::common_components::DisplayName, game::{dimension::dimension_resources::DimensionEntityMap, multiplayer::{multiplayer_events::*, multiplayer_utils, ConnectionAttempt}, player::player_resources::PlayerData, tilemap::terrain_gen::terrgen_resources::{OpListEntityMap, TerrGenEntityMap}}, pregame_menus::main_menu::main_menu_components::MainMenuIpLineEdit, ui::ui_components::CurrentText, AppState};

pub fn attempt_join(
    mut cmd: Commands, 
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

    //TODO despues mover a otra parte
    cmd.init_resource::<TerrGenEntityMap>();
    cmd.init_resource::<OpListEntityMap>();
    cmd.init_resource::<DimensionEntityMap>();

    multiplayer_utils::join_server(&mut cmd, channels, ip, port)?;

    lobby_state.set(ConnectionAttempt::PostAttempt);

    Ok(())
}

pub fn client_on_connect_succesful(
    mut cmd: Commands, 
    mut app_state: ResMut<NextState<AppState>>,
    player_data: Res<PlayerData>,
    
) {

    app_state.set(AppState::StatefulGameSession);
    let name = player_data.name.clone();
    info!("connected as Client {name}");

    cmd.client_trigger(SendPlayerName(DisplayName::new(name)));

}

pub fn client_on_connect_failed(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,

    //client: Res<RenetClient>,
) {
    app_state.set(AppState::PreGame);

    info!("Couldn't connect to server, returning to main menu");
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn client_on_disconnect(
    commands: Commands, 
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
