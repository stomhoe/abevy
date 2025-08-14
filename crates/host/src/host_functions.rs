use std::{net::{Ipv4Addr, UdpSocket}, time::SystemTime};

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::{
    RenetChannelsExt,
    netcode::{
        NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    renet::{ConnectionConfig, RenetServer},
};
use multiplayer_shared::multiplayer_shared::PROTOCOL_ID;


pub fn host_server(
    commands: &mut Commands,
    channels: Res<RepliconChannels>,
    port: Option<u16>,
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

    Ok(())
}


