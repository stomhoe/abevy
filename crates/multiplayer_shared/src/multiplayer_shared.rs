

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::{prelude::*, shared::{AuthMethod, RepliconSharedPlugin}};
use common::{common_states::*};
use crate::*;


pub const PROTOCOL_ID: u64 = 7;


#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((RepliconSharedPlugin { auth_method: AuthMethod::Custom }, ))

    .add_systems(OnExit(AppState::StatefulGameSession), (
        all_clean_resources
    ))

    .init_resource::<TargetJoinServer>()

    .add_server_event::<HostStartedGameplay>(Channel::Unordered)

    .add_client_event::<SendUsername>(Channel::Unordered)



    ;
}
