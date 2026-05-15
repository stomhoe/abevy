use being_shared::{Being, ComputedLocally};
use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::StrId;
use modifier_shared::WallPhaser;
use param_sets::BlockingTileParamSet;
use common::log_targets::{BEING_SYSTEM, MOVEMENT_SYSTEM};
use common::file_logging::file_log;
use ::sprite_animation_shared::*;

use tilemap::tile::tile_components::Tile;
use tilemap_shared::{BeingsAtGpos, *};

use being_shared::movement_shared_components::*;
use crate::grid_movement_helpers::*;
use crate::movement_helpers::*;
use crate::movement_messages::*;

const TILE_CORRECTION_INTERVAL_SECS: f32 = 2.0;



#[allow(unused_parens)]
pub fn sync_occupancy_for_beings_at_gpos_res(
    mut beings_at_gpos: ResMut<BeingsAtGpos>,
    mut removed_beings: RemovedComponents<Being>,
    query: Query<
        (
            Entity, &DimensionRef, &GlobalTilePos,
            Option<&CardinalDirection>,
            Option<&InteractionZones>,
        ),
        (
            With<Being>,//<-don't delete
            Or<(Added<Being>, Changed<GlobalTilePos>, Changed<DimensionRef>,
                Changed<CardinalDirection>, Changed<InteractionZones>)>,
        ),
    >,
    mut reused_colmask_vec: Local<Vec<GlobalTilePos>>,
) {
    for ent in removed_beings.read() {
        let _ = beings_at_gpos.remove_being_ent_entries(ent);
    }
    for (being_ent, &dim_ref, &gpos, facing_dir, interaction_zones) in query.iter() {
        reused_colmask_vec.clear();
        if let Some(interaction_zones) = interaction_zones {
            interaction_zones.gather_zone_positions_for_hashid(
                InteractionZones::COLLISION,
                facing_dir.copied().unwrap_or_default(),
                gpos.to_pixelpos(),
                &mut reused_colmask_vec,
            );
        }
        if reused_colmask_vec.is_empty() {
            reused_colmask_vec.push(gpos);
        }
        let _ = beings_at_gpos.update_being_occupy(
            being_ent,
            dim_ref,
            reused_colmask_vec.as_slice(),
        );
    }
}

