use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use ::game_common::*;

use ::being_shared::*;

pub mod being_ai_melee_systems;
pub mod being_melee_systems;
pub mod being_predation_systems;
pub mod being_hostile_systems;

use being_ai_melee_systems::*;
use being_melee_systems::*;
use being_predation_systems::*;
use being_hostile_systems::*;
#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
    .add_systems(
    Update,
        (
            sync_predator_squad_marker,
            remove_dead_targets_from_hostile_chase,
            add_melee_target_comp_to_ai_controlled,
            update_predator_hostile_chased_targets,
            make_hunted_be_automeleed,
            sync_chasing_to_host_chase,
        ).in_set(SimRunningSystems),
    )
    .add_systems(
        Update,
        (
            (
                emit_ai_melee_attack_requests.run_if(on_timer(Duration::from_millis(30))),
                apply_melee_attack,
            ).in_set(HostSystems).in_set(SimRunningSystems),
        ),
    )
    ; 
}
