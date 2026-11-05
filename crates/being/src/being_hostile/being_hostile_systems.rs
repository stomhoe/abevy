use ::being_shared::*;
use bevy::prelude::*;
use game_common::Dead;




#[allow(unused_parens, )]
pub fn remove_dead_targets_from_hostile_chase(
    mut commands: Commands,
    live_targets: Query<(), Without<Dead>>,
    hostile_chasers: Query<(Entity, &HostileChase)>,
) {
    for (pred_ent, hunting) in hostile_chasers.iter() {
        if live_targets.get(hunting.prey).is_err() || live_targets.get(pred_ent).is_err() {
            commands.entity(pred_ent).try_remove::<(HostileChase, NavChasing)>();
        }
    }
}


#[allow(unused_parens, )]
pub fn sync_chasing_to_host_chase(
    mut cmd: Commands,
    hunting_predators: Query<(Entity, &HostileChase, Option<&NavChasing>, ), (Changed<HostileChase>, Without<Dead>)>,
    live_targets: Query<(), Without<Dead>>,
) {
    for (pred_ent, hunting, chasing, ) in hunting_predators.iter() {
        if live_targets.get(hunting.prey).is_err() { continue; }
        if chasing
            .map(|chasing| chasing.target == hunting.prey)
            .unwrap_or(false)
        {
            continue;
        }
        cmd.entity(pred_ent).try_insert(NavChasing::new(hunting.prey, 1.5));
    }
}
