use std::time::Duration;

#[allow(unused_imports)] use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_states::AssetLoading;

pub mod chunking_components {
    pub use ::tilemap_shared::chunking_shared_components::*;
}

pub mod macro_chunk_components;
pub mod chunking_spawn_systems;
pub mod chunking_visibility_systems;
pub mod chunking_despawn_systems;

pub use macro_chunk_components::*;
pub use chunking_spawn_systems::*;
pub use chunking_visibility_systems::*;
pub use chunking_despawn_systems::*;
pub use chunking_components::*;
pub use ::tilemap_shared::chunking_shared_resources::*;
use ::tilemap_shared::*;

use crate::{ChunkSystems, };

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (
        update_activating_chunk_positions
            .after(detect_activators_with_pos_changes)
            .run_if(on_message::<UpdateActivatedChunkPos>),
        spawn_activated_chunks
            .after(update_activating_chunk_positions)
            .run_if(on_message::<UpdateActivatedChunkPos>),//DON'T TOUCH
        on_message_signal_despawn_all_chunks
            .run_if(on_message::<ForceAllChunksDespawn>),
    ).in_set(ChunkSystems))

    .add_systems(Update, (
        make_checked_chunks_despawn_if_unreferenced
            .run_if(on_message::<CheckIfChunkShouldDespawn>)
            .after(add_activating_chunks_to_activate_chunks_around)
            .before(despawn_chunks),
        detect_activators_with_pos_changes,
        rem_outofrange_chunks_from_activators
            .after(update_activating_chunk_positions)
            .before(make_checked_chunks_despawn_if_unreferenced),
        despawn_chunks
            .after(make_checked_chunks_despawn_if_unreferenced)
            .in_set(PreChunkDespawnSystems),
    ).in_set(ChunkSystems))

    .add_systems(Update, (
        update_chunk_visib
            .after(detect_camera_change_pos_visib)
            .after(periodically_recheck_chunk_visibility)
            .run_if(on_message::<RecheckChunksVisibility>),
        detect_camera_change_pos_visib,
        periodically_recheck_chunk_visibility.run_if(on_timer(Duration::from_millis(500))),
        update_beings_within_chunk_res,
    ).in_set(ChunkSystems))

    .add_systems(Update, (
        add_activating_chunks_to_activate_chunks_around,
    ).before(make_checked_chunks_despawn_if_unreferenced))
    .init_resource::<LoadChunksAround>()
    .init_resource::<LoadedChunks>()
    .init_resource::<LoadedMacroChunks>()
    .init_resource::<BeingsInCpos>()
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), load_chunking_settings)

    .add_observer(on_chunk_despawn)

    .add_message::<CheckIfChunkShouldDespawn>()
    .add_message::<UpdateActivatedChunkPos>()
    .add_message::<RecheckChunksVisibility>()
    .add_message::<MakeChunkDespawn>()
    .add_message::<ForceAllChunksDespawn>()
    .add_message::<ChunkWithBeingsWantsDespawn>()
    .add_message::<ChunkBeingsChanged>()


    ;
}
