use bevy::prelude::*;
use bevy_replicon::prelude::{ClientState, };
use common::common_states::*;
use game_common::game_common::*;

use crate::game_init_systems::*;



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .init_resource::<GameInitSettings>()
    .init_resource::<CommonSpawnOriginCache>()
    .add_observer(put_player_beings_on_map)

    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (load_game_init_settings, server_or_singleplayer_setup).chain()
        .run_if(
            in_state(ClientState::Disconnected)
        )
        .in_set(GameplaySystems)
    )
    .add_systems(Update, (
        host_on_player_added.run_if(in_state(ClientState::Disconnected)),
        find_common_player_spawn_origin
            .run_if(in_state(ClientState::Disconnected))
            .in_set(GameplaySystems),
        place_unpositioned_player_beings_with_cached_origin
            .run_if(in_state(ClientState::Disconnected))
            .in_set(GameplaySystems)
            .after(host_on_player_added),
    ))
    //.add_systems(Update, ())



    ;
}
