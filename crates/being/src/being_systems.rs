use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use common::log_targets::{BEING_SYSTEM, MOVEMENT_SYSTEM};
use game_common::game_common_components::{EntityZeroRef, HealthDamage};
use movement::movement_components::InputMoveDir;
use tilemap::tile::tile_components::TileFlip;

use crate::{being_components::*, being_messages::*};

#[allow(unused_parens)]
#[allow(unused_parens)]
pub fn beings_sync_transform_to_added_gpos(
    mut query: Query<(&GlobalTilePos, &mut Transform), (With<Being>, Added<GlobalTilePos>)>,
) {
    for (&gpos, mut transform) in query.iter_mut() {
        transform.translation = gpos.to_translation(transform.translation.z);
    }
}

#[allow(unused_parens)]
pub fn sync_occupancy_for_beings_at_gpos_res(
    mut beings_at_gpos: ResMut<BeingsAtGpos>,
    mut removed_beings: RemovedComponents<Being>,
    mut tracked_pos: Local<EntityHashMap<(DimensionRef, GlobalTilePos)>>,
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos),
        (
            With<Being>,
            Or<(Added<Being>, Changed<GlobalTilePos>, Changed<DimensionRef>)>,
        ),
    >,
) {
    for ent in removed_beings.read() {
        let Some((old_dim, old_gpos)) = tracked_pos.remove(&ent) else {
            continue;
        };
        beings_at_gpos.remove_being(old_dim, old_gpos, ent);
    }

    for (being_ent, &dim_ref, &gpos) in query.iter() {
        let prev = tracked_pos.get(&being_ent).copied();

        let Some((old_dim, old_gpos)) = prev else {
            tracked_pos.insert(being_ent, (dim_ref, gpos));
            beings_at_gpos.insert_being(dim_ref, gpos, being_ent);
            continue;
        };

        if old_dim == dim_ref && old_gpos == gpos {
            continue;
        }
        beings_at_gpos.remove_being(old_dim, old_gpos, being_ent);
        beings_at_gpos.insert_being(dim_ref, gpos, being_ent);
        tracked_pos.insert(being_ent, (dim_ref, gpos));
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
            Option<&HitboxReceiver>,
        ),
        With<Being>,
    >,
    being_receivers: Query<
        (
            &GlobalTransform,
            Option<&InteractionZones>,
            Option<&HitboxReceiver>,
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
    tile_receivers: Query<(Option<&InteractionZones>, Option<&TiledCollisionMask>)>,
    mut health_damage_writer: MessageWriter<HealthDamage>,
    mut candidate_tile_gposes: Local<Vec<GlobalTilePos>>,
    mut health_damage_messages: Local<Vec<HealthDamage>>,
) {
    const MELEE_DAMAGE: f32 = 10.0;
    for melee in melee_attacks.read() {
        let attacker_ent = melee.being_ent;
        let Ok((&attacker_dim, attacker_transform, &attacker_direction, interaction_zones, _)) =
            beings.get(attacker_ent)
        else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} not found", attacker_ent);
            continue;
        };
        let Some(interaction_zones) = interaction_zones else {
            info!(target: BEING_SYSTEM, "Melee ignored: attacker {:?} has no InteractionZones", attacker_ent);
            continue;
        };
        let Ok(melee_zone) = interaction_zones.0.get(InteractionZones::MELEE) else {
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

        for (&(dim_ref, target_pos), target_entities) in beings_at_gpos.0.iter() {
            if dim_ref != attacker_dim {
                continue;
            }
            if !melee_zone.is_inside_any(attacker_direction, attacker_pos, target_pos.to_pixelpos()) {
                continue;
            }
            for &target_entity in target_entities.iter() {
                if target_entity == attacker_ent || !hit_entities.insert(target_entity) {
                    continue;
                }
                let Ok((target_transform, target_zones, target_hitbox_receiver, target_direction)) =
                    being_receivers.get(target_entity)
                else {
                    continue;
                };
                let receiver = target_hitbox_receiver.copied().unwrap_or_default().0;
                let target_pos_px = target_transform.translation().xy();
                let hit_point = target_pos.to_pixelpos();
                let accepts_hit = if receiver == COLLISION_MASK_HASHID {
                    true
                } else {
                    let Some(target_zones) = target_zones else {
                        continue;
                    };
                    target_zones.is_inside_interaction_zone(
                        receiver,
                        target_pos_px,
                        hit_point,
                        target_direction.copied().unwrap_or_default(),
                    )
                };
                if !accepts_hit {
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

        candidate_tile_gposes.clear();
        melee_zone.gather_candidate_tiles_at(
            attacker_direction,
            attacker_pos,
            &mut candidate_tile_gposes,
        );
        for &candidate_gpos in candidate_tile_gposes.iter() {
            if !melee_zone.is_inside_any(
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
                let Ok((target_zones, target_collision_mask)) = tile_receivers.get(tile_ezero) else {
                    continue;
                };
                let hit_point = candidate_gpos;
                let accepts_hit = if let Some(target_zones) = target_zones {
                    target_zones.is_inside_interaction_zone(
                        HITBOX_HASHID,
                        tile_origin.to_pixelpos(),
                        hit_point.to_pixelpos(),
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
