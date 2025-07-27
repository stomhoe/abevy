use std::{mem, };

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};

use crate::game::{faction::faction_components::{BelongsToFaction, Faction, BelongsToSelfPlayerFaction}, multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::*, multiplayer_utils, ConnectionAttempt}, player::player_components::{OfSelf, Player} };


pub fn receive_transf_from_client(//PROVISORIO, DEBE RECIBIR INPUTS EN REALIDAD
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

