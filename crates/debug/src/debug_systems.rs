use bevy::prelude::*;
use game_common::game_common_components::Dead;
use bevy_replicon::prelude::FromClient;
use ::being_shared::{Being, GridLockedMovement, GridLockedMovementVisual};
use being::body::{BodySums, DamageDistributeMode, HeldBody, IncHealthDamageOrHeal};
use tilemap_shared::{DimensionRef, GlobalTilePos};

use crate::debug_messages::{
    ClientDebugSetBeingCurrentBloodRequest, ClientDebugSetBeingCurrentHpRequest,
    ClientDebugSetBeingDimensionRequest, ClientDebugKillBeingRequest,
    ClientDebugReviveBeingRequest, ClientDebugTeleportBeingRequest,
};

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

#[allow(unused_parens, )]
pub fn receive_client_debug_set_being_current_hp_request(
    mut requests: MessageReader<FromClient<ClientDebugSetBeingCurrentHpRequest>>,
    held_body_query: Query<&HeldBody>,
    body_sums_query: Query<&BodySums>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok(held_body) = held_body_query.get(being_ent) else {
            continue;
        };
        let body_ent = held_body.entity();
        let Ok(body_sums) = body_sums_query.get(body_ent) else {
            continue;
        };
        let current_hp = body_sums.current_hp;
        damage_messages.write(IncHealthDamageOrHeal {
            source_ent: being_ent,
            target_ent: body_ent,
            amount: current_hp - request.message.current_hp,
            distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
        });
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_set_being_current_blood_request(
    mut requests: MessageReader<FromClient<ClientDebugSetBeingCurrentBloodRequest>>,
    held_body_query: Query<&HeldBody>,
    mut body_sums_query: Query<&mut BodySums>,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok(held_body) = held_body_query.get(being_ent) else {
            continue;
        };
        let body_ent = held_body.entity();
        let Ok(mut body_sums) = body_sums_query.get_mut(body_ent) else {
            continue;
        };
        body_sums.blood = request.message.blood.max(0.0);
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_kill_being_request(
    mut requests: MessageReader<FromClient<ClientDebugKillBeingRequest>>,
    held_body_query: Query<&HeldBody>,
    body_sums_query: Query<&BodySums>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
    mut being_cmd: Commands,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok(held_body) = held_body_query.get(being_ent) else {
            continue;
        };
        let body_ent = held_body.entity();
        let Ok(body_sums) = body_sums_query.get(body_ent) else {
            continue;
        };
        damage_messages.write(IncHealthDamageOrHeal {
            source_ent: being_ent,
            target_ent: body_ent,
            amount: body_sums.current_hp.max(0.0),
            distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
        });
        being_cmd.entity(being_ent).try_insert_if_new(Dead);
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_revive_being_request(
    mut requests: MessageReader<FromClient<ClientDebugReviveBeingRequest>>,
    held_body_query: Query<&HeldBody>,
    mut body_sums_query: Query<&mut BodySums>,
    mut being_query: Query<(&mut Transform, &GlobalTilePos), With<Being>>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
    mut being_cmd: Commands,
) {
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok(held_body) = held_body_query.get(being_ent) else {
            continue;
        };
        let body_ent = held_body.entity();
        let Ok(body_sums) = body_sums_query.get_mut(body_ent) else {
            continue;
        };
        damage_messages.write(IncHealthDamageOrHeal {
            source_ent: being_ent,
            target_ent: body_ent,
            amount: body_sums.current_hp - body_sums.total_hp.max(0.0),
            distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
        });

        if let Ok((mut transform, gpos)) = being_query.get_mut(being_ent) {
            let z = transform.translation.z;
            *transform = Transform::from_translation(gpos.to_translation(z));
        }
        being_cmd.entity(being_ent).try_remove::<Dead>();
        if let Ok(mut body_sums) = body_sums_query.get_mut(body_ent) {
            body_sums.blood = body_sums.blood_capacity.max(0.0);
        }
    }
}
