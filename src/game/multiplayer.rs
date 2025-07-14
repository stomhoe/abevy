use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;

use crate::game::{beings::beings_components::Being, multiplayer::{multiplayer_events::*, multiplayer_systems::*}, player::player_components::Player, setup_menus::lobby::lobby_events::SendPlayerName};

// Module multiplayer
pub mod multiplayer_components;
mod multiplayer_systems;
pub mod multiplayer_events;
//mod multiplayer_styles;
mod multiplayer_resources;
pub struct MpPlugin;
#[allow(unused_parens, path_statements)]
impl Plugin for MpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                RepliconPlugins,
                RepliconRenetPlugins,
            ))
            .replicate::<Player>()
            .replicate_bundle::<(Player,Name)>()
            .add_client_trigger::<TransformFromClient>(Channel::Unreliable)
            .add_server_trigger::<TransformFromServer>(Channel::Unreliable)

            .add_observer(receive_transf_from_client)
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Once),
    ))
*/