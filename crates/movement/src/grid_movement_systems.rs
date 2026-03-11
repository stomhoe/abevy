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

pub fn start_grid_locked_steps(
    time: Res<Time>,
    client_state: Res<State<ClientState>>,
    server_state: Res<State<ServerState>>,
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
    mut to_drain: Local<Vec<Entity>>,
) {
    for (
        entity,
        controlled_locally,
        controlled_by,
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
            step_duration_secs(move_state.speed_magnitude, dir),
            ticks_per_tile(move_state.speed_magnitude, time.delta_secs(), dir),
        ) {
            continue;
        }
        if server_state.get() == &ServerState::Running && !connected.is_empty() {
            let message = SyncGpos {
                being_ent: entity,
                gpos: *tile_pos,
            };
            let mode = match controlled_by {
                Some(controlled_by) => {
                    SendMode::BroadcastExcept(ClientId::Client(controlled_by.client_ent))
                }
                None => SendMode::Broadcast,
            };
            debug!(target: MOVEMENT_SYSTEM, "Sending gpos {:?} for {:?}", tile_pos, entity);
            messages.push(ToClients { mode, message });
        }
        *facing_dir = CardinalDirection::from_dir_vec(dir);
    }
    writer.write_batch(messages.drain(..));
}

pub fn progress_tile_transition_transform(
    time: Res<Time>,
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
        glm.progress_grid_step(*tile_pos, time.delta_secs());
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
    mut reader: MessageReader<SyncGpos>,
    mut beings: Query<(&mut GlobalTilePos, Has<ComputedLocally>), With<Being>>,
) {
    for message in reader.read() {
        let SyncGpos { being_ent, gpos } = message;
        let Ok((mut being_gpos, computed_locally)) = beings.get_mut(*being_ent) else {
            continue;
        };
        if computed_locally {
            continue;
        }
        *being_gpos = *gpos;
        debug!(target: MOVEMENT_SYSTEM, "Received gpos {:?} for {:?}", gpos, being_ent);
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
