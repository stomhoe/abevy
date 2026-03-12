use ::being_shared::*;
use ::tilemap_shared::*;
use ac_input::ac_input_actions::{BeingDirectControlInputContext, BeingMeleeAttackAction};
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;
use camera::camera_components::CameraTarget;
use common::log_targets::{BEING_CONTROL, BEING_SYSTEM, MOVEMENT_SYSTEM};
use faction::faction_components::*;
use game_common::game_common_components::{EntityZeroRef, HealthDamage};
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use modifier_shared::{modifier_components::*, modifier_move_bundles::TempSpeedModifier};
use movement::movement_components::InputMoveDir;
use player::player_components::*;
use tilemap::{
    chunking::chunking_components::ActivatingChunks,
    chunking::chunking_resources::AaChunkRangeSettings, tile::tile_components::*,
};

use crate::{being_components::*, being_messages::*};

#[allow(unused_parens)]
// A L CENTRO DE LA BASE VA A HABER Q PONERLE UNO DE ALGUNA FORMA
pub fn add_activates_chunks(
    mut cmd: Commands,
    query: Query<(Entity), (With<Being>, Added<BelongsToAPlayerFaction>)>,
    mut removed: RemovedComponents<BelongsToAPlayerFaction>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    let mut activates_chunks = Vec::new();
    query.iter().for_each(|ent| {
        activates_chunks.push((ent, ActivatingChunks::new(&chunk_range)));
    });
    for ent in removed.read() {
        cmd.entity(ent).try_remove::<ActivatingChunks>();
    }
    cmd.try_insert_batch(activates_chunks);
}

#[allow(unused_parens)]
pub fn on_control_change(
    mut commands: Commands,
    self_player: Query<(Entity, Has<HostPlayer>), (With<Player>, With<Mine>)>,
    self_player_became_mine: Query<(), (With<Player>, With<Mine>, Added<Mine>)>,
    changed_query: Query<(Entity, &ComputedBy, Has<CameraTarget>), Changed<ComputedBy>>,
    computed_by_query: Query<(Entity, &ComputedBy, Has<CameraTarget>)>,
    mut input_dirs: Query<&mut InputMoveDir>,
    mut removed_controlled_by: RemovedComponents<ComputedBy>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for being_ent in removed_controlled_by.read() {
        commands.entity(being_ent).try_remove::<(ComputedLocally, HumanControlled, CameraTarget)>();
    }
    let Ok((self_entity, is_host)) = self_player.single() else {
        debug_once!(target: BEING_CONTROL, "Skipping control refresh until local player is marked Mine");
        return;
    };
    let mut apply_control_change = |being_ent: Entity,
                                    controlled_by: &ComputedBy,
                                    is_camera_target: bool| {
        if let Ok(mut input_dir) = input_dirs.get_mut(being_ent) {
            trace!(
                target: BEING_CONTROL,
                "InputMoveDir reset on control change for {:?}: {:?} -> {:?}",
                being_ent,
                input_dir.0,
                Vec2::ZERO
            );
            input_dir.0 = Vec2::ZERO;
        }
        if controlled_by.client_ent == self_entity {
            info!(target: BEING_CONTROL, "debug {:?} is now computed locally by self", being_ent);
            commands
                .entity(being_ent)
                .try_insert_if_new((ComputedLocally, ActivatingChunks::new(&chunk_range)));
            if controlled_by.human_input {
                //PROVISORIO
                debug!(target: BEING_CONTROL, "Entity {:?} is now a CameraTarget due to human input", being_ent);
                commands
                    .entity(being_ent)
                    .try_insert((HumanControlled, CameraTarget::default()));
            } else {
                debug!(target: BEING_CONTROL, "Entity {:?} is no longer a CameraTarget", being_ent);
                commands.entity(being_ent).try_remove::<(CameraTarget, HumanControlled)>();
            } //ENDOF PROVISORIO
        } else {
            commands.entity(being_ent).try_remove::<(ComputedLocally, CameraTarget)>();
            if !is_host {
                commands.entity(being_ent).try_remove::<HumanControlled>();
                if !is_camera_target {
                    commands.entity(being_ent).try_remove::<ActivatingChunks>();
                }
            } else {
                commands.entity(being_ent).try_insert(HumanControlled);
            }
        }
    };
    if !self_player_became_mine.is_empty() {
        for (being_ent, controlled_by, is_camera_target) in computed_by_query.iter() {
            apply_control_change(being_ent, controlled_by, is_camera_target);
        }
        return;
    }
    for (being_ent, controlled_by, is_camera_target) in changed_query.iter() {
        apply_control_change(being_ent, controlled_by, is_camera_target);
    }
}

