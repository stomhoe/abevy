use ac_input::ac_input_actions::*;
use bevy::prelude::*;
use common::HashId;
use common::log_targets::DEBUG;
use game_common::game_common_components::Templ;
use bevy_replicon::prelude::FromClient;
use ::being_shared::{Being, GridLockedMovement, GridLockedMovementVisual};
use tilemap_shared::{DimensionRef, GlobalTilePos};
use modifier_shared::{
    modifier_components::*,
    modifier_move_bundles::SpeedModifier,
    modifier_types::WalkStrength,
};

use crate::debug_messages::{
    ClientDebugDecreaseSpeedRequest, ClientDebugIncreaseSpeedRequest, ClientDebugSetBeingDimensionRequest,
    ClientDebugTeleportBeingRequest,
};

fn apply_speed_factor(
    cmd: &mut Commands,
    being_ent: Entity,
    applied_modifiers: &AppliedModifiers,
    debug_modi_query: &mut Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
    factor: f32,
) {
    let debug_hash = HashId::hash("debug");
    for modifier_ent in applied_modifiers.iter() {
        let Ok((entity, hash_id, base_value, )) = debug_modi_query.get_mut(modifier_ent) else {
            continue;
        };
        if *hash_id != debug_hash {
            continue;
        }
        let new_value = match base_value {
            Some(mut base_value) => {
                let new_value = (base_value.0 * factor).max(0.1);
                base_value.0 = new_value;
                new_value
            }
            None => {
                let new_value = factor.max(0.1);
                cmd.entity(entity).insert(BaseValue(new_value));
                new_value
            }
        };
        trace!(target: DEBUG, "Applied debug speed factor {} to {:?}, new_base={}", factor, being_ent, new_value);
        return;
    }
    let new_value = factor.max(0.1);
    cmd.spawn((
        SpeedModifier::new(being_ent, being_ent, new_value, ApplyMode::Mul),
        debug_hash,
    ));
    trace!(target: DEBUG, "Applied debug speed factor {} to {:?}, new_base={}", factor, being_ent, new_value);
}

#[allow(unused_parens, )]
pub fn receive_debug_increase_speed_request(
    mut cmd: Commands,
    mut requests: MessageReader<LocalDebugIncreaseSpeedRequest>,
    controlled_beings_query: Query<(&AppliedModifiers, ), ()>,
    mut debug_modi_query: Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
) {
    for request in requests.read() {
        let &LocalDebugIncreaseSpeedRequest { being_ent } = request;
        let Ok((applied_modifiers, )) = controlled_beings_query.get(being_ent) else {
            continue;
        };
        apply_speed_factor(&mut cmd, being_ent, applied_modifiers, &mut debug_modi_query, 1.02);
    }
}

#[allow(unused_parens, )]
pub fn receive_debug_decrease_speed_request(
    mut cmd: Commands,
    mut requests: MessageReader<LocalDebugDecreaseSpeedRequest>,
    controlled_beings_query: Query<(&AppliedModifiers, ), ()>,
    mut debug_modi_query: Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
) {
    for request in requests.read() {
        let &LocalDebugDecreaseSpeedRequest { being_ent } = request;
        let Ok((applied_modifiers, )) = controlled_beings_query.get(being_ent) else {
            continue;
        };
        apply_speed_factor(&mut cmd, being_ent, applied_modifiers, &mut debug_modi_query, 0.98);
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_increase_speed_request(
    mut cmd: Commands,
    mut requests: MessageReader<FromClient<ClientDebugIncreaseSpeedRequest>>,
    controlled_beings_query: Query<(&AppliedModifiers, ), ()>,
    mut debug_modi_query: Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok((applied_modifiers, )) = controlled_beings_query.get(being_ent) else {
            continue;
        };
        apply_speed_factor(&mut cmd, being_ent, applied_modifiers, &mut debug_modi_query, 1.02);
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_decrease_speed_request(
    mut cmd: Commands,
    mut requests: MessageReader<FromClient<ClientDebugDecreaseSpeedRequest>>,
    controlled_beings_query: Query<(&AppliedModifiers, ), ()>,
    mut debug_modi_query: Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok((applied_modifiers, )) = controlled_beings_query.get(being_ent) else {
            continue;
        };
        apply_speed_factor(&mut cmd, being_ent, applied_modifiers, &mut debug_modi_query, 0.98);
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_set_being_dimension_request(
    mut requests: MessageReader<FromClient<ClientDebugSetBeingDimensionRequest>>,
    mut being_query: Query<(&mut DimensionRef, ), (With<Being>,)>,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok((mut dim_ref, )) = being_query.get_mut(being_ent) else {
            continue;
        };
        dim_ref.0 = request.message.dim_ref.0;
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_teleport_being_request(
    mut requests: MessageReader<FromClient<ClientDebugTeleportBeingRequest>>,
    mut being_query: Query<(&mut GlobalTilePos, &mut Transform, &mut GridLockedMovement, &mut GridLockedMovementVisual), (With<Being>,)>,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok((mut gpos, mut transform, mut grid_locked, mut grid_locked_visual)) = being_query.get_mut(being_ent) else {
            continue;
        };
        let new_gpos = request.message.gpos;
        let z = transform.translation.z;
        *gpos = new_gpos;
        *transform = Transform::from_translation(new_gpos.to_translation(z));
        *grid_locked = GridLockedMovement::default();
        *grid_locked_visual = GridLockedMovementVisual::default();
    }
}
