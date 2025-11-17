#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy_replicon::prelude::{AppRuleExt};
use common::common_states::{AppState, GamePhase};
use game_common::game_common::{GameplaySystems, StatefulSessionSystems};

use crate::{player_resources::*, player_systems::*, player_components::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    
    app
    
    // .add_systems(Update, (
        
    // ))
    .replicate::<Player>()
    .replicate::<HostPlayer>()
    
    .init_resource::<KeyboardInputMappings>()
    .init_resource::<PlayerData>()
   

    ;
}