pub fn assign_uncontrolled_beings_to_host(
    mut commands: Commands,
    self_player: Query<Entity, (With<Mine>, With<Player>, With<HostPlayer>)>,
    beings: Query<Entity, (With<Being>, Without<ComputedBy>)>,
) {
    let Ok(self_entity) = self_player.single() else {
        return;
    };
    for being_ent in beings.iter() {
        commands.entity(being_ent).try_insert(ComputedBy {
            client_ent: self_entity,
            human_input: false,
        });
    }
}

#[allow(unused_parens)]
pub fn cross_portal(
    mut cmd: Commands,
    portal_query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            Option<&PortalTo>,
            Option<&EntityZeroRef>,
            Option<&CardinalDirection>,
        ),
        (Without<Being>),
    >,
    interaction_zones_query: Query<(&InteractionZones), ()>,
    portal_arrival_sampler_query: Query<&GlobalTilePosWeightedSampler>,
    mut being_query: Query<
        (
            Entity,
            &mut DimensionRef,
            &GlobalTilePos,
            &GlobalTransform,
            Option<&TouchingPortal>,
        ),
        (With<Being>,),
    >,
) {
    let mut rng = rand::rng();
    for (
        being_entity,
        mut being_dimension_ref,
        &being_gpos,
        being_globtransform,
        touching_portal,
    ) in being_query.iter_mut()
    {
        portal_query.iter().for_each(
            |(portal0_ent, &portal0_dim, &portal0_gpos, portal_to, portal0_ezero_ref, portal0_facedir)| {
                if being_dimension_ref.clone() != portal0_dim {
                    return;
                }
                let Some(portal_to) = portal_to
                else { return; };


                let is_interacting = if let Some(ezero_ref) = portal0_ezero_ref {
                    if let Ok(interaction_zones) = interaction_zones_query.get(ezero_ref.0) {
                        interaction_zones.is_inside_interaction_zone(
                            InteractionZones::ENTER,
                            portal0_gpos.to_pixelpos(),
                            being_globtransform.translation().xy(),
                            portal0_facedir.copied().unwrap_or_default(),
                        )
                    } else {
                        being_gpos == portal0_gpos
                    }
                } else {
                    being_gpos == portal0_gpos
                };

                match (touching_portal, is_interacting) {
                    (None, false) => {}
                    (Some(&TouchingPortal(touching_portal)), false) => {
                        if portal0_ent == touching_portal {
                            cmd.entity(being_entity).try_remove::<TouchingPortal>();
                        }
                    }
                    (Some(&TouchingPortal(touching_portal)), true) => {
                        if portal0_ent != touching_portal {
                            cmd.entity(being_entity)
                                .try_insert(TouchingPortal(portal0_ent));
                        }
                    }
                    (None, true) => {
                        cmd.spawn((TempSpeedModifier::new(
                            being_entity,
                            being_entity,
                            0.0,
                            ApplyMode::Max,
                            1.0,
                        ),));

                        cmd.entity(being_entity)
                            .try_insert((TouchingPortal(portal0_ent),));

                        let Ok((_, &dest_dim, &dest_tile_gpos, _, dest_tile_ezero_ref, _)) =
                            portal_query.get(portal_to.dest_tile)
                        else {
                            error!(
                                "Portal entity {:?} not found in portal query",
                                portal_to.dest_tile
                            );
                            return;
                        };
                        let arrival_sampler = dest_tile_ezero_ref
                            .and_then(|ezero| portal_arrival_sampler_query.get(ezero.0).ok())
                            .and_then(|arrivals| arrivals.sample_with_rng(&mut rng));
                        let sampled_offset = arrival_sampler
                            .or_else(|| portal_to.offset_pos_destinations.sample_with_rng(&mut rng))
                            .unwrap_or_default();

                        being_dimension_ref.0 = dest_dim.0;

                        let arrival_gpos = dest_tile_gpos + sampled_offset;

                        cmd.entity(being_entity) //for replicate_once propagation
                            .try_insert((arrival_gpos));
                    }
                }
            },
        );
    }
}

