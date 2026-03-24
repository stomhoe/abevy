use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::log_targets::BEING_SYSTEM;
use game_common::game_common_components::{EntityZeroRef, HealthDamage};
use param_sets::BlockingTileParamSet;
use tilemap::tile::tile_components::TileFlip;

use crate::{
    being_components::*,
    being_inst_template::being_inst_template_resources::BitRef,
    being_interaction_zone_helper::resolve_being_interaction_zone,
    being_messages::*,
    race::race_resources::RaceRef,
};

#[allow(unused_parens, )]
pub fn apply_melee_attack(
    mut melee_attacks: MessageReader<LocalMeleeAttackRequest>,
    beings_query: Query<
        (
            &DimensionRef,
            &GlobalTransform,
            Option<&BitRef>,
            Option<&RaceRef>,
        ),
        (With<Being>, ),
    >,
    zone_sources: Query<&InteractionZones>,
    beings_at_gpos: Res<BeingsAtGpos>,
    mut tile_gathering: BlockingTileParamSet,
    tile_instances: Query<(
        &GlobalTilePos,
        &EntityZeroRef,
        Option<&TileFlip>,
    )>,
    mut health_damage_writer: MessageWriter<HealthDamage>,
    mut candidate_tile_gposes: Local<Vec<GlobalTilePos>>,
    mut health_damage_messages: Local<Vec<HealthDamage>>,
    mut hit_entities: Local<EntityHashSet>,
) {
    const MELEE_DAMAGE: f32 = 10.0;
    for melee in melee_attacks.read() {
        let attacker_ent = melee.being_ent;
        let Ok((&attacker_dim, attacker_transform, bit_ref, race_ref, )) =
            beings_query.get(attacker_ent)
        else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} not found", attacker_ent);
            continue;
        };
        let Ok(attacker_direction) = tile_gathering.cardinal_direction_query().get_mut(attacker_ent) else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} has no facing direction", attacker_ent);
            continue;
        };
        let attacker_direction = *attacker_direction;
        let attacker_interaction_zones = zone_sources.get(attacker_ent).ok();

        let melee_zone = resolve_being_interaction_zone(
            attacker_interaction_zones,
            bit_ref,
            race_ref,
            InteractionZones::MELEE_ATTACK,
            &zone_sources,
        );

        let attacker_pos = attacker_transform.translation().xy();
        hit_entities.clear();
        let mut hit_beings = 0usize;
        let mut hit_tiles = 0usize;

        info!(
            target: BEING_SYSTEM,
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
        for &candidate_gpos in candidate_tile_gposes.iter() {
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
                let Ok((_, target_transform, target_bit_ref, target_race_ref, )) =
                    beings_query.get(target_ent)
                else {
                    continue;
                };
                let target_direction = tile_gathering.cardinal_direction_query().get(target_ent).cloned().unwrap_or_default();
                let target_interaction_zones = zone_sources.get(target_ent).ok();

                let target_pos_px = target_transform.translation().xy();
                let collision_zone = resolve_being_interaction_zone(
                    target_interaction_zones,
                    target_bit_ref,
                    target_race_ref,
                    InteractionZones::COLLISION,
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
                health_damage_messages.push(HealthDamage {
                    entity: target_ent,
                    amount: MELEE_DAMAGE,
                });
                hit_beings += 1;
                hit_done = true;
                info!(target: BEING_SYSTEM, "Melee hit being {:?}", target_ent);
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
            let tile_entities = tile_gathering.gather_tiles_at(attacker_dim, candidate_gpos).to_vec();
            for target_entity in tile_entities {
                if !hit_entities.insert(target_entity) {
                    error!(target: BEING_SYSTEM, "Melee hit entity {:?} already hit", target_entity);
                    continue;
                }
                let Ok((&tile_origin, &EntityZeroRef(tile_ezero), _tile_flip)) = tile_instances.get(target_entity)
                else {
                    error!(target: BEING_SYSTEM, "Melee hit entity {:?} has no tile instance", target_entity);
                    continue;
                };
                let tile_direction = tile_gathering
                    .cardinal_direction_query()
                    .get_mut(target_entity)
                    .map(|direction| *direction)
                    .unwrap_or_default();
                let Ok(target_zones) = zone_sources.get(tile_ezero) else {
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
                health_damage_messages.push(HealthDamage {
                    entity: target_entity,
                    amount: MELEE_DAMAGE,
                });
                hit_tiles += 1;
                hit_done = true;
                info!(
                    target: BEING_SYSTEM,
                    "Melee hit tile instance {:?} (ezero {:?})",
                    target_entity,
                    tile_ezero
                );
                break;
            }
        }

        if hit_beings == 0 && hit_tiles == 0 {
            info!(target: BEING_SYSTEM, "Melee ended: no valid receiver hit");
        } else {
            info!(
                target: BEING_SYSTEM,
                "Melee ended: {} being hit(s), {} tile hit(s)",
                hit_beings,
                hit_tiles
            );
        }
    }
    health_damage_writer.write_batch(health_damage_messages.drain(..));
}
