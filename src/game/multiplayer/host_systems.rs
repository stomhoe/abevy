use std::{mem, };

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};

use crate::game::{multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::*, multiplayer_utils, ConnectionAttempt}, player::player_components::Player, };


pub fn receive_transf_from_client(
    trigger: Trigger<FromClient<TransformFromClient>>,
    mut commands: Commands,
    mut query: Query<(&MpAuthority, &mut Transform,)>,
) {
   let (mp_auth, mut transf) = query.get_mut(trigger.entity).unwrap();

   if mp_auth.0 == trigger.client_entity {
        if transf.translation != trigger.transf.translation || transf.rotation != trigger.transf.rotation || transf.scale != trigger.transf.scale{
            commands.entity(trigger.entity).insert(trigger.transf.clone());

            commands.server_trigger(
                ToClients { mode: SendMode::BroadcastExcept(trigger.client_entity), event: TransformFromServer::from(trigger.event().event.clone()) },
                
            );
        }

   }
}

pub fn attempt_host(
    mut commands: Commands, 
    channels: Res<RepliconChannels>,
    
) -> Result {
    multiplayer_utils::host_server(&mut commands, channels, None, "host", 3)?;
    Ok(())
}


#[allow(unused_parens)]
pub fn host_receive_client_name(mut trigger: Trigger<FromClient<SendPlayerName>>, mut commands: Commands) {

    commands.entity(trigger.client_entity).insert(mem::take(&mut trigger.event_mut().0));
}



pub fn host_on_player_connect(trigger: Trigger<OnAdd, ConnectedClient>, mut commands: Commands) {
    info!("(HOST) `{}` connected", trigger.target());

    commands.entity(trigger.target()).insert((Player, Replicated));
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