#[allow(unused_parens)]
pub fn sync_transform_on_added_gpos(
    mut query: Query<(&GlobalTilePos, &mut Transform), (With<Being>, Added<GlobalTilePos>)>,
) {
    for (&gpos, mut transform) in query.iter_mut() {
        transform.translation = gpos.to_translation(transform.translation.z);
    }
}

#[allow(unused_parens)]
pub fn sync_beings_at_gpos(
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
    mut melee_attacks: MessageReader<LocalMeleeAttackRequested>,
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
    tile_gathering: TileGatheringParamSet,
    tile_instances: Query<(
        &GlobalTilePos,
        &EntityZeroRef,
        Option<&TileFlip>,
        Option<&CardinalDirection>,
    )>,
    tile_receivers: Query<(Option<&InteractionZones>, Option<&TiledCollisionMask>)>,
    mut health_damage_writer: MessageWriter<HealthDamage>,
    mut tiles_to_drain: Local<Vec<Entity>>,
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
            tiles_to_drain.clear();
            tile_gathering.gather_tiles_at(&mut *tiles_to_drain, attacker_dim, candidate_gpos);
            for &target_entity in tiles_to_drain.iter() {
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

pub fn trigger_melee_requests_from_player_input(
    player_melee_actions: Query<
        (&Action<BeingMeleeAttackAction>, &ComputedBeings),
        (With<Mine>, With<Player>, Changed<Action<BeingMeleeAttackAction>>),
    >,
    controlled_beings: Query<&ComputedBy>,
    mut writer: MessageWriter<LocalMeleeAttackRequested>,
    mut messages: Local<Vec<LocalMeleeAttackRequested>>,
) {
    for (melee_action, computed_beings) in player_melee_actions.iter() {
        if !**melee_action {
            continue;
        }
        for &being_ent in computed_beings.being_ents() {
            let Ok(controlled_by) = controlled_beings.get(being_ent) else {
                continue;
            };
            if !controlled_by.human_input {
                continue;
            }
            messages.push(LocalMeleeAttackRequested { being_ent });
        }
    }
    writer.write_batch(messages.drain(..));
}

pub fn send_melee_attack_to_server(
    mut event_writer: MessageWriter<ClientMeleeAttack>,
    mut melee_attacks: MessageReader<LocalMeleeAttackRequested>,
    beings: Query<(), With<ComputedLocally>>,
    mut messages: Local<Vec<ClientMeleeAttack>>,
) {
    for melee in melee_attacks.read() {
        let being_ent = melee.being_ent;
        let Ok(()) = beings.get(being_ent) else {
            continue;
        };

        messages.push(ClientMeleeAttack { being_ent });
    }
    event_writer.write_batch(messages.drain(..));
}

pub fn receive_melee_attack_from_client(
    mut events: MessageReader<FromClient<ClientMeleeAttack>>,
    controlled_beings_query: Query<&ComputedBy>,
    mut writer: MessageWriter<LocalMeleeAttackRequested>,
    mut messages: Local<Vec<LocalMeleeAttackRequested>>,
) {
    for from_client in events.read() {
        let ClientMeleeAttack { being_ent } = from_client.message.clone();
        let Ok(controlled_by) = controlled_beings_query.get(being_ent) else {
            warn!(target: BEING_SYSTEM, "Client tried to melee with missing/uncontrolled being {}", being_ent);
            continue;
        };
        let Some(client_entity) = from_client.client_id.entity() else {
            continue;
        };
        if controlled_by.client_ent != client_entity {
            warn!(
                target: BEING_SYSTEM,
                "Client tried to melee with a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                being_ent,
                controlled_by.client_ent,
                client_entity
            );
            continue;
        }
        messages.push(LocalMeleeAttackRequested { being_ent });
    }
    writer.write_batch(messages.drain(..));
}
