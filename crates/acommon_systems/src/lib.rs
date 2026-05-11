use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common_systems::*;
use clone_children_systems::*;
use ::game_common::*;
use ::common::*;
use game_common_systems::*;
use repli_clone_systems::*;
use timer_systems::*;

mod common_systems;
mod repli_clone_systems;
mod game_common_systems;
mod clone_children_systems;
mod timer_systems;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_hash_id_from_str_id,
            add_signature_from_hash_id,
            update_img_sizes_on_load,
            sync_replicate_if_server_starts,
            clone_and_tell_server.run_if(in_state(ClientState::Connected)),
            remove_replicated_after_clone_from_client
                .run_if(in_state(ServerState::Running))
                .run_if(on_message::<FromClient<RemoveReplicated>>),
            tick_time_based_multipliers.in_set(SimRunningSystems),
            tick_timers,
            despawn_sprites_without_childof,
            set_entity_name,
            clone_templ_children_ents_for_new_instances,
        ),
    );
}
