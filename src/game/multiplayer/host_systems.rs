use std::{mem, };

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};

use crate::game::{faction::faction_components::{BelongsToFaction, Faction, BelongsToSelfPlayerFaction}, multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::*, multiplayer_utils, ConnectionAttempt}, player::player_components::{OfSelf, Player} };


pub fn attempt_host(
    mut commands: Commands, 
    channels: Res<RepliconChannels>,
    
) -> Result {
    multiplayer_utils::host_server(&mut commands, channels, None, 3)?;
    Ok(())
}


#[allow(unused_parens)]
pub fn host_receive_client_name(mut trigger: Trigger<FromClient<SendPlayerName>>, mut commands: Commands) {

    commands.entity(trigger.client_entity).insert(mem::take(&mut trigger.event_mut().0));
}


#[allow(unused_parens, )]
pub fn host_on_player_connect(trigger: Trigger<OnAdd, ConnectedClient>, 
    mut cmd: Commands, host_faction: Single<(Entity ), (With<Faction>, With<OfSelf>)>,

) {
    info!("(HOST) `{}` connected", trigger.target());

    // TA BIEN, TODOS LOS JOINERS POR DEFECTO SON DE LA FACTION DEL HOST, SI NO ES ASÍ, AL CARGAR LA SAVEGAME SE CAMBIA?
    cmd.entity(trigger.target()).insert((Player, Replicated, BelongsToFaction(host_faction.into_inner())));
}



pub fn clean_resources(
    mut commands: Commands,
    mut lobby_state: ResMut<NextState<ConnectionAttempt>>,
){
    lobby_state.set(ConnectionAttempt::default());

    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<RenetServer>();
    commands.remove_resource::<NetcodeClientTransport>();
    commands.remove_resource::<NetcodeServerTransport>();

}

