

use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::{prelude::*, shared::RepliconSharedPlugin};
use bevy_replicon_renet::RepliconRenetPlugins;
use common::{common_components::*, common_states::*};
use crate::{multiplayer_events::*, multiplayer_resources::TargetJoinServer, multiplayer_shared_systems::*};


pub const PROTOCOL_ID: u64 = 7;


// #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct HostSystems;

// #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct ClientSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((RepliconSharedPlugin::default(), ))

    .add_systems(OnExit(AppState::StatefulGameSession), (
        all_clean_resources
    ))

    .init_resource::<TargetJoinServer>()

    .add_server_event::<HostStartedGameplay>(Channel::Unordered)
    
    .add_client_event::<SendUsername>(Channel::Unordered)



    ;
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), ReplicationMode::Once),
    ))
*/