use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::log_targets::BEING_MELEE_SYSTEMS;
use game_common::game_common_components::TemplEntiRef;
use param_sets::BlockingTileParamSet;
use tilemap::tile::tile_components::TileFlip;
use std::time::Duration;
use crate::body::{DamageDistributeMode, IncHealthDamageOrHeal};

use crate::{being_interaction_zone_helper::resolve_being_interaction_zone, being_messages::*};

const TEMP_AI_MELEE_ATTACK_COOLDOWN: Duration = Duration::from_secs(1);

#[allow(unused_parens, )]
pub fn make_hunted_be_melee_targets(
    mut hunted_beings: Query<
        (Entity, &Hunting, &mut AiAutoMeleeTargets, ),
        (With<Being>, LocalAiControlled, With<Hunting>, ),
    >,
    mut ai_melee_targets_query: Query<
        &mut AiAutoMeleeTargets,
        (With<Being>, LocalAiControlled, Without<Hunting>, ),
    >,
    mut ceased_to_hunt: RemovedComponents<Hunting>,
) {
    for (_, hunting, mut ai_melee_targets, ) in hunted_beings.iter_mut() {
        ai_melee_targets.0.clear();
        ai_melee_targets.0.push(hunting.prey);
    }
    for being_ent in ceased_to_hunt.read() {
        let Ok(mut ai_melee_targets) = ai_melee_targets_query.get_mut(being_ent) else {
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
            &DimensionRef,
            &GlobalTransform,
            &AiAutoMeleeTargets,
            Option<&BitRef>,
            Option<&RaceRef>,
        ),
        (With<Being>, LocalAiControlled, ),
    >,
    direction_query: Query<&CardinalDirection, (),>,
    target_pos_query: Query<
        (&DimensionRef, &GlobalTilePos, ),
        (With<Being>, ),
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
        attacker_dim,
        attacker_transform,
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
        let attacker_pos = attacker_transform.translation().xy();

        let mut should_attack = false;
        for &target_ent in &attacker_targets.0 {
            if target_ent == attacker_ent {
                continue;
            }
            let Ok((target_dim, target_gpos, )) = target_pos_query.get(target_ent) else {
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
            let target_pos_px = target_gpos.to_pixelpos();
            if !target_collision_zone.intersects_zone(
                *target_direction,
                target_pos_px,
                &melee_zone,
                *attacker_direction,
                attacker_pos,
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
        debug!(
            target: BEING_MELEE_SYSTEMS,
            "AI melee request queued for {:?} at dim {:?}",
            attacker_ent,
            attacker_dim
        );
    }

    local_cooldowns.retain(|attacker_ent, _| ai_beings_query.get(*attacker_ent).is_ok());
    writer.write_batch(local_requests.drain(..));
}

#[allow(unused_parens, )]
pub fn apply_melee_attack(
    mut melee_attacks: MessageReader<LocalMeleeAttackRequest>,
    beings_query: Query<
        (
            &DimensionRef,
            &GlobalTransform,
        ),
        (With<Being>, ),
    >,
    bit_map: Res<BeingInstTemplateEntityMap>,
    race_map: Res<RaceEntityMap>,
    zone_sources: Query<&InteractionZones>,
    beings_at_gpos: Res<BeingsAtGpos>,
    mut tile_gathering: BlockingTileParamSet,
    tile_instances: Query<(&TemplEntiRef, Option<&TileFlip>,)>,
    mut hit_entities: Local<EntityHashSet>,
    mut candidate_tile_gposes: Local<Vec<GlobalTilePos>>,
    mut incoming_damage_messages: Local<Vec<IncHealthDamageOrHeal>>,
    mut incoming_damage_writer: MessageWriter<IncHealthDamageOrHeal>,
) {
    const MELEE_DAMAGE: f32 = 10.0;
    for melee in melee_attacks.read() {
        let attacker_ent = melee.being_ent;
        let Ok((&attacker_dim, attacker_transform, )) =
            beings_query.get(attacker_ent)
        else {
            info!(target: BEING_MELEE_SYSTEMS, "Melee ignored: attacker {:?} not found", attacker_ent);
            continue;
        };
        let Ok(attacker_direction) = tile_gathering.cardinal_direction_query().get_mut(attacker_ent) else {
            info!(target: BEING_MELEE_SYSTEMS, "Melee ignored: attacker {:?} has no facing direction", attacker_ent);
            continue;
        };
        let attacker_direction = *attacker_direction;
        let attacker_interaction_zones = zone_sources.get(attacker_ent).ok();
        let attacker_bit_ref = tile_gathering.get_being_bit_ref(attacker_ent);
        let attacker_race_ref = tile_gathering.get_being_race_ref(attacker_ent);

        let melee_zone = resolve_being_interaction_zone(
            attacker_interaction_zones,
            attacker_bit_ref,
            attacker_race_ref,
            InteractionZones::MELEE_ATTACK,
            &bit_map,
            &race_map,
            &zone_sources,
        );

        let attacker_pos = attacker_transform.translation().xy();
        hit_entities.clear();
        let mut hit_beings = 0usize;
        let mut hit_tiles = 0usize;

        info!(
            target: BEING_MELEE_SYSTEMS,
            "Melee started by {:?} at dim {:?}, facing {:?}",
            attacker_ent,
            attacker_dim,
            attacker_direction
        );

        candidate_tile_gposes.clear();
        melee_zone.gather_zone_positions(
            attacker_direction,
            attacker_pos,
            &mut candidate_tile_gposes,
        );
        let mut hit_done = false;
        for candidate_gpos in candidate_tile_gposes.drain(..) {
            if hit_done {
                break;
            }
            if !melee_zone.is_inside_any(
                TileFlip::default(),
                attacker_direction,
                attacker_pos,
                candidate_gpos.to_pixelpos(),
            ) {
                continue;
            }
            for &target_ent in beings_at_gpos.get_beings_at_pos(attacker_dim, candidate_gpos) {
                if target_ent == attacker_ent || !hit_entities.insert(target_ent) {
                    continue;
                }
                let Ok((_, target_transform, )) =
                    beings_query.get(target_ent)
                else {
                    continue;
                };
                let target_direction = tile_gathering.cardinal_direction_query().get(target_ent).cloned().unwrap_or_default();
                let target_interaction_zones = zone_sources.get(target_ent).ok();
                let target_bit_ref = tile_gathering.get_being_bit_ref(target_ent);
                let target_race_ref = tile_gathering.get_being_race_ref(target_ent);

                let target_pos_px = target_transform.translation().xy();
                let collision_zone = resolve_being_interaction_zone(
                    target_interaction_zones,
                    target_bit_ref,
                    target_race_ref,
                    InteractionZones::COLLISION,
                    &bit_map,
                    &race_map,
                    &zone_sources,
                );
                let hit = collision_zone.intersects_zone(
                    target_direction,
                    target_pos_px,
                    &melee_zone,
                    attacker_direction,
                    attacker_pos,
                );
                if !hit {
                    continue;
                }
                incoming_damage_messages.push(IncHealthDamageOrHeal {
                    target_ent,
                    source_ent: attacker_ent,
                    amount: MELEE_DAMAGE,
                    distribute_mode: DamageDistributeMode::SampledBodyPart,
                });
                hit_beings += 1;
                hit_done = true;
                info!(target: BEING_MELEE_SYSTEMS, "Melee hit being {:?}", target_ent);
                break;
            }
            if hit_done {
                break;
            }
            if !melee_zone.is_inside_any(
                TileFlip::default(),
                attacker_direction,
                attacker_pos,
                candidate_gpos.to_pixelpos(),
            ) {
                continue;
            }
            let tile_entities = tile_gathering.gather_tiles(attacker_dim, candidate_gpos).to_vec();
            for target_ent in tile_entities {
                if !hit_entities.insert(target_ent) {
                    error!(target: BEING_MELEE_SYSTEMS, "Melee hit entity {:?} already hit", target_ent);
                    continue;
                }
                let Ok((&TemplEntiRef(tile_templ), _tile_flip)) = tile_instances.get(target_ent)
                else {
                    error!(target: BEING_MELEE_SYSTEMS, "Melee hit entity {:?} has no tile_templ", target_ent);
                    continue;
                };
                let Ok(&tile_origin) = tile_gathering.gpos_query.get(target_ent) else {
                    error!(target: BEING_MELEE_SYSTEMS, "Melee hit entity {:?} has no tile position", target_ent);
                    continue;
                };
                let tile_direction = tile_gathering
                    .cardinal_direction_query()
                    .get_mut(target_ent)
                    .map(|direction| *direction)
                    .unwrap_or_default();
                let Ok(target_zones) = zone_sources.get(tile_templ) else {
                    continue;
                };
                let accepts_hit = target_zones.interaction_zones_intersect(
                    InteractionZones::COLLISION,
                    &melee_zone,
                    tile_direction,
                    tile_origin.to_pixelpos(),
                    attacker_direction,
                    attacker_pos,
                );
                if !accepts_hit {
                    continue;
                }
                incoming_damage_messages.push(IncHealthDamageOrHeal {
                    target_ent,
                    source_ent: attacker_ent,
                    amount: MELEE_DAMAGE,
                    distribute_mode: DamageDistributeMode::SampledBodyPart,
                });
                hit_tiles += 1;
                hit_done = true;
                info!(
                    target: BEING_MELEE_SYSTEMS,
                    "Melee hit tile instance {:?} (templ {:?})",
                    target_ent,
                    tile_templ
                );
                break;
            }
        }

        if hit_beings == 0 && hit_tiles == 0 {
            info!(target: BEING_MELEE_SYSTEMS, "Melee ended: no valid receiver hit");
        } else {
            info!(
                target: BEING_MELEE_SYSTEMS,
                "Melee ended: {} being hit(s), {} tile hit(s)",
                hit_beings,
                hit_tiles
            );
        }
    }
    incoming_damage_writer.write_batch(incoming_damage_messages.drain(..));
}
