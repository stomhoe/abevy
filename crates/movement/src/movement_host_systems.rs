use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::log_targets::MOVEMENT_SYSTEM;
use serde::Deserialize;
use param_sets::BlockingTileParamSet;
use sprite_animation_shared::MirrorHolderStateForSprite;
use tilemap::tile::tile_components::Tile;
use tilemap_shared::{CardinalDirection, DimensionRef, GlobalTilePos};

use crate::{grid_movement_helpers::{advance_steps_immediate, try_start_step}, movement_helpers::{secs_per_tile, ticks_per_tile}, movement_messages::*};

#[derive(Clone, Copy, Debug)]
pub struct ClientStepRateState {
    last_server_secs: f64,
    step_credit: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct DeferredStepRequest {
    client_id: ClientId,
    step_dir: CardinalDirection,
    steps: u16,
}

#[derive(Resource, Clone, Debug)]
pub struct MpSettings {
    pub step_early_tolerance: f32,
    pub max_grid_steps_per_fixed_tick: u16,
}

impl Default for MpSettings {
    fn default() -> Self {
        Self {
            step_early_tolerance: 0.6,
            max_grid_steps_per_fixed_tick: 256,
        }
    }
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct MpSettingsSeri {
    pub id: String,
    #[serde(default = "default_step_early_tolerance")]
    pub step_early_tolerance: f32,
    #[serde(default = "default_max_grid_steps_per_fixed_tick")]
    pub max_grid_steps_per_fixed_tick: u16,
}

impl MpSettingsSeri {
    pub fn to_settings(&self) -> MpSettings {
        MpSettings {
            step_early_tolerance: self.step_early_tolerance,
            max_grid_steps_per_fixed_tick: self.max_grid_steps_per_fixed_tick.max(1),
        }
    }
}

fn default_step_early_tolerance() -> f32 {
    MpSettings::default().step_early_tolerance
}

fn default_max_grid_steps_per_fixed_tick() -> u16 {
    MpSettings::default().max_grid_steps_per_fixed_tick
}

pub fn load_mp_settings(mut settings: ResMut<MpSettings>) {
    let db = match common::def_db::DefDatabase::<MpSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(MpSettingsSeri),
        &["mp.settings.ron"],
        |_| "mp_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(target: MOVEMENT_SYSTEM, "Failed loading MpSettingsSeri defs: {err:#}");
            return;
        }
    };
    let Some(first) = db.into_records().into_iter().next() else {
        return;
    };
    *settings = first.value.to_settings();
}

#[allow(unused_parens, )]
pub fn receive_step_request_from_client(
    time_fixed: Res<Time<Fixed>>,
    settings: Res<MpSettings>,
    mut events: MessageReader<FromClient<SendStepRequest>>,
    mut blocking_tiles: BlockingTileParamSet,
    controlled_beings: Query<(&ComputedBy, ), ()>,
    mut beings: Query<(
        Entity,
        &DimensionRef,
        &SpeedMagnitude,
        &mut GridLockedMovement,
        &mut GridLockedMovementVisual,
    ), (With<Being>, Without<ComputedLocally>, Without<Tile>)>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut messages: Local<Vec<ToClients<SyncGpos>>>,
    mut move_state_writer: MessageWriter<MirrorHolderStateForSprite>,
    mut move_state_msgs: Local<Vec<MirrorHolderStateForSprite>>,
    mut rate_states: Local<EntityHashMap<ClientStepRateState>>,
    mut deferred_requests: Local<EntityHashMap<DeferredStepRequest>>,
) {
    let mut pending_requests = std::mem::take(&mut *deferred_requests);
    for from_client in events.read() {
        let SendStepRequest {
            being_ent,
            dir: step_dir,
            steps,
        } = from_client.message.clone();
        pending_requests.insert(
            being_ent,
            DeferredStepRequest {
                client_id: from_client.client_id,
                step_dir,
                steps,
            },
        );
    }
    for (being_ent, deferred_request) in pending_requests {
        let DeferredStepRequest { client_id, step_dir, steps } = deferred_request;
        let Ok((controlled_by, )) = controlled_beings.get(being_ent) else {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped step request for uncontrolled/missing being {:?} from {:?}",
                being_ent,
                client_id
            );
            continue;
        };
        if let Some(client_ent) = client_id.entity() {
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
        } else if client_id != ClientId::Server {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped step request from unmapped non-server sender {:?} for {:?}",
                client_id,
                being_ent
            );
            continue;
        }
        let Ok((entity, &dim_ref, speed_magnitude, mut glm, mut glm_visual)) = beings.get_mut(being_ent) else {
            continue;
        };
        let Ok(&tile_pos) = blocking_tiles.gpos_query.get(entity) else {
            continue;
        };
        let mut curr_tile_pos = tile_pos;
        let Some(facing_dir) = blocking_tiles.get_being_direction(being_ent) else {
            continue;
        };
        if steps == 0 {
            if facing_dir != step_dir {
                let _ = blocking_tiles.set_being_direction(being_ent, step_dir);
                move_state_msgs.push(MirrorHolderStateForSprite(being_ent));
                messages.push(ToClients {
                    mode: SendMode::BroadcastExcept(client_id),
                    message: SyncGpos {
                        being_ent,
                        gpos: curr_tile_pos,
                        dir: step_dir,
                        force_resync: false,
                    },
                });
            }
            continue;
        }
        let dir_vec = step_dir.to_dir_vec();
        let secs_per_step = secs_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec);
        if secs_per_step <= 0.0 {
            continue;
        }
        let requested_steps = steps.max(1).min(settings.max_grid_steps_per_fixed_tick);
        let now = time_fixed.elapsed_secs_f64();
        let state = rate_states.entry(being_ent).or_insert(ClientStepRateState {
            last_server_secs: now,
            step_credit: 1.0,
        });
        let elapsed = (now - state.last_server_secs).max(0.0) as f32;
        state.last_server_secs = now;
        state.step_credit = (state.step_credit + elapsed / secs_per_step)
            .min(settings.max_grid_steps_per_fixed_tick as f32 + settings.step_early_tolerance);
        if state.step_credit + settings.step_early_tolerance < requested_steps as f32 {
            if elapsed <= f32::EPSILON {
                trace!(
                    target: MOVEMENT_SYSTEM,
                    "Deferred same-tick early request for {:?}: dir {:?}, steps={}, credit {:.2}",
                    being_ent,
                    step_dir,
                    requested_steps,
                    state.step_credit
                );
                deferred_requests.insert(being_ent, deferred_request);
                continue;
            }
            warn!(
                target: MOVEMENT_SYSTEM,
                "Rejected step request for {:?}: dir {:?}, steps={}, credit {:.2}, expected {:.3}s",
                being_ent,
                step_dir,
                requested_steps,
                state.step_credit,
                secs_per_step
            );
            messages.push(ToClients {
                mode: SendMode::Direct(client_id),
                message: SyncGpos {
                    being_ent,
                    gpos: curr_tile_pos,
                    dir: facing_dir,
                    force_resync: true,
                },
            });
            continue;
        }
        glm.ensure_grid_anchor(&mut glm_visual, curr_tile_pos);
        if glm.is_stepping() {
            trace!(
                target: MOVEMENT_SYSTEM,
                "Deferred request while server step in progress for {:?}: dir {:?}, current {:?}",
                being_ent,
                step_dir,
                curr_tile_pos
            );
            deferred_requests.insert(being_ent, deferred_request);
            continue;
        }
        let step_ticks = ticks_per_tile(speed_magnitude.0, time_fixed.delta_secs(), dir_vec);
        let steps_taken = if step_ticks > 1 {
            let next_gpos = GlobalTilePos(curr_tile_pos.0 + dir_vec);
            if blocking_tiles.is_blocked_at(dim_ref, next_gpos, entity) {
                warn!(
                    target: MOVEMENT_SYSTEM,
                    "Rejected blocked step request for {:?}: dir {:?}, target {:?}",
                    being_ent,
                    step_dir,
                    next_gpos
                );
                messages.push(ToClients {
                    mode: SendMode::Direct(client_id),
                    message: SyncGpos {
                        being_ent,
                        gpos: curr_tile_pos,
                        dir: facing_dir,
                        force_resync: true,
                    },
                });
                continue;
            }
            match try_start_step(
                &mut glm,
                &mut glm_visual,
                &mut blocking_tiles,
                dim_ref,
                entity,
                &mut curr_tile_pos,
                dir_vec,
                step_ticks,
                ) {
                TryStartStepOutcome::Successful => {
                    if facing_dir != step_dir {
                        let _ = blocking_tiles.set_being_direction(being_ent, step_dir);
                        move_state_msgs.push(MirrorHolderStateForSprite(being_ent));
                    }
                    1
                }
                TryStartStepOutcome::AlreadyStepping => {
                    warn!(
                        target: MOVEMENT_SYSTEM,
                        "Rejected already-stepping request for {:?}: dir {:?}, current {:?}",
                        being_ent,
                        step_dir,
                        curr_tile_pos
                    );
                    messages.push(ToClients {
                        mode: SendMode::Direct(client_id),
                        message: SyncGpos {
                            being_ent,
                            gpos: curr_tile_pos,
                            dir: facing_dir,
                            force_resync: true,
                        },
                    });
                    continue;
                }
                TryStartStepOutcome::Blocked => {
                    warn!(
                        target: MOVEMENT_SYSTEM,
                        "Rejected blocked step request for {:?}: dir {:?}, current {:?}",
                        being_ent,
                        step_dir,
                        curr_tile_pos
                    );
                    messages.push(ToClients {
                        mode: SendMode::Direct(client_id),
                        message: SyncGpos {
                            being_ent,
                            gpos: curr_tile_pos,
                            dir: facing_dir,
                            force_resync: true,
                        },
                    });
                    continue;
                }
                TryStartStepOutcome::IVec2ZeroDir | TryStartStepOutcome::ZeroStepTicks => continue,
            }
        } else {
            let steps_taken = advance_steps_immediate(
                &mut glm,
                &mut glm_visual,
                &mut blocking_tiles,
                dim_ref,
                entity,
                &mut curr_tile_pos,
                dir_vec,
                requested_steps,
            );
            if steps_taken == 0 {
                warn!(
                    target: MOVEMENT_SYSTEM,
                    "Rejected immediate step request for {:?}: dir {:?}, requested {}, current {:?}",
                    being_ent,
                    step_dir,
                    requested_steps,
                    curr_tile_pos
                );
                messages.push(ToClients {
                    mode: SendMode::Direct(client_id),
                    message: SyncGpos {
                        being_ent,
                        gpos: curr_tile_pos,
                        dir: facing_dir,
                        force_resync: true,
                    },
                });
                continue;
            }
            if facing_dir != step_dir {
                let _ = blocking_tiles.set_being_direction(being_ent, step_dir);
                move_state_msgs.push(MirrorHolderStateForSprite(being_ent));
            }
            steps_taken
        };
        state.step_credit = (state.step_credit - steps_taken as f32).max(0.0);
        let Ok(mut being_gpos) = blocking_tiles.gpos_query.get_mut(entity) else {
            continue;
        };
        *being_gpos = curr_tile_pos;
        messages.push(ToClients {
            mode: SendMode::BroadcastExcept(client_id),
            message: SyncGpos {
                being_ent,
                gpos: curr_tile_pos,
                dir: step_dir,
                force_resync: false,
            },
        });
        info!(
            target: MOVEMENT_SYSTEM,
            "Accepted step request for {:?}: dir {:?}, steps={}, target {:?}, credit {:.2}, expected {:.3}s",
            being_ent,
            step_dir,
            steps_taken,
            curr_tile_pos,
            state.step_credit,
            secs_per_step
        );
    }
    writer.write_batch(messages.drain(..));
    move_state_writer.write_batch(move_state_msgs.drain(..));
}
