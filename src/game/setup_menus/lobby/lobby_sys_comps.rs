
use crate::{common::common_components::DisplayName, game::{player::player_components::Player, setup_menus::lobby::lobby_events::{ SendPlayerName}, GamePhase}, pregame_menus::PreGameState, AppState};

use bevy::{prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    renet::{ConnectionConfig, RenetClient, RenetServer},
};

use std::{mem, net::{Ipv4Addr, SocketAddr}, time::SystemTime};
use std::{collections::HashMap, net::UdpSocket};


#[derive(Component)]
pub enum LobbyButtonId {
  Start,
  Leave,
  CreateCharacter,
  LobbyJoinability,
  Ready,
}

#[derive(Component)]
pub enum LobbyLineEdit {Chat, LobbyName}


#[derive(Component)]
pub enum LobbySlider {ChatHistory, Settings}

const PROTOCOL_ID: u64 = 7;
const PORT: u16 = 5000;

pub fn setup(mut commands: Commands){

}

pub fn setup_for_host(mut commands: Commands, channels: Res<RepliconChannels>) -> Result {
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT))?;
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };
    let transport = NetcodeServerTransport::new(server_config, socket)?;

    commands.insert_resource(server);
    commands.insert_resource(transport);

    Ok(())
}



#[allow(unused_parens)]
pub fn server_receive_player_name(mut trigger: Trigger<FromClient<SendPlayerName>>, mut commands: Commands) {

    commands.entity(trigger.client_entity).insert(mem::take(&mut trigger.event_mut().0));
}



pub fn attempt_join_lobby(mut commands: Commands, channels: Res<RepliconChannels>) -> Result {
    let ip = Ipv4Addr::new(127, 0, 0, 1); // Localhost for testing
    info!("connecting to {ip}:{PORT}");
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let client_id = current_time.as_millis() as u64;
    let server_addr = SocketAddr::new(std::net::IpAddr::V4(ip), PORT);
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
        //std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

pub fn setup_for_connected_client(
    mut commands: Commands, 
) {
    let name = format!("Player-{}", nano_id::base64::<6>());
    info!("connected as Client {name}");
    commands.client_trigger(SendPlayerName(DisplayName(name)));
   
    // else{
    //     info!("Client not connected.");
    //     app_state.set(AppState::PreGame);
    // }

}

pub fn on_connect_failed(
    mut app_state: ResMut<NextState<AppState>>,
    mut commands: Commands, 
) {
   
    info!("Client not connected.");
    app_state.set(AppState::PreGame);

}

pub fn lobby_button_interaction(
    interaction_query: Query<
    (&Interaction, &LobbyButtonId),
    Changed<Interaction>,
    >,
    //mut app_exit_events: EventWriter<AppExit>,
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut mp_game_state:  ResMut<NextState<GamePhase>>,
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    app_state.set(AppState::PreGame);
                    pregame_state.set(PreGameState::MainMenu);

                }
                LobbyButtonId::Start =>  {
                    //todo chequear si todos están listos
                    mp_game_state.set(GamePhase::InGame);
                },
                LobbyButtonId::CreateCharacter => {},
                LobbyButtonId::Ready => {},
                LobbyButtonId::LobbyJoinability => {},
            }
        }
    }
}

pub fn spawn_clients(trigger: Trigger<OnAdd, ConnectedClient>, mut commands: Commands) {
    // Hash index to generate visually distinctive color.
    
    info!("spawning box for `{}`", trigger.target());

    commands.entity(trigger.target()).insert((Player, Replicated));
    // commands.server_trigger(ToClients {
    //     mode: SendMode::Direct(trigger.target()),
    //     event: ConnectedEvent,
    // });
}

