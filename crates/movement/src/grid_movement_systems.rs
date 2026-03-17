use being_shared::{Being, ComputedLocally};
use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::StrId;
use param_sets::BlockingTileParamSet;
use common::log_targets::{BEING_SYSTEM, MOVEMENT_SYSTEM};
use common::file_logging::file_log;
use ::sprite_animation_shared::*;

use tilemap::tile::prelude::Tile;
use tilemap_shared::{BeingsAtGpos, *};

use crate::movement_components::*;
use crate::grid_movement_helpers::*;
use crate::movement_helpers::*;
use crate::movement_messages::*;

const TILE_CORRECTION_INTERVAL_SECS: f32 = 2.0;

#[allow(unused_parens)]
pub fn beings_snap_transform_to_added_gpos(
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

pub fn resolve_overlapping_beings(
    mut beings: ParamSet<(
        Query<(Entity, &DimensionRef, &GlobalTilePos), With<Being>>,
        Query<(
            Option<&StrId>,
            &DimensionRef,
            &mut GlobalTilePos,
            &mut Transform,
            Option<&mut GridLockedMovement>,
        ), With<Being>>,
    )>,
    loaded_chunks: Res<LoadedChunks>,
    mut blocking_tiles: BlockingTileParamSet,
    mut occupied_positions: Local<HashMap<(DimensionRef, GlobalTilePos), Vec<Entity>>>,
    mut duplicate_positions: Local<Vec<((DimensionRef, GlobalTilePos), Vec<Entity>)>>,
    mut reserved_positions: Local<HashSet<(DimensionRef, GlobalTilePos)>>,
) {
    occupied_positions.clear();
    duplicate_positions.clear();
    reserved_positions.clear();

    for (being_ent, &dim_ref, &gpos) in beings.p0().iter() {
        occupied_positions
            .entry((dim_ref, gpos))
            .or_default()
            .push(being_ent);
    }

    for (&position_key, overlapping_beings) in occupied_positions.iter_mut() {
        overlapping_beings.sort_unstable_by_key(|ent| ent.to_bits());
        reserved_positions.insert(position_key);
        if overlapping_beings.len() > 1 {
            duplicate_positions.push((position_key, overlapping_beings.clone()));
        }
    }

    if duplicate_positions.is_empty() {
        return;
    }

    duplicate_positions.sort_unstable_by_key(|((dim_ref, gpos), _)| {
        (dim_ref.0.to_bits(), gpos.0.x, gpos.0.y)
    });

    let mut corrected_beings = 0usize;
    for &((dim_ref, source_gpos), ref overlapping_beings) in duplicate_positions.iter() {
        let keeper = overlapping_beings[0];
        for &being_ent in overlapping_beings.iter().skip(1) {
            let Some(found_gpos) = find_nearest_overlap_resolution_gpos(
                &mut blocking_tiles,
                &loaded_chunks,
                &reserved_positions,
                dim_ref,
                source_gpos,
                being_ent,
            ) else {
                warn!(
                    target: BEING_SYSTEM,
                    "Could not resolve overlapping beings at {:?} {:?}; keeping {:?} in place",
                    dim_ref,
                    source_gpos,
                    being_ent
                );
                continue;
            };

            let mut beings_mut = beings.p1();
            let Ok((str_id, &current_dim, mut gpos, mut transform, movement)) = beings_mut.get_mut(being_ent) else {
                continue;
            };
            if current_dim != dim_ref || *gpos == found_gpos {
                continue;
            }

            *gpos = found_gpos;
            transform.translation = found_gpos.to_translation(transform.translation.z);
            if let Some(mut movement) = movement {
                movement.clear_step(found_gpos);
            }
            reserved_positions.insert((dim_ref, found_gpos));
            corrected_beings += 1;

            debug!(
                target: BEING_SYSTEM,
                "Resolved overlap at {:?} {:?}: kept {:?}, moved {:?} ({}) to {:?}",
                dim_ref,
                source_gpos,
                keeper,
                being_ent,
                str_id.map(StrId::as_str).unwrap_or("<no-strid>"),
                found_gpos
            );
        }
    }

    if corrected_beings > 0 {
        info!(
            target: BEING_SYSTEM,
            "Resolved {} overlapping being positions",
            corrected_beings
        );
    }
}

pub fn start_grid_locked_steps(
    fixed_time: Res<Time<Fixed>>,
    client_state: Res<State<ClientState>>,
    connected: Query<&player::player_components::Player, Without<player::player_components::Mine>>,
    mut blocking_tiles: BlockingTileParamSet,
    mut beings: Query<(
        Entity,
        &FinalNormMoveDir,
        &DimensionRef,
        &SpeedMagnitude,
        &mut GlobalTilePos,
        &mut GridLockedMovement,
        &CardinalDirection,
    ), (With<ComputedLocally>, Without<Tile>)>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut sync_gpos_msgs: Local<Vec<ToClients<SyncGpos>>>,
    mut req_step_msgs: Local<Vec<SendStepRequest>>,
    mut burst_step_credit_by_ent: Local<EntityHashMap<f32>>,
    mut client_req_step_writer: MessageWriter<SendStepRequest>,
) {
    for (
        entity,
        final_norm_move_dir,
        &dim_ref,
        speed_magnitude,
        mut tile_pos,
        mut glm,
        facing_dir,
    ) in beings.iter_mut()
    {
        glm.ensure_grid_anchor(*tile_pos);
        let dir = final_norm_move_dir.normalize_to_axis_dir();
        let next_dir = CardinalDirection::from_dir_vec(dir);
        let step_ticks = ticks_per_tile(speed_magnitude.0, fixed_time.delta_secs(), dir);

        let mut steps_to_request = 0;
        let should_sync_with_others = if step_ticks > 1 {
            burst_step_credit_by_ent.remove(&entity);
            match glm.try_start_step(
                &mut blocking_tiles,
                dim_ref,
                entity,
                &mut tile_pos,
                dir,
                step_ticks,
            ) {
                TryStartStepOutcome::Successful => {
                    steps_to_request = 1;
                    true
                }
                TryStartStepOutcome::Blocked => {
                    if *facing_dir == next_dir {
                        false
                    } else {
                        steps_to_request = 1;
                        true
                    }
                }
                TryStartStepOutcome::IVec2ZeroDir
                | TryStartStepOutcome::AlreadyStepping
                | TryStartStepOutcome::ZeroStepTicks => false,
            }
        } else {
            let credit = burst_step_credit_by_ent.entry(entity).or_insert(0.0);
            *credit = (*credit + tiles_per_tick(speed_magnitude.0, fixed_time.delta_secs(), dir))
                .min(MAX_GRID_STEPS_PER_FIXED_TICK as f32);
            let requested_steps = credit.floor().min(MAX_GRID_STEPS_PER_FIXED_TICK as f32) as u16;
            let steps_taken = glm.advance_steps_immediate(
                &mut blocking_tiles,
                dim_ref,
                entity,
                &mut tile_pos,
                dir,
                requested_steps,
            );
            if steps_taken > 0 {
                *credit = (*credit - steps_taken as f32).max(0.0);
                steps_to_request = steps_taken;
                true
            } else if dir != IVec2::ZERO && *facing_dir != next_dir {
                steps_to_request = 1;
                true
            } else {
                false
            }
        };
        if !should_sync_with_others {
            continue;
        }
        if client_state.get() == &ClientState::Connected {
            req_step_msgs.push(SendStepRequest {
                being_ent: entity,
                dir: next_dir,
                steps: steps_to_request.max(1),
            });
        } else if !connected.is_empty() {
            let message = SyncGpos {
                being_ent: entity,
                gpos: *tile_pos,
                dir: next_dir,
                force_resync: false,
            };
            sync_gpos_msgs.push(ToClients {
                mode: SendMode::Broadcast,
                message: message.clone(),
            });
        }
    }
    writer.write_batch(sync_gpos_msgs.drain(..));
    client_req_step_writer.write_batch(req_step_msgs.drain(..));
}

pub fn progress_tile_transition_transform(
    mut query: Query<(
        Entity,
        &GlobalTilePos,
        &mut Transform,
        &mut MoveAnimActive,
        &mut GridLockedMovement,
    )>,
    mut writer: MessageWriter<MatchHeldSpritesAnimStateToBeingState>,
    mut messages: Local<HashSet<MatchHeldSpritesAnimStateToBeingState>>,
) {
    for (being_ent, tile_pos, mut transform, mut move_anim, mut glm) in query.iter_mut() {
        let had_motion_this_tick = glm.consume_recent_motion();
        glm.ensure_grid_anchor(*tile_pos);
        let was_stepping = glm.is_stepping();
        glm.progress_grid_step(*tile_pos);
        let new_translation = glm.grid_translation(*tile_pos, transform.translation.z);
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
        move_anim_changed(
            being_ent,
            &mut move_anim,
            glm.is_stepping() || was_stepping || had_motion_this_tick,
            &mut messages,
        );
    }
    writer.write_batch(messages.drain());
}

pub fn receive_gpos_from_server(
    fixed_time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut reader: MessageReader<SyncGpos>,
    mut beings: Query<(
        &mut GlobalTilePos,
        &mut CardinalDirection,
        &mut Transform,
        &mut GridLockedMovement,
        &SpeedMagnitude,
        Has<ComputedLocally>,
    ), With<Being>>,
) {
    for message in reader.read() {
        let SyncGpos { being_ent, gpos, dir, force_resync } = message;
        let Ok((mut being_gpos, mut facing_dir, mut transform, mut glm, speed_magnitude, computed_locally)) = beings.get_mut(*being_ent) else {
            continue;
        };
        *facing_dir = *dir;
        if computed_locally {
            if *force_resync {
                *being_gpos = *gpos;
                glm.clear_step(*gpos);
                transform.translation = gpos.to_translation(transform.translation.z);
                commands.entity(*being_ent).remove::<PendingTileCorrection>();
                trace!(
                    target: MOVEMENT_SYSTEM,
                    "Forced client resync for {:?}: {:?} facing {:?}",
                    being_ent,
                    gpos,
                    dir
                );
                file_log(
                    "move",
                    "client",
                    &format!("forced_resync ent={being_ent:?} gpos={gpos:?} facing={dir:?}"),
                );
                continue;
            }
            let delta = being_gpos.0 - gpos.0;
            if delta.x.abs().max(delta.y.abs()) < 1 {
                commands.entity(*being_ent).remove::<PendingTileCorrection>();
                continue;
            }
            if delta.x.abs().max(delta.y.abs()) > 1 {
                *being_gpos = *gpos;
                glm.clear_step(*gpos);
                transform.translation = gpos.to_translation(transform.translation.z);
                commands.entity(*being_ent).remove::<PendingTileCorrection>();
                trace!(target: MOVEMENT_SYSTEM, "Immediate client burst resync for {:?}: {:?} facing {:?}", being_ent, gpos, dir);
                file_log(
                    "move",
                    "client",
                    &format!("immediate_burst_resync ent={being_ent:?} gpos={gpos:?} facing={dir:?}"),
                );
                continue;
            }

            let _ = (&mut transform, &mut glm);
            commands.entity(*being_ent).insert(PendingTileCorrection {
                gpos: *gpos,
                secs_left: TILE_CORRECTION_INTERVAL_SECS,
            });
            continue;
        }
        let prev_gpos = *being_gpos;
        *being_gpos = *gpos;
        let dir = gpos.0 - prev_gpos.0;
        if dir.x.abs() + dir.y.abs() == 1 {
            glm.visual_origin_tile = prev_gpos.0;
            glm.step_dir = dir;
            glm.progress_ticks = 0;
            glm.step_ticks_total =
                ticks_per_tile(speed_magnitude.0, fixed_time.delta_secs(), dir).max(1);
            transform.translation = prev_gpos.to_translation(transform.translation.z);
        } else {
            glm.clear_step(*gpos);
            transform.translation = gpos.to_translation(transform.translation.z);
        }
        trace!(target: MOVEMENT_SYSTEM, "Received gpos {:?} facing {:?} for {:?}", gpos, dir, being_ent);
    }
}

pub fn apply_pending_tile_corrections(
    fixed_time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut beings: Query<(
        Entity,
        &mut GlobalTilePos,
        &mut Transform,
        &mut GridLockedMovement,
        &mut PendingTileCorrection,
    ), With<ComputedLocally>>,
) {
    for (being_ent, mut gpos, mut transform, mut glm, mut correction) in beings.iter_mut() {
        if gpos.0 == correction.gpos.0 {
            commands.entity(being_ent).remove::<PendingTileCorrection>();
            continue;
        }
        correction.secs_left -= fixed_time.delta_secs();
        if correction.secs_left > 0.0 {
            continue;
        }
        trace!(
            target: MOVEMENT_SYSTEM,
            "Snapping client tile correction for {:?}: {:?} -> {:?}",
            being_ent,
            gpos,
            correction.gpos
        );
        *gpos = correction.gpos;
        glm.clear_step(correction.gpos);
        transform.translation = correction.gpos.to_translation(transform.translation.z);
        commands.entity(being_ent).remove::<PendingTileCorrection>();
    }
}
