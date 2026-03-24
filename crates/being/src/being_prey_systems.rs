use crate::being_components::Being;
use crate::being_inst_template::being_inst_template_resources::BitRef;
use crate::being_messages::PredatorSeenByPrey;
use crate::pack::pack_components::{
    Pack,
    PackAttackAlertEffectivenessFalloff,
    PackCounterRegroupTightness,
    PackOnAttackBehavior,
};
use crate::race::race_resources::RaceRef;
use ::being_shared::*;
use bevy::{
    ecs::{entity::EntityHashSet, entity_disabling::Disabled},
    prelude::*,
};
use common::AnyDisabling;
use common::log_targets::BEING_SYSTEM;
use tilemap_shared::{CardinalDirection, DimensionRef, GlobalTilePos};

fn collect_reenabled_entities(removed_disabled: &mut RemovedComponents<Disabled>) -> EntityHashSet {
    let mut entities = EntityHashSet::default();
    entities.extend(removed_disabled.read());
    entities
}

fn target_in_vision_cone(
    origin: GlobalTilePos,
    facing: CardinalDirection,
    target: GlobalTilePos,
    cone: DetectionVisionCone,
) -> bool {
    let delta = target.0 - origin.0;
    if delta == IVec2::ZERO {
        return true;
    }
    let delta_v = delta.as_vec2();
    let range = cone.range_tiles.max(0.0);
    if delta_v.length_squared() > range * range {
        return false;
    }
    let half_angle = cone.half_angle_deg.clamp(1.0, 179.0);
    let facing_v = facing.to_dir_vec().as_vec2();
    let dir_v = delta_v.normalize_or_zero();
    if dir_v == Vec2::ZERO || facing_v == Vec2::ZERO {
        return false;
    }
    let cos_limit = half_angle.to_radians().cos();
    facing_v.dot(dir_v) >= cos_limit
}

#[allow(unused_parens, )]
pub fn sync_detection_vision_cone_from_sources(
    mut commands: Commands,
    changed_beings: Query<Entity, (With<Being>, Or<(Changed<BitRef>, Changed<RaceRef>)>, )>,
    beings: Query<(Option<&BitRef>, Option<&RaceRef>, ), (With<Being>, AnyDisabling, )>,
    bit_cfg: Query<&DetectionVisionCone>,
    race_cfg: Query<&DetectionVisionCone>,
    mut removed_disabled: RemovedComponents<Disabled>,
) {
    let reenabled_beings = collect_reenabled_entities(&mut removed_disabled);
    let mut beings_to_sync = reenabled_beings;
    beings_to_sync.extend(changed_beings.iter());

    for being_ent in beings_to_sync {
        let Ok((bit_ref, race_ref, )) = beings.get(being_ent) else {
            continue;
        };
        let bit_cone = bit_ref.and_then(|r| bit_cfg.get(r.0).ok()).copied();
        let race_cone = race_ref.and_then(|r| race_cfg.get(r.0).ok()).copied();
        let Some(chosen) = bit_cone.or(race_cone) else {
            commands.entity(being_ent).try_remove::<DetectionVisionCone>();
            continue;
        };
        commands.entity(being_ent).try_insert(chosen);
    }
}

#[allow(unused_parens, )]
pub fn detect_predators_with_vision_cones(
    prey_query: Query<
        (
            Entity,
            &GlobalTilePos,
            &DimensionRef,
            &DetectionVisionCone,
            Option<&CardinalDirection>,
        ),
        (With<Being>, ),
    >,
    predators_query: Query<(Entity, &GlobalTilePos, &DimensionRef), (With<Being>, With<Predator>, )>,
    mut writer: MessageWriter<PredatorSeenByPrey>,
    mut messages: Local<Vec<PredatorSeenByPrey>>,
) {
    messages.clear();
    for (prey_ent, prey_gpos, &prey_dim, &cone, facing) in prey_query.iter() {
        let facing = facing.copied().unwrap_or_default();
        for (predator_ent, predator_gpos, &predator_dim) in predators_query.iter() {
            if predator_ent == prey_ent || predator_dim != prey_dim {
                continue;
            }
            if !target_in_vision_cone(*prey_gpos, facing, *predator_gpos, cone) {
                continue;
            }
            messages.push(PredatorSeenByPrey {
                prey: prey_ent,
                predator: predator_ent,
            });
        }
    }
    writer.write_batch(messages.drain(..));
}

#[allow(unused_parens, )]
pub fn update_prey_nav_states_from_predator_detection(
    mut cmd: Commands,
    mut reader: MessageReader<PredatorSeenByPrey>,
    prey_query: Query<
        (
            Option<&SquadMemberOf>,
            &GlobalTilePos,
            &DimensionRef,
        ),
        (With<Being>, ),
    >,
    pack_query: Query<
        (
            Option<&PackOnAttackBehavior>,
            Option<&PackAttackAlertEffectivenessFalloff>,
            Option<&PackCounterRegroupTightness>,
            Option<&SquadMembers>,
        ),
        (With<Pack>, ),
    >,
    member_pos_query: Query<(&GlobalTilePos, &DimensionRef, ), (With<Being>, )>,
) {
    for msg in reader.read() {
        let Ok((member_of, victim_gpos, &victim_dim, )) = prey_query.get(msg.prey) else {
            continue;
        };
        cmd.entity(msg.prey).try_insert(Fleeing::new(msg.predator));
        cmd.entity(msg.predator).try_insert(PredatorDetectedByPrey(msg.prey));

        let Some(member_of) = member_of else {
            trace!(target: BEING_SYSTEM, "Predator detected by {:?}; applying flee only to self against {:?}", msg.prey, msg.predator);
            continue;
        };
        let Ok((on_attack_behavior, alert_falloff, regroup_tightness, members, )) = pack_query.get(member_of.squad) else {
            continue;
        };
        let behavior = on_attack_behavior
            .map(|behavior| behavior.0.as_str())
            .unwrap_or("");
        let alert_falloff = alert_falloff.map(|v| v.0).unwrap_or(0.05).max(0.0);
        let regroup_tightness = regroup_tightness.map(|v| v.0).unwrap_or(1.5).max(0.0);
        let Some(members) = members else {
            continue;
        };

        if behavior == "ignore" {
            continue;
        }
        if behavior == "counter" {
            for member_ent in members.iter() {
                if member_ent == msg.prey {
                    continue;
                }
                let Ok((member_gpos, &member_dim, )) = member_pos_query.get(member_ent) else {
                    continue;
                };
                if member_dim != victim_dim {
                    continue;
                }
                if alert_falloff > 0.0 {
                    let dist = (member_gpos.0 - victim_gpos.0).as_vec2().length();
                    let effectiveness = 1.0 / (1.0 + dist * alert_falloff);
                    if effectiveness < 0.35 {
                        continue;
                    }
                }
                cmd.entity(member_ent).try_insert(Chasing::new(msg.prey, regroup_tightness));
            }
            continue;
        }

        for member_ent in members.iter() {
            let Ok((member_gpos, &member_dim, )) = member_pos_query.get(member_ent) else {
                continue;
            };
            if member_dim != victim_dim {
                continue;
            }
            if member_ent != msg.prey && alert_falloff > 0.0 {
                let predator_dist = (member_gpos.0 - victim_gpos.0).as_vec2().length();
                let effectiveness = 1.0 / (1.0 + predator_dist * alert_falloff);
                if effectiveness < 0.35 {
                    continue;
                }
            }
            cmd.entity(member_ent).try_insert(Fleeing::new(msg.predator));
        }
    }
}
