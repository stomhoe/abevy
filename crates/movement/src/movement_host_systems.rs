use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use param_sets::BlockingTileParamSet;
use sprite_animation_shared::MatchHeldSpritesAnimStateToBeingState;
use tilemap::tile::tile_components::Tile;
use tilemap_shared::{DimensionRef, GlobalTilePos};

use common::log_targets::MOVEMENT_SYSTEM;

use crate::{movement_components::*, movement_helpers::{secs_per_tile, ticks_per_tile, MAX_GRID_STEPS_PER_FIXED_TICK}, movement_messages::*};

const STEP_EARLY_TOLERANCE: f32 = 0.6;

#[derive(Clone, Copy, Debug)]
pub struct ClientStepRateState {
    last_server_secs: f64,
    step_credit: f32,
}

pub fn receive_step_request_from_client(
    time_fixed: Res<Time<Fixed>>,
    mut events: MessageReader<FromClient<SendStepRequest>>,
    mut blocking_tiles: BlockingTileParamSet,
    controlled_beings: Query<&ComputedBy>,
    mut beings: Query<(
        Entity,
        &DimensionRef,
        &SpeedMagnitude,
        &mut GlobalTilePos,
        &mut GridLockedMovement,
    ), (With<Being>, Without<ComputedLocally>, Without<Tile>)>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut messages: Local<Vec<ToClients<SyncGpos>>>,
    mut move_state_writer: MessageWriter<MatchHeldSpritesAnimStateToBeingState>,
    mut move_state_msgs: Local<Vec<MatchHeldSpritesAnimStateToBeingState>>,
    mut rate_states: Local<EntityHashMap<ClientStepRateState>>,
) {
    for from_client in events.read() {
        let SendStepRequest { being_ent, dir: step_dir, steps } = from_client.message.clone();
        let client_id = from_client.client_id;
        let Some(client_ent) = from_client.client_id.entity() else { continue; };
        let Ok(controlled_by) = controlled_beings.get(being_ent) else {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped step request for uncontrolled/missing being {:?} from {:?}",
                being_ent,
                client_ent
            );
            continue;
        };
        if controlled_by.client_ent != client_ent {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped spoofed step request for {:?}: owner {:?}, sender {:?}",
                being_ent,
                controlled_by.client_ent,
                client_ent
            );
            continue;
        }
        let Ok((entity, &dim_ref, speed_magnitude, mut tile_pos, mut glm)) = beings.get_mut(being_ent) else {
            continue;
        };
        let Some(facing_dir) = blocking_tiles.get_being_direction(being_ent) else {
            continue;
        };
        let dir_vec = step_dir.to_dir_vec();
        let secs_per_step = secs_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec);
        if secs_per_step <= 0.0 {
            continue;
        }
        let requested_steps = steps.max(1).min(MAX_GRID_STEPS_PER_FIXED_TICK);
        let now = time_fixed.elapsed_secs_f64();
        let state = rate_states.entry(being_ent).or_insert(ClientStepRateState {
            last_server_secs: now,
            step_credit: 1.0,
        });
        let elapsed = (now - state.last_server_secs).max(0.0) as f32;
        state.last_server_secs = now;
        state.step_credit = (state.step_credit + elapsed / secs_per_step)
            .min(MAX_GRID_STEPS_PER_FIXED_TICK as f32 + STEP_EARLY_TOLERANCE);
        if state.step_credit + STEP_EARLY_TOLERANCE < requested_steps as f32 {
            messages.push(ToClients {
                mode: SendMode::Direct(client_id),
                message: SyncGpos {
                    being_ent,
                    gpos: *tile_pos,
                    dir: facing_dir,
                    force_resync: true,
                },
            });
            continue;
        }
        glm.ensure_grid_anchor(*tile_pos);
        let step_ticks = ticks_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec);
        let steps_taken = if step_ticks > 1 {
            let next_gpos = GlobalTilePos(tile_pos.0 + dir_vec);
            if blocking_tiles.is_blocked_at(dim_ref, next_gpos, entity) {
                messages.push(ToClients {
                    mode: SendMode::Direct(client_id),
                message: SyncGpos {
                    being_ent,
                    gpos: *tile_pos,
                    dir: facing_dir,
                    force_resync: true,
                },
            });
            continue;
            }
            match glm.try_start_step(
                &mut blocking_tiles,
                dim_ref,
                entity,
                &mut tile_pos,
                dir_vec,
                step_ticks,
            ) {
                TryStartStepOutcome::Successful => {
                    if facing_dir != step_dir {
                        let _ = blocking_tiles.set_being_direction(being_ent, step_dir);
                        move_state_msgs.push(MatchHeldSpritesAnimStateToBeingState(being_ent));
                    }
                    1
                }
                TryStartStepOutcome::AlreadyStepping => {
                    messages.push(ToClients {
                        mode: SendMode::Direct(client_id),
                        message: SyncGpos {
                            being_ent,
                            gpos: *tile_pos,
                            dir: facing_dir,
                            force_resync: true,
                        },
                    });
                    continue;
                }
                TryStartStepOutcome::Blocked => {
                    messages.push(ToClients {
                        mode: SendMode::Direct(client_id),
                        message: SyncGpos {
                            being_ent,
                            gpos: *tile_pos,
                            dir: facing_dir,
                            force_resync: true,
                        },
                    });
                    continue;
                }
                TryStartStepOutcome::IVec2ZeroDir | TryStartStepOutcome::ZeroStepTicks => continue,
            }
        } else {
            let steps_taken = glm.advance_steps_immediate(
                &mut blocking_tiles,
                dim_ref,
                entity,
                &mut tile_pos,
                dir_vec,
                requested_steps,
            );
            if steps_taken == 0 {
                messages.push(ToClients {
                    mode: SendMode::Direct(client_id),
                    message: SyncGpos {
                        being_ent,
                        gpos: *tile_pos,
                        dir: facing_dir,
                        force_resync: true,
                    },
                });
                continue;
            }
            if facing_dir != step_dir {
                let _ = blocking_tiles.set_being_direction(being_ent, step_dir);
                move_state_msgs.push(MatchHeldSpritesAnimStateToBeingState(being_ent));
            }
            steps_taken
        };
        state.step_credit = (state.step_credit - steps_taken as f32).max(0.0);
        messages.push(ToClients {
            mode: SendMode::Broadcast,
            message: SyncGpos {
                being_ent,
                gpos: *tile_pos,
                dir: step_dir,
                force_resync: false,
            },
        });
        trace!(
            target: MOVEMENT_SYSTEM,
            "Accepted step request for {:?}: dir {:?}, steps={}, target {:?}, credit {:.2}, expected {:.3}s",
            being_ent,
            step_dir,
            steps_taken,
            tile_pos,
            state.step_credit,
            secs_per_step
        );
    }
    writer.write_batch(messages.drain(..));
    move_state_writer.write_batch(move_state_msgs.drain(..));
}
