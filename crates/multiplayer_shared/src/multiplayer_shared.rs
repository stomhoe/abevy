

use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::{prelude::*, shared::RepliconSharedPlugin};
use common::{common_components::*, common_states::*};
use crate::{multiplayer_events::*, multiplayer_resources::TargetJoinServer, multiplayer_shared_systems::*};


pub const PROTOCOL_ID: u64 = 7;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct HostSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((RepliconSharedPlugin::default()))
  

    .configure_sets(OnEnter(ConnectionAttempt::Triggered), (
        HostSystems.run_if(in_state(GameSetupType::AsHost)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner)),
    ))
    .configure_sets(OnEnter(AppState::StatefulGameSession), (
        HostSystems.run_if(in_state(GameSetupType::AsHost)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner)),
    ))
    .configure_sets(OnExit(AppState::StatefulGameSession), (
        HostSystems.run_if(in_state(GameSetupType::AsHost)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner)),
    ))
    .configure_sets(Update, (
        HostSystems.run_if(in_state(GameSetupType::AsHost).or(server_running)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner).or(not(server_or_singleplayer))),
    ))
    .configure_sets(FixedUpdate, (
        HostSystems.run_if(in_state(GameSetupType::AsHost).or(server_running)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner).or(not(server_or_singleplayer))),
    ))
    .add_systems(OnExit(AppState::StatefulGameSession), (
        all_clean_resources
    ))

    .init_resource::<TargetJoinServer>()

    .add_server_trigger::<HostStartedGame>(Channel::Unordered)
    
    .add_client_trigger::<SendUsername>(Channel::Ordered)


    //.replicate_once::<ActivatingChunks>()

    ;
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Once),
    ))
*/