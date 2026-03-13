use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use param_sets::BlockingTileParamSet;
use tilemap::tile::prelude::Tile;
use tilemap_shared::{CardinalDirection, DimensionRef, GlobalTilePos};

use common::file_logging::file_log;
use common::log_targets::MOVEMENT_SYSTEM;

use crate::{movement_components::*, movement_helpers::{secs_per_tile, ticks_per_tile}, movement_messages::*};

const STEP_CREDIT_CAP: f32 = 1.75;
const STEP_EARLY_TOLERANCE: f32 = 0.6;

#[derive(Clone, Copy, Debug)]
pub struct ClientStepRateState {
    last_server_secs: f64,
    step_credit: f32,
}

pub fn receive_step_request_from_client(
    time_fixed: Res<Time<Fixed>>,
    mut events: MessageReader<FromClient<SendStepRequest>>,
    blocking_tiles: BlockingTileParamSet,
    controlled_beings: Query<&ComputedBy>,
    mut beings: Query<(
        Entity,
        &DimensionRef,
        &SpeedMagnitude,
        &mut GlobalTilePos,
        &mut GridLockedMovement,
        &mut CardinalDirection,
    ), (With<Being>, Without<ComputedLocally>, Without<Tile>)>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut messages: Local<Vec<ToClients<SyncGpos>>>,
    mut to_drain: Local<Vec<Entity>>,
    mut rate_states: Local<EntityHashMap<ClientStepRateState>>,
) {
    for from_client in events.read() {
        let SendStepRequest { being_ent, dir: step_dir } = from_client.message.clone();
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
        let Ok((entity, &dim_ref, speed_magnitude, mut tile_pos, mut glm, mut facing_dir)) = beings.get_mut(being_ent) else {
            continue;
        };
        let dir_vec = step_dir.to_dir_vec();
        glm.step_dir = dir_vec;
        if *facing_dir != step_dir {
            *facing_dir = step_dir;
        }
        let secs_per_step = secs_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec);
        if secs_per_step <= 0.0 {
            continue;
        }
        let now = time_fixed.elapsed_secs_f64();
        let state = rate_states.entry(being_ent).or_insert(ClientStepRateState {
            last_server_secs: now,
            step_credit: 1.0,
        });
        let elapsed = (now - state.last_server_secs).max(0.0) as f32;
        state.last_server_secs = now;
        state.step_credit = (state.step_credit + elapsed / secs_per_step).min(STEP_CREDIT_CAP);
        if state.step_credit + STEP_EARLY_TOLERANCE < 1.0 {
            messages.push(ToClients {
                mode: SendMode::Direct(client_id),
                message: SyncGpos {
                    being_ent,
                    gpos: *tile_pos,
                    dir: step_dir,
                    force_resync: true,
                },
            });
            continue;
        }
        state.step_credit -= 1.0;
        let next_gpos = GlobalTilePos(tile_pos.0 + dir_vec);
        if blocking_tiles.is_blocked_at(&mut to_drain, dim_ref, next_gpos, entity) {
            messages.push(ToClients {
                mode: SendMode::Direct(client_id),
                message: SyncGpos {
                    being_ent,
                    gpos: *tile_pos,
                    dir: step_dir,
                    force_resync: true,
                },
            });
            continue;
        }
        glm.ensure_grid_anchor(*tile_pos);
        let step_result = glm.try_start_step(
            &blocking_tiles,
            &mut to_drain,
            dim_ref,
            entity,
            &mut tile_pos,
            dir_vec,
            ticks_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec),
        );
        match step_result {
            TryStartStepOutcome::Successful => {
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
                    "Accepted step request for {:?}: dir {:?}, target {:?}, credit {:.2}, expected {:.3}s",
                    being_ent,
                    step_dir,
                    tile_pos,
                    state.step_credit,
                    secs_per_step
                );
            }
            TryStartStepOutcome::AlreadyStepping => {
                glm.start_step(
                    &mut tile_pos,
                    dir_vec,
                    ticks_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec),
                );
                messages.push(ToClients {
                    mode: SendMode::Broadcast,
                    message: SyncGpos {
                        being_ent,
                        gpos: *tile_pos,
                        dir: step_dir,
                        force_resync: false,
                    },
                });
                debug!(
                    target: MOVEMENT_SYSTEM,
                    "Accepted overlapping step request for {:?}: dir {:?}, target {:?}, credit {:.2}, expected {:.3}s",
                    being_ent,
                    step_dir,
                    tile_pos,
                    state.step_credit,
                    secs_per_step
                );
            }
            TryStartStepOutcome::Blocked => {
                error!(
                    target: MOVEMENT_SYSTEM,
                    "Unexpected blocked try_start_step after precheck for {:?}: dir {:?}, target {:?}",
                    being_ent,
                    step_dir,
                    next_gpos
                );
            }
            TryStartStepOutcome::IVec2ZeroDir | TryStartStepOutcome::ZeroStepTicks => {}
        }
    }
    writer.write_batch(messages.drain(..));
}
