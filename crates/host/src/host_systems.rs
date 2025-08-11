use std::{mem, };

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{netcode::{NetcodeClientTransport, NetcodeServerTransport}, renet::{RenetClient, RenetServer}};
use common::{components::{DisplayName, EntityPrefix}, states::ConnectionAttempt};
use game::{being_components::ControlledBy, faction_components::{BelongsToFaction, Faction}, player::{OfSelf, Player}};
use multiplayer_shared::multiplayer_events::SendPlayerName;
use sprite_shared::{animation_shared::MoveAnimActive, sprite_shared::SpriteCfgEntityMap};
use multiplayer_shared::multiplayer_events::MoveStateUpdated;
use tilemap::{terrain_gen::terrgen_resources::*, tile::{tile_components::HashPosEntiWeightedSampler, tile_resources::TilingEntityMap}};

use crate::host_functions::host_server;



pub fn attempt_host(
    mut commands: Commands, 
    channels: Res<RepliconChannels>,
    
) -> Result {
    host_server(&mut commands, channels, None, 3)?;
    Ok(())
}


#[allow(unused_parens)]
pub fn host_receive_client_name(mut trigger: Trigger<FromClient<SendPlayerName>>, mut commands: Commands) {

    commands.entity(trigger.client_entity).insert(mem::take(&mut trigger.event_mut().0));
}


#[allow(unused_parens, )]
pub fn host_on_player_connect(trigger: Trigger<OnAdd, ConnectedClient>, 
    mut cmd: Commands, host_faction: Single<(Entity ), (With<Faction>, With<OfSelf>)>,
    own_tiling_map: Res<TilingEntityMap>,
    own_sprite_cfg_map: Res<SpriteCfgEntityMap>,
    samplers: Query <(Entity,), (With<HashPosEntiWeightedSampler>)>,
) -> Result {
    let client_entity = trigger.target();
    cmd.entity(client_entity).insert((Player, Replicated, BelongsToFaction(host_faction.into_inner())));
    info!("(HOST) `{}` connected", client_entity);

    // Clone the map, but filter out any samplers
    let mut filtered_map = TilingEntityMap::default();
    for (hash_id, entity) in own_tiling_map.0.iter() {
        if samplers.iter().all(|(sampler_entity, )| *entity != sampler_entity) {
            filtered_map.0.insert_with_hash(hash_id, *entity, &EntityPrefix::default())?;
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
pub fn host_on_player_added(mut cmd: Commands, query: Query<(Entity, &DisplayName),(Added<DisplayName>, With<Player>)>) {
    for (player_ent, player_name) in query.iter() {
        let being = cmd.spawn((Being, DisplayName::new(player_name.0.clone()),)).id();
        cmd.entity(player_ent).insert((CreatedCharacter(being),));
    }
}


#[allow(unused_parens)]
pub fn update_animstate_for_clients(
    mut cmd: Commands,
    connected: Query<&Player, Without<OfSelf>>,
    started_query: Query<(Entity, &MoveAnimActive, &DisplayName), (Changed<MoveAnimActive>)>,
    controller: Query<&ControlledBy>,
){
    if connected.is_empty() { return; }

    for (being_ent, &MoveAnimActive(moving), dn) in started_query.iter() {
        let event_data = MoveStateUpdated {being_ent, moving};
        if let Ok(controller) = controller.get(being_ent) {
            cmd.server_trigger(ToClients {
                mode: SendMode::BroadcastExcept(controller.client),
                event: event_data,
            });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} named {} to all clients except {:?}", moving, being_ent, dn, controller.client);
        }
        else {
            cmd.server_trigger(ToClients { mode: SendMode::Broadcast, event: event_data, });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} to all clients", moving, being_ent);
        }
    }
}


pub fn clean_resources(
    mut cmd: Commands,
    mut lobby_state: ResMut<NextState<ConnectionAttempt>>,
){
    lobby_state.set(ConnectionAttempt::default());

    cmd.remove_resource::<RenetServer>();
    cmd.remove_resource::<NetcodeServerTransport>();
    cmd.remove_resource::<TerrGenEntityMap>();
    cmd.remove_resource::<OpListEntityMap>();
}

