use bevy::prelude::*;
use game_common::HostSystems;
use tilemap::{
    terrain::terrgen_messages::ChunkTerrainBuilt,
};

use crate::{wildlife_resources::*, wildlife_spawning_systems::*};

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .init_resource::<NaturalSpawnReservationIndex>()
        .init_resource::<SeededNaturalWildlifeMacroChunks>()
        .add_systems(Update, cleanup_pending_natural_wildlife_index.in_set(HostSystems))
        .add_systems(
            Update,
            (
                spawn_natural_wildlife_for_macro_chunk,
                unfreeze_natural_wildlife_for_first_time_loaded_chunks
                    .run_if(on_message::<ChunkTerrainBuilt>),
            )
                .chain()
                .in_set(HostSystems),
        )
    ;
}
