use std::{net::{Ipv4Addr, SocketAddr, UdpSocket}, time::SystemTime};

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    renet::{ConnectionConfig, RenetClient, RenetServer},
};

use crate::game::{game_components::DisplayName, player::player_components::{HostPlayer, SelfPlayer}};

const PROTOCOL_ID: u64 = 7;

pub fn host_server<T: Into<String>>(
    commands: &mut Commands,
    channels: Res<RepliconChannels>,
    port: Option<u16>,
    host_name: T,
    max_clients: u8,
) -> Result {
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let port = port.unwrap_or(5000);

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))?;
    let server_config = ServerConfig {
        current_time,
        max_clients: max_clients as usize,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };
    let transport = NetcodeServerTransport::new(server_config, socket)?;

    commands.insert_resource(server);
    commands.insert_resource(transport);


    commands.spawn((
        HostPlayer,
        SelfPlayer,
        DisplayName(host_name.into()),
    ));
    Ok(())
}

pub fn stop_server(
    commands: &mut Commands,
    server: &mut RenetServer,
) {
        info!("Stopping server");
        server.disconnect_all();
        
}

pub fn join_server (
    commands: &mut Commands,
    channels: Res<RepliconChannels>,
    ip: Ipv4Addr, port: Option<u16>,
) -> Result{
    let port = port.unwrap_or(5000);
    info!("attempting connect to {ip}:{port}");
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();
    

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let client_id = current_time.as_millis() as u64;
    let server_addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

    commands.insert_resource(client);
    commands.insert_resource(transport);
    Ok(())
}

pub fn disconnect_from_server(
    commands: &mut Commands,
    client: &mut RenetClient,
) {
    info!("Disconnecting from server");
    client.disconnect();
    //commands.remove_resource::<RenetClient>();
}