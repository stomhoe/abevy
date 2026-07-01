use bevy::prelude::*;
use ::being_shared::WallPhaserOnSpawn;
use common::common_states::*;
use game_common::game_common::*;

use crate::game_init_systems::*;



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .init_resource::<GameInitSettings>()
    .init_resource::<CommonSpawnOriginCache>()
    .init_resource::<WallPhaserOnSpawn>()
    .init_resource::<::being_shared::InvulnerableOnSpawn>()
    .add_observer(put_player_beings_on_map)

    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (load_game_init_settings, server_or_singleplayer_setup).chain()
        .in_set(GameplaySystems).in_set(HostSystems)
    )
    .add_systems(Update, (
        host_on_player_added,

        (find_common_player_spawn_origin, place_unpositioned_player_beings_with_cached_origin
        .after(host_on_player_added),
        )
    .in_set(GameplaySystems)
    ).in_set(HostSystems))
    //.add_systems(Update, ())



    ;
}
