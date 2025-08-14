use std::net::Ipv4Addr;

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeDisconnectReason::*}, renet::RenetClient};
use common::{common_resources::PlayerData, common_states::*};
use game::{being_components::{ControlledBy, ControlledLocally, HumanControlled}, movement_components::InputMoveVector, player::{OfSelf, Player}};
use multiplayer_shared::{multiplayer_events::*, multiplayer_resources::TargetJoinServer, };
use sprite::sprite_resources::SpriteCfgEntityMap;
use sprite_animation::sprite_animation_components::MoveAnimActive;
use tilemap::{terrain_gen::{terrgen_components::{Operand, OperationList}, terrgen_resources::{OpListEntityMap, TerrGenEntityMap}}, tile::tile_resources::TilingEntityMap};

use crate::{client_functions::*, };



pub fn attempt_join(
    mut cmd: Commands, 
    channels: Res<RepliconChannels>,
    mut lobby_state: ResMut<NextState<ConnectionAttempt>>,
    target_join_server: Option<Res<TargetJoinServer>>,
    //line_edit_query: Single<&CurrentText, With<MainMenuIpLineEdit>>,
) -> Result {

    let Some(joined_server) = target_join_server else {
        error!("No address was specified for joining, aborting attempt_join");
        return Ok(());
    };


    join_server(&mut cmd, channels, joined_server.ip(), joined_server.port())?;

    lobby_state.set(ConnectionAttempt::PostAttempt);

    Ok(())
}

pub fn client_on_connect_succesful(
    mut cmd: Commands, 
    mut app_state: ResMut<NextState<AppState>>,
    player_data: Res<PlayerData>,
    
) {

    app_state.set(AppState::StatefulGameSession);
    let name = player_data.username.clone();
    info!("connected as Client {name}");

    cmd.client_trigger(SendUsername(name));

}

