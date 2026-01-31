use std::time::Duration;

#[allow(unused_imports)] use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;

pub mod chunking_components;
pub mod chunking_resources;
pub mod chunking_spawn_systems;
pub mod chunking_visibility_systems;
pub mod chunking_despawn_systems;

pub use chunking_components::*;
pub use chunking_resources::*;
pub use chunking_spawn_systems::*;
pub use chunking_visibility_systems::*;
pub use chunking_despawn_systems::*;
use tilemap_shared::{ChunkPos, ForceAllChunksDespawn};

use crate::{ChunkSystems, tile::tile_systems::despawn_if_not_excepted};

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (
    (
        periodically_recheck_chunk_visibility.run_if(on_timer(Duration::from_millis(500))),
        activate_chunks_every_second,
        detect_camera_change_pos, 
        update_chunk_visib,
        periodically_check_despawn_unreferenced_chunks.run_if(on_timer(Duration::from_secs(2))),
        on_message_signal_despawn_all_chunks,
        detect_activators_with_pos_changes, 
        spawn_chunks_around_activators.after(despawn_chunks).after(despawn_if_not_excepted),//DON'T TOUCH

    ).in_set(ChunkSystems),
        despawn_chunks, 
        rem_outofrange_chunks_from_activators, 

    ))
    .register_type::<LoadedChunks>()
    .register_type::<ActivatingChunks>()
    .register_type::<ChunkPos>()
    .register_type::<AaChunkRangeSettings>()
    .init_resource::<AaChunkRangeSettings>()
    .init_resource::<LoadedChunks>()

    .add_message::<CheckChunkDespawn>()
    .add_message::<ReactivateChunksFor>()
    .add_message::<RecheckChunksVisibility>()
    .add_message::<ForceChunkDespawn>()
    .add_message::<ForceAllChunksDespawn>()

    ;
}
