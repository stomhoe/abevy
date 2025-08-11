#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy_replicon::prelude::server_or_singleplayer;
use common::common_states::{AppState, GamePhase};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{being_components::*, faction_resources::FactionEntityMap, faction_systems::*, game_systems::*, player::*, player_systems::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .init_resource::<KeyboardInputMappings>()
    .init_resource::<FactionEntityMap>()

    .add_systems(OnEnter(AppState::StatefulGameSession), (
        (server_or_singleplayer_setup,).run_if(server_or_singleplayer),
    ))
    .add_systems(OnEnter(GamePhase::ActiveGame), (
        (spawn_player_beings,).run_if(server_or_singleplayer),
    ))
    .add_systems(Update, (
        (set_as_self_faction, update_ofself_faction, on_control_change, react_on_control_removal).in_set(StatefulSessionSystems)
    ))
 
    .register_type::<ControlledBy>()
    .register_type::<Controls>()
    .register_type::<FollowerOf>()
    .register_type::<Followers>()
    .register_type::<CharacterCreatedBy>()
    .register_type::<CreatedCharacters>()
    .register_type::<FactionEntityMap>()
    .register_type::<HumanControlled>()

    ;
}


