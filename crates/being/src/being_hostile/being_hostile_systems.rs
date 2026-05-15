use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::log_targets::BEING_MELEE_SYSTEMS;
use game_common::{Dead, game_common_components::TemplEntiRef};
use std::time::Duration;

use crate::{being_interaction_zone_helper::resolve_being_interaction_zone, being_messages::*};

const TEMP_AI_MELEE_ATTACK_COOLDOWN: Duration = Duration::from_secs(1);



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
