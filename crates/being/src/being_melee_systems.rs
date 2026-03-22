use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::log_targets::BEING_SYSTEM;
use game_common::game_common_components::{EntityZeroRef, HealthDamage};
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
    beings: Query<
        (
            &DimensionRef,
            &GlobalTransform,
            &CardinalDirection,
            Option<&InteractionZones>,
            Option<&BitRef>,
            Option<&RaceRef>,
        ),
        (With<Being>, ),
    >,
    being_receivers: Query<
        (
            &GlobalTransform,
            Option<&InteractionZones>,
            Option<&CardinalDirection>,
            Option<&BitRef>,
            Option<&RaceRef>,
        ),
        (With<Being>, ),
    >,
    zone_sources: Query<&InteractionZones>,
    beings_at_gpos: Res<BeingsAtGpos>,
    mut tile_gathering: TileGatheringParamSet,
    tile_instances: Query<(
        &GlobalTilePos,
        &EntityZeroRef,
        Option<&TileFlip>,
        Option<&CardinalDirection>,
    )>,
    tile_receivers: Query<&InteractionZones>,
    mut health_damage_writer: MessageWriter<HealthDamage>,
    mut candidate_tile_gposes: Local<Vec<GlobalTilePos>>,
    mut health_damage_messages: Local<Vec<HealthDamage>>,
    mut hit_entities: Local<EntityHashSet>,
) {
    const MELEE_DAMAGE: f32 = 10.0;
    for melee in melee_attacks.read() {
        let attacker_ent = melee.being_ent;
        let Ok((&attacker_dim, attacker_transform, &attacker_direction, interaction_zones, bit_ref, race_ref, )) =
            beings.get(attacker_ent)
        else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} not found", attacker_ent);
            continue;
        };
        let melee_zone = resolve_being_interaction_zone(
            interaction_zones,
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
            for &target_entity in beings_at_gpos.get_beings_at_pos(attacker_dim, candidate_gpos) {
                if target_entity == attacker_ent || !hit_entities.insert(target_entity) {
                    continue;
                }
                let Ok((target_transform, target_zones, target_direction, target_bit_ref, target_race_ref, )) =
                    being_receivers.get(target_entity)
                else {
                    continue;
                };
                let target_pos_px = target_transform.translation().xy();
                let hit_point = candidate_gpos.to_pixelpos();
                let collision_zone = resolve_being_interaction_zone(
                    target_zones,
                    target_bit_ref,
                    target_race_ref,
                    COLLISION_MASK_HASHID,
                    &zone_sources,
                );
                let hit = collision_zone.is_inside_any(
                    TileFlip::default(),
                    target_direction.copied().unwrap_or_default(),
                    target_pos_px,
                    hit_point,
                );
                if !hit {
                    continue;
                }
                health_damage_messages.push(HealthDamage {
                    entity: target_entity,
                    amount: MELEE_DAMAGE,
                });
                hit_beings += 1;
                hit_done = true;
                info!(target: BEING_SYSTEM, "Melee hit being {:?}", target_entity);
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
            for &target_entity in tile_gathering.gather_tiles_at_to_drain(attacker_dim, candidate_gpos) {
                if !hit_entities.insert(target_entity) {
                    continue;
                }
                let Ok((&tile_origin, &EntityZeroRef(tile_ezero), tile_flip, tile_direction)) =
                    tile_instances.get(target_entity)
                else {
                    continue;
                };
                let Ok(target_zones) = tile_receivers.get(tile_ezero) else {
                    continue;
                };
                let hit_point = candidate_gpos;
                let accepts_hit = target_zones.is_point_inside_zone(
                    COLLISION_MASK_HASHID,
                    tile_origin.to_pixelpos(),
                    tile_direction.copied().unwrap_or_default(),
                    tile_flip.copied().unwrap_or_default(),
                    hit_point.to_pixelpos(),
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
