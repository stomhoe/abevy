use bevy::prelude::*;

use game_common::HostSystems;
use tilemap::chunking::{despawn_chunks, rem_outofrange_chunks_from_activators, ChunkWithBeingsWantsDespawn};

use crate::{being_on_chunk_despawn_systems::on_chunk_with_beings_attempt_unload, prelude::MakeChunkSnapshotForChaser};

pub mod being_nav_resources;
pub mod being_nav_structs;
pub mod being_nav_components;
pub mod being_nav_helpers;
pub mod being_nav_systems;
pub mod being_nav_wander_systems;
pub mod being_nav_chase_systems;

pub use being_nav_resources::*;
pub use being_nav_structs::*;
pub use being_nav_components::*;
pub use being_nav_helpers::*;
pub use being_nav_systems::*;
pub use being_nav_wander_systems::*;
pub use being_nav_chase_systems::*;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_ai_nav_grids,
            rebuild_chaser_nav_plans,
            chase_behavior,

            make_chunk_snapshot_for_hunter
                .after(on_chunk_with_beings_attempt_unload)
                .in_set(HostSystems)
                .run_if(on_message::<MakeChunkSnapshotForChaser>),
            dynamically_extend_retained_chasepaths_due_to_moving_player_prey,
            wander_behavior,
        )
    )
    .add_systems(
        Update,
        (
            cleanup_player_chase_chunk_retention,
            retain_chunks_for_player_faction_chasers,
        )
            .chain()
            .after(rem_outofrange_chunks_from_activators)
            .before(despawn_chunks)
            .in_set(HostSystems)
    );
}
