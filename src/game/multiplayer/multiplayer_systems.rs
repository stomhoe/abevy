use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::renet::ClientId;

use crate::game::{multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::*}, player::player_components::Player};



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