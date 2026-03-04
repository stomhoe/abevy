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
use ::tilemap_shared::*;

use crate::{ChunkSystems, };

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (
    (
        //spawn systems
        spawn_chunks_around_activators
            .after(despawn_chunks)
            .run_if(on_message::<ReactivateChunksFor>),//DON'T TOUCH
        activate_chunks_every_second,
        on_message_signal_despawn_all_chunks
            .run_if(on_message::<ForceAllChunksDespawn>),

        //despawn systems
        periodically_check_despawn_unreferenced_chunks.run_if(on_timer(Duration::from_secs(2))),
        detect_activators_with_pos_changes,

        //visibility systems
        update_chunk_visib
            .after(detect_camera_change_pos_visib)
            .after(periodically_recheck_chunk_visibility)
            .run_if(on_message::<RecheckChunksVisibility>),
        detect_camera_change_pos_visib,
        periodically_recheck_chunk_visibility.run_if(on_timer(Duration::from_millis(500))),

    ).in_set(ChunkSystems),
        despawn_chunks.after(PreChunkDespawnSystems),
        rem_outofrange_chunks_from_activators,

    ))
    .init_resource::<AaChunkRangeSettings>()
    .init_resource::<LoadedChunks>()

    .add_observer(on_chunk_despawn)

    .add_message::<CheckChunkDespawn>()
    .add_message::<ReactivateChunksFor>()
    .add_message::<RecheckChunksVisibility>()
    .add_message::<ForceChunkDespawn>()
    .add_message::<ForceAllChunksDespawn>()

    ;
}
