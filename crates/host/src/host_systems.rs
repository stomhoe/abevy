use std::{mem, };

use sprite_animation_shared::sprite_animation_shared::MoveAnimActive;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};
use common::{common_components::{DisplayName, EntityPrefix, StrId}, common_states::ConnectionAttempt};
use game::{being_components::{Being, ControlledBy}, faction_components::{BelongsToFaction, Faction}, player::{CharacterCreatedBy, CreatedCharacters, OfSelf, Player}};
use multiplayer_shared::multiplayer_events::SendUsername;
use multiplayer_shared::multiplayer_events::MoveStateUpdated;
use sprite::{sprite_components::SpriteConfigStringIds, sprite_resources::SpriteCfgEntityMap};
use tilemap::{terrain_gen::terrgen_resources::*, tile::{tile_components::HashPosEntiWeightedSampler, tile_resources::TilingEntityMap}};

use crate::host_functions::host_server;



pub fn attempt_host(
    mut commands: Commands, 
    channels: Res<RepliconChannels>,
    
) -> Result {
    host_server(&mut commands, channels, None, 3)
}




#[allow(unused_parens, )]
pub fn host_on_player_connect(trigger: Trigger<OnAdd, ConnectedClient>, 
    mut cmd: Commands, host_faction: Single<(Entity ), (With<Faction>, With<OfSelf>)>,
    own_tiling_map: Res<TilingEntityMap>,
    own_sprite_cfg_map: Res<SpriteCfgEntityMap>,
    samplers: Query <(Entity,), (With<HashPosEntiWeightedSampler>)>,
) -> Result {
    let client_entity = trigger.target();
    cmd.entity(client_entity).insert((Player, BelongsToFaction(host_faction.into_inner())));
    info!("(HOST) `{}` connected", client_entity);

    // Clone the map, but filter out any samplers
    let mut filtered_map = TilingEntityMap::default();
    for (&hash_id, &entity) in own_tiling_map.0.iter() {
        if samplers.iter().all(|(sampler_entity, )| entity != sampler_entity) {
            filtered_map.0.insert_with_hash(hash_id, entity, )?;
        }
    }

    let sync_tiles = ToClients { mode: SendMode::Direct(client_entity), event: filtered_map, };
    cmd.server_trigger(sync_tiles);

    let sync_sprite_cfgs = ToClients { mode: SendMode::Direct(client_entity), event: own_sprite_cfg_map.clone(),};
    cmd.server_trigger(sync_sprite_cfgs);
        // TA BIEN, TODOS LOS JOINERS POR DEFECTO SON DE LA FACTION DEL HOST, SI NO ES ASÍ, AL CARGAR LA SAVEGAME SE CAMBIA?
    Ok(())
}

#[allow(unused_parens)]
pub fn host_receive_client_name(mut trigger: Trigger<FromClient<SendUsername>>, 
    mut cmd: Commands, 
) {
    let username = mem::take(&mut trigger.event_mut().0);
    cmd.entity(trigger.client_entity).insert(username.clone());
    //TODO chequear el estado actual de la partida (new game o loaded (cargar su character si ya tiene)) y los Res<State<GamePhase>> antes de hacer esto
   
}

#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands, 
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>,
    player_query: Query<(&CreatedCharacters)>,

    host_faction: Single<(Entity), (With<Faction>, With<OfSelf>)>,
) {
    let host_faction = host_faction.into_inner();
    for (player_ent, username) in query.iter() {

        if player_query.get(player_ent).is_err() {


            cmd.spawn((Being, username.clone(), 
                ControlledBy { client: player_ent }, 
                CharacterCreatedBy { player: player_ent },
                
                BelongsToFaction(host_faction.clone()),
                Transform::default(),
                SpriteConfigStringIds::new(["humanhe0", "humanbo0"]),
                
            ));

        }else{
            //TODO ASIGNARLE SU CHARACTER SI TIENE EL MISMO OWNER
        }
    }
}


#[allow(unused_parens)]
pub fn update_animstate_for_clients(
    mut cmd: Commands,
    connected: Query<&Player, Without<OfSelf>>,
    started_query: Query<(Entity, &MoveAnimActive, Option<&StrId>), (Changed<MoveAnimActive>)>,
    controller: Query<&ControlledBy>,
){
    if connected.is_empty() { return; }

    for (being_ent, &MoveAnimActive(moving), id) in started_query.iter() {
        let event_data = MoveStateUpdated {being_ent, moving};
        if let Ok(controller) = controller.get(being_ent) {
            cmd.server_trigger(ToClients {
                mode: SendMode::BroadcastExcept(controller.client),
                event: event_data,
            });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} {} to all clients except {:?}", moving, being_ent, id.cloned().unwrap_or_default(), controller.client);
        }
        else {
            cmd.server_trigger(ToClients { mode: SendMode::Broadcast, event: event_data, });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} to all clients", moving, being_ent);
        }
    }
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