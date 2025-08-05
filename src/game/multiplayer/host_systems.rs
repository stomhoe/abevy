use std::{mem, };

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};

use crate::{common::common_components::EntityPrefix, game::{faction::faction_components::{BelongsToFaction, BelongsToSelfPlayerFaction, Faction}, multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::*, multiplayer_utils, ConnectionAttempt}, player::player_components::{OfSelf, Player}, tilemap::tile::{tile_components::HashPosEntiWeightedSampler, tile_resources::AnyTilingEntityMap} }};


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
    own_tiling_map: Res<AnyTilingEntityMap>,
    samplers: Query <(Entity,), (With<HashPosEntiWeightedSampler>)>,
) -> Result {
    let client_entity = trigger.target();
    cmd.entity(client_entity).insert((Player, Replicated, BelongsToFaction(host_faction.into_inner())));
    info!("(HOST) `{}` connected", client_entity);

    // Clone the map, but filter out any samplers
    let mut filtered_map = AnyTilingEntityMap::default();
    for (hash_id, entity) in own_tiling_map.0.iter() {
        if samplers.iter().all(|(sampler_entity, )| *entity != sampler_entity) {
            filtered_map.0.insert_with_hash(hash_id, *entity, &EntityPrefix::default())?;
        }
    }

    let sync_tiles = ToClients { 
        mode: SendMode::Direct(client_entity), 
        event: filtered_map,
    };
    cmd.server_trigger(sync_tiles);
        // TA BIEN, TODOS LOS JOINERS POR DEFECTO SON DE LA FACTION DEL HOST, SI NO ES ASÍ, AL CARGAR LA SAVEGAME SE CAMBIA?
    Ok(())
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

