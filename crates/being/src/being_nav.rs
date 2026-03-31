use bevy::prelude::*;

use game_common::HostSystems;
use superstate::superstate_plugin;
use tilemap::chunking::{despawn_chunks, rem_outofrange_chunks_from_activators};

use ::being_shared::*;
use crate::being_messages::NavOrder;
use crate::being_messages::MakeChunkSnapshotForChaser;
use crate::being_on_chunk_despawn_systems::on_chunk_with_beings_attempt_unload as on_chunk_with_beings_attempt_unload_system;

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
    app
    .add_plugins(superstate_plugin::<BehavorialNavState, (WanderState, Chasing, Fleeing)>)
    .add_systems(
        Update,
        (
            (
                ensure_loaded_beings_have_nav_state,
                update_goto_from_chasing,
                update_goto_from_fleeing,
                wander_behavior,
                apply_nav_orders.run_if(on_message::<NavOrder>),
                clear_nav_outputs_for_beings_without_nav_state,
                sync_ai_nav_grids,
                rebuild_goto_nav_plans,
                goto_behavior,
            ).chain(),

            make_chunk_snapshot_for_hunter
                .after(on_chunk_with_beings_attempt_unload_system)
                .in_set(HostSystems)
                .run_if(on_message::<MakeChunkSnapshotForChaser>),
            dynamically_extend_retained_chasepaths_due_to_moving_player_prey,
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
