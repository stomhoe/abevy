use bevy::prelude::*;
use ::being_shared::*;
use game_common::HostSystems;
use tilemap::{
    terrain::terrgen_messages::ChunkTerrainBuilt,
};

use crate::{
    wildlife_cleanup_systems::*,
    wildlife_seeding_systems::*,
    wildlife_spawning_systems::*,
};

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .init_resource::<BeingsToEnableOnChunkLoad>()
        .add_observer(on_pending_natural_spawn_unfreeze_despawn)
        .add_systems(
            Update,
            (
                seed_natural_wildlife_for_new_macro_chunks,
                activate_beings_in_first_time_loaded_chunks
                    .run_if(on_message::<ChunkTerrainBuilt>),
            )
                .chain()
                .in_set(HostSystems),
        )
    ;
}
