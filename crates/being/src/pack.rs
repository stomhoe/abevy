use std::time::Duration;

use ::being_shared::*;
#[allow(unused_imports, )]
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::{HostSystems, game_common::SimRunningSystems};
use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::pack::{pack_init_systems::*, pack_systems::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_pack,))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            ((init_packs, map_pack_id_to_entity).chain()).in_set(PackSystems),
        )
        .add_systems(
            Update,
            despawn_empty_squads.in_set(PackSystems),
        )
        .add_systems(
            Update,
            update_pack_center_pos.run_if(on_timer(Duration::from_secs_f32(1.))).in_set(PackSystems).in_set(HostSystems).in_set(SimRunningSystems),
        )

    ;
}

mod pack_init_systems;
pub mod pack_seris;
pub mod pack_systems;
