use std::{mem, };

use bevy_replicon_renet::{RenetServer, netcode::NetcodeServerTransport};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::common::*;
use debug_shared::DebugUiConfig;
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
        cmd.entity(settings_entity).insert((Replicated, SimTimeScale::default(), Name::new("Settings Entity"), DebugUiConfig::default()));
        settings_entity
    } else {
        cmd.spawn((SettingsEntity, SimTimeScale::default(), Name::new("Settings Entity"), DebugUiConfig::default())).id()
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
        error!(target: HOST_SYSTEMS, "Failed to get host faction hash while authorizing connected clients");
        return;
    };

    for client in &connected_clients {
        cmd.entity(client).insert((AuthorizedClient, Player, FactionRef(host_faction_hash)));
    }

    if pending_game_start.is_some() {
        game_phase.set(GamePhase::ActiveGame);
        cmd.server_trigger(ToClients {
            targets: SendTargets::All,
            message: HostStartedGameplay,
        });
        cmd.remove_resource::<PendingGameStart>();
    }

    info!(target: HOST_SYSTEMS, "Host server is now accepting connections.");
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
        error!(target: HOST_SYSTEMS, "Failed to get host faction hash for assigning to connected client");
        return;
    };

    cmd.entity(on_connected_client.entity).insert((AuthorizedClient, Player, FactionRef(host_faction_hash)));
    info!(target: HOST_SYSTEMS, "(HOST) `{}` connected", on_connected_client.entity);

}

#[allow(unused_parens)]
pub fn host_receive_client_name(
    mut on_receive_username: On<FromClient<SendUsername>>,
    mut cmd: Commands,
    existing_players: Query<(Entity, &StrId), With<Player>>,
) {
    let username = mem::take(&mut on_receive_username.event_mut().0);

    let Some(entity) = on_receive_username.client_id.entity() else {
        warn!(target: HOST_SYSTEMS, "Received username from server {:?}", on_receive_username.client_id);
        return;
    };

    if existing_players
        .iter()
        .any(|(player_ent, existing_username)| player_ent != entity && *existing_username == username)
    {
        warn!(
            target: HOST_SYSTEMS,
            "Kicking connecting client {:?} because username {:?} is already taken",
            entity,
            username
        );
        cmd.entity(entity).despawn();
        return;
    }

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
