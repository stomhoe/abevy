use ::being_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use param_sets::BlockingTileParamSet;
use tilemap::tile::prelude::Tile;
use tilemap_shared::{CardinalDirection, DimensionRef, GlobalTilePos};

use common::log_targets::MOVEMENT_SYSTEM;

use crate::{movement_components::*, movement_helpers::ticks_per_tile, movement_messages::*};

pub fn receive_step_request_from_client(
    fixed_time: Res<Time<Fixed>>,
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
) {
    for from_client in events.read() {
        let SendStepRequest { being_ent, gpos } = from_client.message.clone();
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
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped step request for {:?}: missing movement state or unexpectedly ComputedLocally",
                being_ent
            );
            continue;
        };
        let dir = gpos.0 - tile_pos.0;
        if dir == IVec2::ZERO {
            continue;
        }
        if blocking_tiles.is_blocked_at(&mut to_drain, dim_ref, gpos, entity) {
            debug!(
                target: MOVEMENT_SYSTEM,
                "Rejected step request for {:?}: blocked target {:?}",
                being_ent,
                gpos
            );
            continue;
        }
        glm.ensure_grid_anchor(*tile_pos);
        glm.start_step(
            &mut tile_pos,
            dir,
            ticks_per_tile(speed_magnitude.0, fixed_time.delta_secs(), dir),
        );
        *facing_dir = CardinalDirection::from_dir_vec(dir);
        messages.push(ToClients {
            mode: SendMode::Broadcast,
            message: SyncGpos { being_ent, gpos: *tile_pos },
        });
        debug!(
            target: MOVEMENT_SYSTEM,
            "Accepted step request for {:?}: target {:?}",
            being_ent,
            tile_pos
        );
    }
    writer.write_batch(messages.drain(..));
}