pub fn client_on_connect_failed(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,

    //client: Res<RenetClient>,
) {
    app_state.set(AppState::NoSession);

    warn!("Couldn't connect to server, returning to main menu");
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

pub fn client_on_disconnect(
    mut app_state: ResMut<NextState<AppState>>,
    netcode_client_transport: Option<Res<NetcodeClientTransport>>,
) {
    info!("We disconnected our client, cleaning up resources...");

    if let Some(transport) = netcode_client_transport {
        match transport.disconnect_reason() {
            Some(reason) => 
            {
                info!("Client (self) has disconnected with reason: {:?}", reason);
                match reason{
                    DisconnectedByClient => {
                        app_state.set(AppState::NoSession);
                    },//LO DEJÉ ASÍ POR SI SE QUIERE VOLVER A INTENTAR CONECTAR A LA IP EN NETCODECLIENTTRANSPORT
                    // ConnectTokenExpired => todo!(),
                    // ConnectionTimedOut => todo!(),
                    // ConnectionResponseTimedOut => todo!(),
                    // ConnectionRequestTimedOut => todo!(),
                    // ConnectionDenied => todo!(),
                    // DisconnectedByServer => todo!(),
                    _ => {},
                }
                app_state.set(AppState::NoSession);//provisorio
            },
            None => warn!("Client (self) has disconnected without a reason"),
        }
    }
}

#[allow(unused_parens)]
pub fn client_on_game_started(trigger: Trigger<HostStartedGame>, mut state: ResMut<NextState<GamePhase>>, 
    client: Option<Res<RenetClient>>) {
    if client.is_none() {
        return;
    }
    info!(target: "lobby", "Host started game event received, transitioning to GamePhase::ActiveGame");
    state.set(GamePhase::ActiveGame);

}


pub fn client_cleanup(
    mut commands: Commands,
    client: Option<ResMut<RenetClient>>,
){
    trace!("Cleaning up client resources...");
    if let Some(mut client) = client {
        debug!("Client disconnecting...");
        client.disconnect();
    } else {
        trace!("Client was not connected, no need to disconnect");
    }

    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();

}



pub fn client_map_server_tiling(
    trigger: Trigger<TilingEntityMap>, client: Option<Res<RenetClient>>,
    mut entis_map: ResMut<ServerEntityMap>, own_map: Res<TilingEntityMap>,
) {
    if client.is_none() { return; }

    //debug!(target: "tiling_loading", "Own TilingEntityMap: \n{:?}", own_map.0);

    let TilingEntityMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {

            //debug!(target: "tiling_loading", "Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!(target: "tiling_loading", "Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}


pub fn client_receive_moving_anim(
    trigger: Trigger<MoveStateUpdated>, mut query: Query<&mut MoveAnimActive>, client: Option<Res<RenetClient>>,
) {
    if client.is_none() { return; }

    let MoveStateUpdated { being_ent, moving } = trigger.event().clone();
    info!(target: "sprite_animation", "Received moving {} for entity {:?}", moving, being_ent);

    if let Ok(mut move_anim) = query.get_mut(being_ent) {
        move_anim.0 = moving;
    } else {
        warn!("Received moving state for entity {:?} that does not exist in this client.", being_ent);
    }
}

pub fn client_map_server_sprite_cfgs(
    trigger: Trigger<SpriteCfgEntityMap>,
    client: Option<Res<RenetClient>>,
    mut entis_map: ResMut<ServerEntityMap>,
    own_map: Res<SpriteCfgEntityMap>,
) {
    if client.is_none() { return; }


    let SpriteCfgEntityMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {
            //debug!(target: "sprite_loading", "Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!(target: "sprite_loading", "Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}

#[allow(unused_parens, )]
pub fn send_move_input_to_server(
    mut cmd: Commands,  move_input: Query<(Entity, &InputMoveVector), (Changed<InputMoveVector>, With<ControlledLocally>)>,
) {
    for (being_ent, move_vec) in move_input.iter() {
        trace!(target: "movement", "Sending move input for entity {:?} with vector {:?}", being_ent, move_vec);
        cmd.client_trigger( SendMoveInput { being_ent, vec: move_vec.clone(), } );
    }

}


#[allow(unused_parens)]
pub fn client_change_operand_entities(
    mut query: Query<(&mut OperationList), (Added<OperationList>)>, 
    mut map: ResMut<ServerEntityMap>,
)
{
    for mut oplist in query.iter_mut() {
        for (operand, _) in &mut oplist.trunk {
            let Operand::Entities(entities) = operand 
            else { continue };

            let mut new_entities = Vec::with_capacity(entities.len());
            for ent in entities.iter() {
                if let Some(new_ent) = map.server_entry(*ent).get() {
                    new_entities.push(new_ent);
                } else {
                    error!(target: "oplist_loading", "Entity {} not found in ServerEntityMap", ent);
                    new_entities.push(Entity::PLACEHOLDER);
                }
            }
            *operand = Operand::Entities(new_entities);
        
        }
    }
}

#[allow(unused_parens)]
pub fn on_receive_transf_from_server(//TODO REHACER TODO ESTO CON ALGUNA CRATE DE INTERPOLATION/PREDICTION/ROLLBACK/LOQSEA
    trigger: Trigger<TransformFromServer>, client: Option<Res<RenetClient>>,
    mut being_query: Query<(&mut Transform, &ControlledBy, &HumanControlled)>,
    selfplayer: Single<(Entity), (With<OfSelf>, With<Player>)>,
) -> Result {
    let TransformFromServer { being: entity, trans: transform, interpolate } = trigger.event().clone();

    if client.is_none() {return Ok(());}

    let Ok((mut transf, controller, human_controlled)) = being_query.get_mut(entity) else {
        let err = Err(BevyError::from(format!("Received transform for entity that does not exist: {:?}", entity)));
        return err;
    };
    
    //debug!("Applying transform to entity: {:?}", entity);
    if controller.client == selfplayer.into_inner() && interpolate && human_controlled.0 {
        transf.translation = transf.translation.lerp(transform.translation, 0.5);
    } else {
        *transf = transform;
    }
    
   Ok(())
}

#[allow(unused_parens)]
pub fn client_init_resources(mut cmd: Commands, ) {
    cmd.init_resource::<TerrGenEntityMap>();
    cmd.init_resource::<OpListEntityMap>();
}

// HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

// PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES
