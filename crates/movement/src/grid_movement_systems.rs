use being_shared::{Being, ComputedLocally};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use param_sets::BlockingTileParamSet;
use common::log_targets::MOVEMENT_SYSTEM;
use sprite_animation_shared::{BeingChangedMoveState, MoveAnimActive};
use tilemap::tile::prelude::Tile;
use tilemap_shared::*;

use crate::movement_components::*;
use crate::movement_helpers::*;
use crate::movement_messages::*;

const TILE_CORRECTION_INTERVAL_SECS: f32 = 2.0;

pub fn start_grid_locked_steps(
    fixed_time: Res<Time<Fixed>>,
    client_state: Res<State<ClientState>>,
    server_state: Res<State<ServerState>>,
    trusted_player: Query<(), (With<player::player_components::Mine>, With<player::player_components::Player>, With<player::player_components::TrustedForMovement>)>,
    connected: Query<&player::player_components::Player, Without<player::player_components::Mine>>,
    blocking_tiles: BlockingTileParamSet,
    mut beings: Query<(
        Entity,
        Has<ComputedLocally>,
        Option<&being_shared::ComputedBy>,
        &InputMoveDir,
        &DimensionRef,
        &MoveVecMag,
        &mut GlobalTilePos,
        &mut GridLockedMovement,
        &mut CardinalDirection,
    ), Without<Tile>>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut messages: Local<Vec<ToClients<SyncGpos>>>,
    mut trusted_writer: MessageWriter<SendTrustedGpos>,
    mut trusted_messages: Local<Vec<SendTrustedGpos>>,
    mut to_drain: Local<Vec<Entity>>,
) {
    let is_trusted_client = client_state.get() == &ClientState::Connected && !trusted_player.is_empty();
    for (
        entity,
        controlled_locally,
        _controlled_by,
        input_move_dir,
        &dim_ref,
        move_state,
        mut tile_pos,
        mut glm,
        mut facing_dir,
    ) in beings.iter_mut()
    {
        if client_state.get() == &ClientState::Connected && !controlled_locally  {
            continue;
        }
        glm.ensure_grid_anchor(*tile_pos);
        let dir = normalize_to_axis_dir(input_move_dir.0);
        if !glm.try_start_step(
            &blocking_tiles,
            &mut to_drain,
            dim_ref,
            entity,
            &mut tile_pos,
            dir,
            ticks_per_tile(move_state.speed_magnitude, fixed_time.delta_secs(), dir),
        ) {
            continue;
        }
        if is_trusted_client {
            trusted_messages.push(SendTrustedGpos {
                being_ent: entity,
                gpos: *tile_pos,
            });
        }
        if server_state.get() == &ServerState::Running && !connected.is_empty() {
            let message = SyncGpos {
                being_ent: entity,
                gpos: *tile_pos,
            };
            debug!(target: MOVEMENT_SYSTEM, "Sending gpos {:?} for {:?}", tile_pos, entity);
            messages.push(ToClients {
                mode: SendMode::Broadcast,
                message: message.clone(),
            });
        }
        *facing_dir = CardinalDirection::from_dir_vec(dir);
    }
    writer.write_batch(messages.drain(..));
    trusted_writer.write_batch(trusted_messages.drain(..));
}

pub fn progress_tile_transition_transform(
    server_state: Res<State<ServerState>>,
    connected: Query<&player::player_components::Player, Without<player::player_components::Mine>>,
    mut query: Query<(
        Entity,
        &GlobalTilePos,
        Option<&being_shared::ComputedBy>,
        &mut Transform,
        &mut MoveAnimActive,
        &mut GridLockedMovement,
    )>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut messages: Local<HashSet<BeingChangedMoveState>>,
    mut sync_writer: MessageWriter<ToClients<SyncTransform>>,
    mut sync_messages: Local<Vec<ToClients<SyncTransform>>>,
) {
    let should_sync = server_state.get() == &ServerState::Running && !connected.is_empty();
    for (being_ent, tile_pos, controlled_by, mut transform, mut move_anim, mut glm) in query.iter_mut() {
        glm.ensure_grid_anchor(*tile_pos);
        glm.progress_grid_step(*tile_pos);
        let new_translation = glm.grid_translation(*tile_pos, transform.translation.z);
        if transform.translation != new_translation {
            transform.translation = new_translation;
            if should_sync {
                let message = SyncTransform {
                    being_ent,
                    transform: *transform,
                };
                let mode = match controlled_by {
                    Some(controlled_by) => {
                        SendMode::BroadcastExcept(ClientId::Client(controlled_by.client_ent))
                    }
                    None => SendMode::Broadcast,
                };
                debug!(target: MOVEMENT_SYSTEM, "Sending transform for {:?}", being_ent);
                sync_messages.push(ToClients { mode, message });
            }
        }
        move_anim_changed(being_ent, &mut move_anim, glm.is_stepping(), &mut messages);
    }
    writer.write_batch(messages.drain());
    sync_writer.write_batch(sync_messages.drain(..));
}

pub fn receive_gpos_from_server(
    mut commands: Commands,
    mut reader: MessageReader<SyncGpos>,
    mut beings: Query<(&mut GlobalTilePos, &mut Transform, &mut GridLockedMovement, Has<ComputedLocally>), With<Being>>,
) {
    for message in reader.read() {
        let SyncGpos { being_ent, gpos } = message;
        let Ok((mut being_gpos, mut transform, mut glm, computed_locally)) = beings.get_mut(*being_ent) else {
            continue;
        };
        if computed_locally {
            let delta = being_gpos.0 - gpos.0;
            if delta.x.abs().max(delta.y.abs()) < 1 {
                commands.entity(*being_ent).remove::<PendingTileCorrection>();
                continue;
            }
            debug!(
                target: MOVEMENT_SYSTEM,
                "Queued client tile correction for {:?}: {:?} -> {:?}",
                being_ent,
                being_gpos,
                gpos
            );
            let _ = (&mut transform, &mut glm);
            commands.entity(*being_ent).insert(PendingTileCorrection {
                gpos: *gpos,
                secs_left: TILE_CORRECTION_INTERVAL_SECS,
            });
            continue;
        }
        *being_gpos = *gpos;
        debug!(target: MOVEMENT_SYSTEM, "Received gpos {:?} for {:?}", gpos, being_ent);
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
        debug!(
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

pub fn receive_transform_from_server(
    mut reader: MessageReader<SyncTransform>,
    mut beings: Query<(&mut Transform, Has<ComputedLocally>),>,
) {
    for message in reader.read() {
        let SyncTransform { being_ent, transform } = message;
        let Ok((mut being_transform, computed_locally)) = beings.get_mut(*being_ent) else {
            continue;
        };
        if computed_locally {
            continue;
        }
        *being_transform = transform.clone();
        debug!(target: MOVEMENT_SYSTEM, "Received transform for {:?}", being_ent);
    }
}
