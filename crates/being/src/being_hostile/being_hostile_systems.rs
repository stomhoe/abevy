use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::log_targets::BEING_MELEE_SYSTEMS;
use game_common::{Dead, game_common_components::TemplEntiRef};
use std::time::Duration;

use crate::{being_interaction_zone_helper::resolve_being_interaction_zone, being_messages::*};



#[allow(unused_parens, )]
pub fn remove_dead_targets_from_hostile_chase(
    mut commands: Commands,
    dead_beings: Query<Entity, Added<Dead>>,
    hostile_chasers: Query<(Entity, &HostileChase), With<HostileChase>>,
    mut dead_targets: Local<EntityHashSet>,
) {
    dead_targets.clear();
    for dead_ent in dead_beings.iter() {
        dead_targets.insert(dead_ent);
    }
    if dead_targets.is_empty() {
        return;
    }

    for (pred_ent, hunting) in hostile_chasers.iter() {
        if dead_targets.contains(&hunting.prey) {
            commands.entity(pred_ent).try_remove::<(HostileChase, NavChasing)>();
        }
    }
}


#[allow(unused_parens, )]
pub fn sync_chasing_to_host_chase(
    mut cmd: Commands,
    hunting_predators: Query<(Entity, &HostileChase, Option<&NavChasing>, ), (Changed<HostileChase>)>,
) {
    for (pred_ent, hunting, chasing, ) in hunting_predators.iter() {
        if chasing
            .map(|chasing| chasing.target == hunting.prey)
            .unwrap_or(false)
        {
            continue;
        }
        cmd.entity(pred_ent).try_insert(NavChasing::new(hunting.prey, 1.5));
    }
}
