use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::log_targets::BEING_MELEE_SYSTEMS;
use game_common::{Dead, game_common_components::TemplEntiRef};
use std::time::Duration;

use crate::{being_interaction_zone_helper::resolve_being_interaction_zone, being_messages::*};

const TEMP_AI_MELEE_ATTACK_COOLDOWN: Duration = Duration::from_secs(1);



#[allow(unused_parens, )]
pub fn make_hunted_be_automeleed(
    mut hunting_beings: Query<
        (Entity, &HostileChase, ),
        (LocalAiControlled, Changed<HostileChase>, ),
    >,
    mut auto_melee_targets_query: Query<&mut AutoMeleeIfInInteractionZone, >,

    mut ceased_to_hunt: RemovedComponents<HostileChase>,
) {
    for (_, hunting, ) in hunting_beings.iter_mut() {
        let Ok(mut ai_melee_targets) = auto_melee_targets_query.get_mut(hunting.prey) else {
            continue;
        };
        ai_melee_targets.0.clear();
        ai_melee_targets.0.push(hunting.prey);
    }
    for being_ent in ceased_to_hunt.read() {
        let Ok(mut ai_melee_targets) = auto_melee_targets_query.get_mut(being_ent) else {
            continue;
        };
        ai_melee_targets.0.clear();
    }
}

#[allow(unused_parens, )]
pub fn emit_ai_melee_attack_requests(
    time: Res<Time>,
    ai_beings_query: Query<
        (
            Entity,
            &AutoMeleeIfInInteractionZone,
            Option<&BitRef>,
            Option<&RaceRef>,
        ),
        (LocalAiControlled, ),
    >,
    direction_query: Query<&CardinalDirection, (),>,
    pos_query: Query<
        (&DimensionRef, &GlobalTransform, ),
        (Without<Dead>, ),
    >,
    bit_map: Res<BeingInstTemplateEntityMap>,
    race_map: Res<RaceEntityMap>,
    zone_sources: Query<&InteractionZones>,
    mut local_cooldowns: Local<EntityHashMap<Timer>>,
    mut local_requests: Local<Vec<LocalMeleeAttackRequest>>,
    mut writer: MessageWriter<LocalMeleeAttackRequest>,
) {
    let delta = time.delta();
    for (
        attacker_ent,
        attacker_targets,
        attacker_bit_ref,
        attacker_race_ref,
    ) in ai_beings_query.iter()
    {
        let Ok(attacker_direction) = direction_query.get(attacker_ent) else {
            continue;
        };
        let cooldown = local_cooldowns
            .entry(attacker_ent)
            .or_insert_with(|| Timer::from_seconds(0.0, TimerMode::Once));
        cooldown.tick(delta);
        if !cooldown.is_finished() {
            continue;
        }

        let attacker_interaction_zones = zone_sources.get(attacker_ent).ok();
        let melee_zone = resolve_being_interaction_zone(
            attacker_interaction_zones,
            attacker_bit_ref,
            attacker_race_ref,
            InteractionZones::MELEE_ATTACK,
            &bit_map,
            &race_map,
            &zone_sources,
        );
        let Ok((attacker_dim, attacker_transform, )) = pos_query.get(attacker_ent) else {
            continue;
        };

        let attacker_pos = attacker_transform.translation().xy();

        let mut should_attack = false;
        for &target_ent in &attacker_targets.0 {
            if target_ent == attacker_ent {
                continue;
            }
            let Ok((target_dim, target_transform, )) = pos_query.get(target_ent) else {
                continue;
            };
            if target_dim != attacker_dim {
                continue;
            }
            let Ok(target_direction) = direction_query.get(target_ent) else {
                continue;
            };
            let target_interaction_zones = zone_sources.get(target_ent).ok();
            let target_collision_zone = resolve_being_interaction_zone(
                target_interaction_zones,
                None,
                None,
                InteractionZones::COLLISION,
                &bit_map,
                &race_map,
                &zone_sources,
            );
            let target_pos_px = target_transform.translation().xy();
            if !target_collision_zone.intersects_zone_with_tolerance(
                *target_direction,
                target_pos_px,
                &melee_zone,
                *attacker_direction,
                attacker_pos,
                0.15,
                true,
            ) {
                continue;
            }
            should_attack = true;
            break;
        }

        if !should_attack {
            continue;
        }

        local_requests.push(LocalMeleeAttackRequest { being_ent: attacker_ent });
        cooldown.set_duration(TEMP_AI_MELEE_ATTACK_COOLDOWN);
        cooldown.reset();
    }

    local_cooldowns.retain(|attacker_ent, _| ai_beings_query.get(*attacker_ent).is_ok());
    writer.write_batch(local_requests.drain(..));
}



#[allow(unused_parens, )]
pub fn add_melee_target_comp_to_ai_controlled(
    mut commands: Commands,
    ai_controlled_beings: Query<
        Entity,
        (With<Being>, LocalAiControlled, Without<AutoMeleeIfInInteractionZone>, ),
    >,
    ceased_2b_ai_controlled: Query<Entity, Added<HumanControlled>>,
) {
    for being_ent in ai_controlled_beings.iter() {
        commands.entity(being_ent).try_insert_if_new(AutoMeleeIfInInteractionZone::default());
    }
    for being_ent in ceased_2b_ai_controlled.iter() {
        commands.entity(being_ent).try_remove::<AutoMeleeIfInInteractionZone>();
    }
}