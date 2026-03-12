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
use crate::movement_log::movement_log;
use crate::movement_messages::*;

const TILE_CORRECTION_INTERVAL_SECS: f32 = 2.0;

pub fn start_grid_locked_steps(
    fixed_time: Res<Time<Fixed>>,
    client_state: Res<State<ClientState>>,
    connected: Query<&player::player_components::Player, Without<player::player_components::Mine>>,
    blocking_tiles: BlockingTileParamSet,
    mut beings: Query<(
        Entity,
        &InputMoveDir,
        &DimensionRef,
        &SpeedMagnitude,
        &mut GlobalTilePos,
        &mut GridLockedMovement,
        &mut CardinalDirection,
    ), (With<ComputedLocally>, Without<Tile>)>,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
    mut messages: Local<Vec<ToClients<SyncGpos>>>,
    mut step_writer: MessageWriter<SendStepRequest>,
    mut step_messages: Local<Vec<SendStepRequest>>,
    mut to_drain: Local<Vec<Entity>>,
) {
    for (
        entity,
        input_move_dir,
        &dim_ref,
        speed_magnitude,
        mut tile_pos,
        mut glm,
        mut facing_dir,
    ) in beings.iter_mut()
    {
        glm.ensure_grid_anchor(*tile_pos);
        let dir = input_move_dir.normalize_to_axis_dir();

        if !glm.try_start_step(
            &blocking_tiles,
            &mut to_drain,
            dim_ref,
            entity,
            &mut tile_pos,
            dir,
            ticks_per_tile(speed_magnitude.0, fixed_time.delta_secs(), dir),
        ) {
            continue;
        }
        *facing_dir = CardinalDirection::from_dir_vec(dir);
        if client_state.get() == &ClientState::Connected {
            step_messages.push(SendStepRequest {
                being_ent: entity,
                dir: CardinalDirection::from_dir_vec(dir),
            });
        }
        else if !connected.is_empty() {
            let message = SyncGpos {
                being_ent: entity,
                gpos: *tile_pos,
                dir: *facing_dir,
                force_resync: false,
            };
            trace!(target: MOVEMENT_SYSTEM, "Sending gpos {:?} for {:?}", tile_pos, entity);
            messages.push(ToClients {
                mode: SendMode::Broadcast,
                message: message.clone(),
            });
        }
    }
    writer.write_batch(messages.drain(..));
    step_writer.write_batch(step_messages.drain(..));
}

pub fn progress_tile_transition_transform(
    mut query: Query<(
        Entity,
        &GlobalTilePos,
        &mut Transform,
        &mut MoveAnimActive,
        &mut GridLockedMovement,
    )>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut messages: Local<HashSet<BeingChangedMoveState>>,
) {
    for (being_ent, tile_pos, mut transform, mut move_anim, mut glm) in query.iter_mut() {
        glm.ensure_grid_anchor(*tile_pos);
        glm.progress_grid_step(*tile_pos);
        let new_translation = glm.grid_translation(*tile_pos, transform.translation.z);
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
        move_anim_changed(being_ent, &mut move_anim, glm.is_stepping(), &mut messages);
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
                movement_log(
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
