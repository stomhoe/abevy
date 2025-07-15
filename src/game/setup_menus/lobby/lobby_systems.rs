
use crate::{common::common_components::DisplayName, game::{player::player_components::Player, setup_menus::lobby::{lobby_events::SendPlayerName, JoiningState}, GamePhase, GameSetupScreen}, pregame_menus::{main_menu::main_menu_components::MainMenuIpLineEdit, PreGameState}, ui::ui_components::CurrentText, AppState};

use bevy::{prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon::server::client_entity_map;
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    renet::{ConnectionConfig, RenetClient, RenetServer},
};
use bevy_ui_text_input::TextInputContents;

use std::{mem, net::{Ipv4Addr, SocketAddr}, time::SystemTime, net::UdpSocket};



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

    commands.spawn((
        Player,
        DisplayName("Host".to_string()),
    ));

    Ok(())
}



#[allow(unused_parens)]
pub fn server_receive_player_name(mut trigger: Trigger<FromClient<SendPlayerName>>, mut commands: Commands) {

    commands.entity(trigger.client_entity).insert(mem::take(&mut trigger.event_mut().0));
}



pub fn attempt_join_lobby(
    mut commands: Commands, 
    channels: Res<RepliconChannels>,
    mut joining_state: ResMut<NextState<JoiningState>>,
    line_edit_query: Single<(&CurrentText, &MainMenuIpLineEdit)>,
) -> Result {
    let ip_port_str = line_edit_query.0.0.trim();
    let mut split = ip_port_str.split(':');
    let ip_str = split.next().ok_or("Missing IP address")?;
    let ip: Ipv4Addr = ip_str.parse().map_err(|e| format!("Invalid IP address: {}", e))?;
    let port = if let Some(port_str) = split.next() {
        port_str.parse::<u16>().map_err(|e| format!("Invalid port: {}", e))?
    } else {
        PORT
    };
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

    joining_state.set(JoiningState::PostAttempt);

    Ok(())
}

pub fn client_on_connect_successful(
    mut commands: Commands, 
    mut joining_state: ResMut<NextState<JoiningState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    app_state.set(AppState::GameSession);
    joining_state.set(JoiningState::default());

    let name = format!("Player-{}", nano_id::base64::<6>());
    info!("connected as Client {name}");
    
    commands.client_trigger(SendPlayerName(DisplayName(name)));
    
    
}

pub fn client_on_connect_failed(
    mut app_state: ResMut<NextState<AppState>>,
    mut joining_state: ResMut<NextState<JoiningState>>,
    //client: Res<RenetClient>,
) {
    joining_state.set(JoiningState::default());

    info!("Couldn't connect to server, returning to main menu");
    app_state.set(AppState::PreGame);
}

pub fn lobby_button_interaction(
    interaction_query: Query<
    (&Interaction, &LobbyButtonId),
    Changed<Interaction>,
    >,
    mut game_setup_screen: ResMut<NextState<GameSetupScreen>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_phase:  ResMut<NextState<GamePhase>>,
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    app_state.set(AppState::PreGame);
                    
                }
                LobbyButtonId::Start =>  {
                    //todo chequear si todos están listos
                    game_phase.set(GamePhase::InGame);
                },
                LobbyButtonId::CreateCharacter => {
                    game_setup_screen.set(GameSetupScreen::CharacterCreation);
                },
                LobbyButtonId::Ready => {},
                LobbyButtonId::LobbyJoinability => {},
            }
        }
    }
}

pub fn add_player_comp(trigger: Trigger<OnAdd, ConnectedClient>, mut commands: Commands) {
    info!("spawning box for `{}`", trigger.target());

    commands.entity(trigger.target()).insert((Player, Replicated));
    // commands.server_trigger(ToClients {
    //     mode: SendMode::Direct(trigger.target()),
    //     event: ConnectedEvent,
    // });
}

#[allow(unused_parens)]
pub fn display_stuff(
    mut query: Query<(&DisplayName),(With<Player>)>

) {
    // info!("Displaying players in lobby...");
    // for (display_name) in &mut query {
    //     info!("Player: {}", display_name.0);
    // }
}
