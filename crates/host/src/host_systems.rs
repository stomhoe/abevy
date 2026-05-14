use std::{mem, };

use bevy_replicon_renet::{RenetServer, netcode::NetcodeServerTransport};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::common::*;
use faction::faction_resources::FactionRef;
use faction_shared::Faction;
use multiplayer_shared::multiplayer_events::{SendUsername, AttemptHostServer, StartServerFailed, HostStartedGameplay};
use multiplayer_shared::multiplayer_resources::PendingGameStart;
use player_shared::player_components::{Mine, Player};
use time_shared::SimTimeScale;

use crate::host_functions::open_server_on_udp_socket;


pub fn attempt_host(
    start_server: On<AttemptHostServer>,
    mut cmd: Commands,
    channels: Res<RepliconChannels>,
) {

    if let Err(err) = open_server_on_udp_socket(&mut cmd, channels, start_server.event().clone(), ) {
        error!("Failed to start host server: {}", err);
        cmd.trigger(StartServerFailed { reason: err });
    }
}

pub fn on_server_start_successful(
    mut cmd: Commands,
) {

    cmd.spawn((Name::new("HOOOOOOOOOOOOOSTIIIIIIIIING"),));
    cmd.spawn((Name::new("HOOOOOOOOOOOOOSTIIIIIIIIING"),));
    cmd.trigger(SpawnSettingsEntity);

}


#[allow(unused_parens)]
pub fn on_enter_setup(
    _: On<SpawnSettingsEntity>,
    mut cmd: Commands, server_state: Res<State<ServerState>>, 
    settings_entity: Query<Entity, With<SettingsEntity>>,
    mut assets_loading_state: ResMut<NextState<AssetLoading>>,
) {
    let settings_entity =
    if let Ok(settings_entity) = settings_entity.single() {
        cmd.entity(settings_entity).insert((Replicated, SimTimeScale::default(), Name::new("Settings Entity")));
        settings_entity
    } else {
        cmd.spawn((SettingsEntity, SimTimeScale::default(), Name::new("Settings Entity"))).id()
    };
    
    if *server_state.get() == ServerState::Running {
        cmd.entity(settings_entity).insert(Replicated);
    } else{
        cmd.entity(settings_entity).try_remove::<Replicated>();
    }
    assets_loading_state.set(AssetLoading::LoadingAssetsIntoHandles);
}



#[allow(unused_parens)]
pub fn on_assets_finish_loading(
    mut cmd: Commands,
    host_faction_hash: Query<&HashId, (With<Faction>, With<Mine>)>,
    connected_clients: Query<Entity, With<ConnectedClient>>,
    pending_game_start: Option<Res<PendingGameStart>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    let Ok(&host_faction_hash) = host_faction_hash.single() else {
        error!(target: "host_systems", "Failed to get host faction hash while authorizing connected clients");
        return;
    };

    for client in &connected_clients {
        cmd.entity(client).insert((AuthorizedClient, Player, FactionRef(host_faction_hash)));
    }

    if pending_game_start.is_some() {
        game_phase.set(GamePhase::ActiveGame);
        cmd.server_trigger(ToClients {
            mode: SendMode::Broadcast,
            message: HostStartedGameplay,
        });
        cmd.remove_resource::<PendingGameStart>();
    }

    info!(target: "host_systems", "Host server is now accepting connections.");
}


#[allow(unused_parens, )]
pub fn host_on_player_connect(
    on_connected_client: On<Add, ConnectedClient>,
    mut cmd: Commands,
    host_faction_hash: Query<&HashId, (With<Faction>, With<Mine>)>,
    asset_loading_state: Res<State<AssetLoading>>,
) {
    if *asset_loading_state.get() != AssetLoading::Finished {
        return;
    }

    let Ok(&host_faction_hash) = host_faction_hash.single()
    else {
        error!(target: "host_systems", "Failed to get host faction hash for assigning to connected client");
        return;
    };

    cmd.entity(on_connected_client.entity).insert((AuthorizedClient, Player, FactionRef(host_faction_hash)));
    info!(target: "host_systems", "(HOST) `{}` connected", on_connected_client.entity);

}

#[allow(unused_parens)]
pub fn host_receive_client_name(mut on_receive_username: On<FromClient<SendUsername>>,
    mut cmd: Commands,
) {
    let username = mem::take(&mut on_receive_username.event_mut().0);

    let Some(entity) = on_receive_username.client_id.entity() else {
        warn!(target: "host_systems", "Received username from server {:?}", on_receive_username.client_id);
        return;
    };

    cmd.entity(entity).insert(username.clone());

}




pub fn server_cleanup(
    mut cmd: Commands,
    server: Option<ResMut<RenetServer>>,
) {
    debug!(target: "server_cleanup", "Cleaning up server resources");
    if let Some(mut server) = server {
        server.disconnect_all();
    }
    cmd.remove_resource::<RenetServer>();//HAY Q BORRAR LOS DOS
    cmd.remove_resource::<NetcodeServerTransport>();
}
