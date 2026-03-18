use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::common_components::StrId;
use common::log_targets::BEING_SYSTEM;
use game_common::game_common_components::{EntityZeroRef, HealthDamage};
use tilemap::tile::tile_components::TileFlip;

use crate::{being_components::*, being_messages::*};

#[allow(unused_parens)]
pub fn validate_added_beings_have_position_and_transform(
    query: Query<(Entity, Option<&StrId>, Has<GlobalTilePos>, Has<Transform>), (Added<Being>)>,
) {
    for (ent, str_id, has_gpos, has_transform) in query.iter() {
        if has_gpos && has_transform {
            continue;
        }
        error_once!(
            target: BEING_SYSTEM,
            "Added Being {:?} {} missing required components: GlobalTilePos={} Transform={}",
            ent,
            str_id.map(StrId::as_str).unwrap_or("<no-strid>"),
            has_gpos,
            has_transform
        );
    }
}

pub fn apply_melee_attack(
    mut melee_attacks: MessageReader<LocalMeleeAttackRequest>,
    beings: Query<
        (
            &DimensionRef,
            &GlobalTransform,
            &CardinalDirection,
            Option<&InteractionZones>,
        ),
        With<Being>,
    >,
    being_receivers: Query<
        (
            &GlobalTransform,
            Option<&InteractionZones>,
            Option<&CardinalDirection>,
        ),
        With<Being>,
    >,
    beings_at_gpos: Res<BeingsAtGpos>,
    mut tile_gathering: TileGatheringParamSet,
    tile_instances: Query<(
        &GlobalTilePos,
        &EntityZeroRef,
        Option<&TileFlip>,
        Option<&CardinalDirection>,
    )>,
    tile_receivers: Query<(Option<&InteractionZones>, Option<&SizeInTiles>, Option<&TiledCollisionMask>)>,
    mut health_damage_writer: MessageWriter<HealthDamage>,
    mut candidate_tile_gposes: Local<Vec<GlobalTilePos>>,
    mut health_damage_messages: Local<Vec<HealthDamage>>,
) {
    const MELEE_DAMAGE: f32 = 10.0;
    for melee in melee_attacks.read() {
        let attacker_ent = melee.being_ent;
        let Ok((&attacker_dim, attacker_transform, &attacker_direction, interaction_zones, )) =
            beings.get(attacker_ent)
        else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} not found", attacker_ent);
            continue;
        };
        let Some(interaction_zones) = interaction_zones else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} has no InteractionZones", attacker_ent);
            continue;
        };
        let Ok(melee_zone) = interaction_zones.0.get(InteractionZones::MELEE_ATTACK) else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} has no melee zone", attacker_ent);
            continue;
        };

        let attacker_pos = attacker_transform.translation().xy();
        let mut hit_entities = EntityHashSet::default();
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
        melee_zone.gather_candidate_tiles_at(
            attacker_direction,
            attacker_pos,
            &mut candidate_tile_gposes,
        );
        for &candidate_gpos in candidate_tile_gposes.iter() {
            if !melee_zone.is_inside_any(
                SizeInTiles::default(),
                TileFlip::default(),
                attacker_direction,
                attacker_pos,
                candidate_gpos.to_pixelpos(),
            ) {
                continue;
            }
            for &target_entity in beings_at_gpos.beings_at_pos(attacker_dim, candidate_gpos) {
                if target_entity == attacker_ent || !hit_entities.insert(target_entity) {
                    continue;
                }
                let Ok((target_transform, target_zones, target_direction)) =
                    being_receivers.get(target_entity)
                else {
                    continue;
                };
                let receiver = COLLISION_MASK_HASHID;
                let target_pos_px = target_transform.translation().xy();
                let hit_point = candidate_gpos.to_pixelpos();

                let Some(target_zones) = target_zones else {
                    continue;
                };
                let hit = target_zones.is_inside_interaction_zone(
                    receiver,
                    SizeInTiles::default(),
                    target_pos_px,
                    hit_point,
                    TileFlip::default(),
                    target_direction.copied().unwrap_or_default(),
                );
                if !hit {
                    continue;
                }
                health_damage_messages.push(HealthDamage {
                    entity: target_entity,
                    amount: MELEE_DAMAGE,
                });
                hit_beings += 1;
                info!(target: BEING_SYSTEM, "Melee hit being {:?}", target_entity);
            }
        }

        for &candidate_gpos in candidate_tile_gposes.iter() {
            if !melee_zone.is_inside_any(
                SizeInTiles::default(),
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
                let Ok((target_zones, target_size_in_tiles, target_collision_mask)) = tile_receivers.get(tile_ezero) else {
                    continue;
                };
                let hit_point = candidate_gpos;
                let accepts_hit = if let Some(target_zones) = target_zones {
                    target_zones.is_inside_interaction_zone(
                        HITBOX_HASHID,
                        target_size_in_tiles.copied().unwrap_or_default(),
                        tile_origin.to_pixelpos(),
                        hit_point.to_pixelpos(),
                        tile_flip.copied().unwrap_or_default(),
                        tile_direction.copied().unwrap_or_default(),
                    )
                } else {
                    target_collision_mask.is_some_and(|mask| {
                        mask.is_solid_at_world_pos_with_flip(
                            tile_origin,
                            hit_point,
                            tile_flip.copied().unwrap_or_default(),
                            tile_direction.copied().unwrap_or_default(),
                        )
                    })
                };
                if !accepts_hit {
                    continue;
                }
                health_damage_messages.push(HealthDamage {
                    entity: target_entity,
                    amount: MELEE_DAMAGE,
                });
                hit_tiles += 1;
                info!(
                    target: BEING_SYSTEM,
                    "Melee hit tile instance {:?} (ezero {:?})",
                    target_entity,
                    tile_ezero
                );
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
