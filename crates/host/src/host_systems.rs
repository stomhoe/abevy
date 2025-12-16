use std::{mem, };

use being::being_components::{Being, CharacterCreatedBy, DirControlledBy, CreatedCharacters};
use faction::faction_components::{BelongsToFaction, Faction};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};
use common::{common_components::{DisplayName, EntityPrefix, StrId}, common_states::ConnectionAttempt};
use multiplayer_shared::multiplayer_events::{SendUsername, StartServer};
use player::player_components::{OfSelf, Player};
use sprite_shared::SpriteConfigStrIds;

use crate::host_functions::host_server;


pub fn attempt_host(
    start_server: On<StartServer>,
    mut cmd: Commands, 
    channels: Res<RepliconChannels>,
) -> Result {

    host_server(&mut cmd, channels, start_server.event().clone(), 3)?;
    cmd.spawn((Name::new("HOOOOOOOOOOOOOSTIIIIIIIIING"),));
    Ok(())
}



#[allow(unused_parens, )]
pub fn host_on_player_connect(on_connected_client: On<Add, ConnectedClient>, 
    mut cmd: Commands, host_faction: Single<(Entity ), (With<Faction>, With<OfSelf>)>,
) -> Result {
    
    cmd.entity(on_connected_client.entity).insert((Player, BelongsToFaction(host_faction.into_inner())));
    info!("(HOST) `{}` connected", on_connected_client.entity);



    Ok(())
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
    //TODO chequear el estado actual de la partida (new game o loaded (cargar su character si ya tiene)) y los Res<State<GamePhase>> antes de hacer esto
   
}

#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands, 
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>,
    player_query: Query<(&CreatedCharacters)>,

    host_faction: Single<(Entity), (With<Faction>, With<OfSelf>)>,
) -> Result {
    let host_faction = host_faction.into_inner();
    for (player_ent, username) in query.iter() {

        if player_query.get(player_ent).is_err() {


            cmd.spawn((Being, username.clone(), 
                DirControlledBy { client: player_ent }, 
                CharacterCreatedBy { player: player_ent },

                BelongsToFaction(host_faction.clone()),
                Transform::from_translation(Vec3::new(-400.0, 250.0, 0.0)),
                SpriteConfigStrIds::new(["humanhe0", "humanbo0"]),
                
            ));

        }else{
            //TODO ASIGNARLE SU CHARACTER SI TIENE EL MISMO OWNER
        }
    }
    Ok(())
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