pub fn resolve_overlapping_beings(
    mut beings: Query<(
        Entity,
        Option<&StrId>,
        &DimensionRef,
        &mut Transform,
        Option<&mut GridLockedMovement>,
        Option<&mut GridLockedMovementVisual>,
        Has<WallPhaser>,//use Has instead of Option
    ), With<Being>>,
    mut blocking_tiles: BlockingTileParamSet,
    mut occupied_positions: Local<HashMap<(DimensionRef, GlobalTilePos), Vec<Entity>>>,
    mut duplicate_positions: Local<Vec<((DimensionRef, GlobalTilePos), Vec<Entity>)>>,
    mut reserved_positions: Local<HashSet<(DimensionRef, GlobalTilePos)>>,
) {
    occupied_positions.clear();
    duplicate_positions.clear();
    reserved_positions.clear();

    for (being_ent, _, &dim_ref, ..) in beings.iter_mut() {
        let Ok(&gpos) = blocking_tiles.gpos_query.get(being_ent) else {
            continue;
        };
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
        (dim_ref.0.as_u64(), gpos.0.x, gpos.0.y)
    });

    let mut corrected_beings = 0usize;
    for &((dim_ref, source_gpos), ref overlapping_beings) in duplicate_positions.iter() {
        let keeper = overlapping_beings[0];
        let keeper_has_wallphaser = beings
            .get(keeper)
            .map(|(_, _, _, _, _, _, wallphaser)| wallphaser)
            .unwrap_or(false);
        for &being_ent in overlapping_beings.iter().skip(1) {
            let current_has_wallphaser = beings
                .get(being_ent)
                .map(|(_, _, _, _, _, _, wallphaser)| wallphaser)
                .unwrap_or(false);
            if keeper_has_wallphaser || current_has_wallphaser {
                continue;
            }

            let Some(found_gpos) = find_nearest_overlap_resolution_gpos(
                &mut blocking_tiles,
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

            let Ok((_, str_id, &current_dim, mut transform, movement, movement_visual, _)) = beings.get_mut(being_ent) else {
                continue;
            };
            let Ok(&current_gpos) = blocking_tiles.gpos_query.get(being_ent) else {
                continue;
            };
            if current_dim != dim_ref || current_gpos == found_gpos {
                continue;
            }

            let Ok(mut gpos) = blocking_tiles.gpos_query.get_mut(being_ent) else {
                continue;
            };
            *gpos = found_gpos;
            transform.translation = found_gpos.to_translation(transform.translation.z);
            if let Some(mut movement) = movement {
                let Some(mut movement_visual) = movement_visual else {
                    continue;
                };
                movement.clear_step(&mut movement_visual, found_gpos);
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
    connected: Query<&player_shared::player_components::Player, Without<player_shared::player_components::Mine>>,
    mut blocking_tiles: BlockingTileParamSet,
    mut beings: Query<(
        Entity,
        &FinalNormMoveDir,
        &DimensionRef,
        &SpeedMagnitude,
        &mut GridLockedMovement,
        &mut GridLockedMovementVisual,
    ), (With<ComputedLocally>, Without<Tile>)>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut sync_gpos_msgs: Local<Vec<ToClients<SyncGpos>>>,
    mut req_step_msgs: Local<Vec<SendStepRequest>>,
    mut burst_step_credit_by_ent: Local<EntityHashMap<f32>>,
    mut client_req_step_writer: MessageWriter<SendStepRequest>,
) {
    for (
        being_ent,
        final_norm_move_dir,
        &dim_ref,
        speed_magnitude,
        mut glm,
        mut glm_visual,
    ) in beings.iter_mut()
    {
        let Ok(&tile_pos) = blocking_tiles.gpos_query.get(being_ent) else {
            continue;
        };
        let mut tile_pos = tile_pos;
        glm.ensure_grid_anchor(&mut glm_visual, tile_pos);
        let dir = final_norm_move_dir.normalize_to_axis_dir();
        let next_dir = CardinalDirection::from_dir_vec(dir);
        let step_ticks = ticks_per_tile(speed_magnitude.0, fixed_time.delta_secs(), dir);

        let mut steps_to_request = 0;
        let mut moved_tile_pos = false;
        let should_sync_with_others = if step_ticks > 1 {
            burst_step_credit_by_ent.remove(&being_ent);
            match try_start_step(
                &mut glm,
                &mut glm_visual,
                &mut blocking_tiles,
                dim_ref,
                being_ent,
                &mut tile_pos,
                dir,
                step_ticks,
            ) {
                TryStartStepOutcome::Successful => {
                    steps_to_request = 1;
                    moved_tile_pos = true;
                    true
                }
                TryStartStepOutcome::Blocked => {
                    if blocking_tiles.get_being_direction(being_ent).is_some_and(|facing_dir| facing_dir == next_dir) {
                        false
                    } else {
                        steps_to_request = 0;
                        true
                    }
                }
                TryStartStepOutcome::IVec2ZeroDir
                | TryStartStepOutcome::AlreadyStepping
                | TryStartStepOutcome::ZeroStepTicks => false,
            }
        } else {
            let credit = burst_step_credit_by_ent.entry(being_ent).or_insert(0.0);
            *credit = (*credit + tiles_per_tick(speed_magnitude.0, fixed_time.delta_secs(), dir))
                .min(MAX_GRID_STEPS_PER_FIXED_TICK as f32);
            let requested_steps = credit.floor().min(MAX_GRID_STEPS_PER_FIXED_TICK as f32) as u16;
            let steps_taken = advance_steps_immediate(
                &mut glm,
                &mut glm_visual,
                &mut blocking_tiles,
                dim_ref,
                being_ent,
                &mut tile_pos,
                dir,
                requested_steps,
            );
            if steps_taken > 0 {
                *credit = (*credit - steps_taken as f32).max(0.0);
                steps_to_request = steps_taken;
                moved_tile_pos = true;
                true
            } else if dir != IVec2::ZERO
                && blocking_tiles.get_being_direction(being_ent).is_some_and(|facing_dir| facing_dir != next_dir)
            {
                steps_to_request = 0;
                true
            } else {
                false
            }
        };
        if !should_sync_with_others {
            if moved_tile_pos {
                let Ok(mut being_gpos) = blocking_tiles.gpos_query.get_mut(being_ent) else {
                    continue;
                };
                *being_gpos = tile_pos;
            }
            continue;
        }
        if moved_tile_pos {
            let Ok(mut being_gpos) = blocking_tiles.gpos_query.get_mut(being_ent) else {
                continue;
            };
            *being_gpos = tile_pos;
        }
        if client_state.get() == &ClientState::Connected {
            req_step_msgs.push(SendStepRequest {
                being_ent,
                dir: next_dir,
                steps: steps_to_request,
            });
        } else if !connected.is_empty() {
            let message = SyncGpos {
                being_ent,
                gpos: tile_pos,
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
        &mut GridLockedMovementVisual,
    )>,
) {
    for (being_ent, tile_pos, mut transform, mut move_anim, mut glm, mut glm_visual) in query.iter_mut() {
        let had_motion_this_tick = glm_visual.consume_recent_motion();
        glm.ensure_grid_anchor(&mut glm_visual, *tile_pos);
        let was_stepping = glm.is_stepping();
        glm.progress_grid_step(*tile_pos);
        let new_translation = glm.grid_translation(&glm_visual, *tile_pos, transform.translation.z);
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
        let _ = being_ent;
        move_anim.set(glm.is_stepping() || was_stepping || had_motion_this_tick);
    }
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
        &mut GridLockedMovementVisual,
        &SpeedMagnitude,
        Has<ComputedLocally>,
    ), With<Being>>,
) {
    for message in reader.read() {
        let SyncGpos { being_ent, gpos, dir, force_resync } = message;
        let Ok((mut being_gpos, mut facing_dir, mut transform, mut glm, mut glm_visual, speed_magnitude, computed_locally)) = beings.get_mut(*being_ent) else {
            continue;
        };
        *facing_dir = *dir;
        if computed_locally {
            if *force_resync {
                *being_gpos = *gpos;
                glm.clear_step(&mut glm_visual, *gpos);
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
                glm.clear_step(&mut glm_visual, *gpos);
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
            glm.step_dir = dir;
            glm.progress_ticks = 0;
            glm.step_ticks_total =
                ticks_per_tile(speed_magnitude.0, fixed_time.delta_secs(), dir).max(1);
            glm_visual.visual_origin_tile = prev_gpos.0;
            transform.translation = prev_gpos.to_translation(transform.translation.z);
        } else {
            glm.clear_step(&mut glm_visual, *gpos);
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
        &mut GridLockedMovementVisual,
        &mut PendingTileCorrection,
    ), With<ComputedLocally>>,
) {
    for (being_ent, mut gpos, mut transform, mut glm, mut glm_visual, mut correction) in beings.iter_mut() {
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
        glm.clear_step(&mut glm_visual, correction.gpos);
        transform.translation = correction.gpos.to_translation(transform.translation.z);
        commands.entity(being_ent).remove::<PendingTileCorrection>();
    }
}
