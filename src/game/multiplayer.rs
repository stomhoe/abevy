use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;

// Module multiplayer
mod multiplayer_components;
mod multiplayer_systems;
mod multiplayer_events;
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
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}