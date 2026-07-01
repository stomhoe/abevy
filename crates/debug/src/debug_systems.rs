use bevy::prelude::*;
use common::DEBUG;
use game_common::game_common_components::Dead;
use bevy_replicon::prelude::*;
use ::being_shared::{Being, GridLockedMovement, GridLockedMovementVisual};
use being::body::{BodySums, DamageDistributeMode, HeldBody, IncHealthDamageOrHeal};
use movement::movement_messages::SyncGpos;
use tilemap_shared::{CardinalDirection, DimensionRef, GlobalTilePos};

use crate::debug_messages::{
    ClientDebugSetBeingCurrentBloodRequest, ClientDebugSetBeingCurrentHpRequest,
    ClientDebugSetBeingDimensionRequest, ClientDebugKillBeingRequest,
    ClientDebugReviveBeingRequest, ClientDebugTeleportBeingRequest,
    LocalDebugTeleportBeingRequest,
};

#[allow(unused_parens, )]
pub fn receive_client_debug_set_being_dimension_request(
    request: On<FromClient<ClientDebugSetBeingDimensionRequest>>,
    mut being_query: Query<(&mut DimensionRef, ), (With<Being>,)>,
) {
    let being_ent = request.message.being_ent;
    let Ok((mut dim_ref, )) = being_query.get_mut(being_ent) else {
        return;
    };
    dim_ref.0 = request.message.dim_ref.0;
}

#[allow(unused_parens, )]
pub fn apply_local_debug_teleport_being_request(
    request: On<LocalDebugTeleportBeingRequest>,
    mut being_query: Query<(Option<&CardinalDirection>, ), (With<Being>,)>,
    mut cmd: Commands,
    mut writer: MessageWriter<ToClients<SyncGpos>>,
) {
    info!(
        target: DEBUG,
        "Received LocalDebugTeleportBeingRequest for being {:?} to gpos {:?}",
        request.being_ent,
        request.gpos
    );
    let being_ent = request.being_ent;
    let Ok((facing_dir, )) = being_query.get_mut(being_ent) else {
        warn!(
            target: DEBUG,
            "Teleport request target {:?} was not found for transform update",
            being_ent
        );
        return;
    };
    let new_gpos = request.gpos;


    cmd.entity(being_ent).try_insert(new_gpos);
    let facing_dir = facing_dir.copied().unwrap_or_default();
    writer.write(ToClients {
        targets: SendTargets::All,
        message: SyncGpos {
            being_ent,
            gpos: new_gpos,
            dir: facing_dir,
            force_resync: true,
        },
    });
    info!(
        target: DEBUG,
        "Teleported being {:?} to gpos {:?} facing {:?}",
        being_ent,
        new_gpos,
        facing_dir
    );
}

#[allow(unused_parens, )]
pub fn receive_client_debug_teleport_being_request(
    request: On<FromClient<ClientDebugTeleportBeingRequest>>,
    mut commands: Commands,
) {
    commands.trigger(LocalDebugTeleportBeingRequest {
        being_ent: request.message.being_ent,
        gpos: request.message.gpos,
    });
}

#[allow(unused_parens, )]
pub fn receive_client_debug_set_being_current_hp_request(
    request: On<FromClient<ClientDebugSetBeingCurrentHpRequest>>,
    held_body_query: Query<&HeldBody>,
    body_sums_query: Query<&BodySums>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
) {
    let being_ent = request.message.being_ent;
    let Ok(held_body) = held_body_query.get(being_ent) else {
        return;
    };
    let body_ent = held_body.entity();
    let Ok(body_sums) = body_sums_query.get(body_ent) else {
        return;
    };
    let current_hp = body_sums.current_hp;
    damage_messages.write(IncHealthDamageOrHeal {
        source_ent: being_ent,
        target_ent: body_ent,
        amount: current_hp - request.message.current_hp,
        distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
    });
}

#[allow(unused_parens, )]
pub fn receive_client_debug_set_being_current_blood_request(
    request: On<FromClient<ClientDebugSetBeingCurrentBloodRequest>>,
    held_body_query: Query<&HeldBody>,
    mut body_sums_query: Query<&mut BodySums>,
) {
    let being_ent = request.message.being_ent;
    let Ok(held_body) = held_body_query.get(being_ent) else {
        return;
    };
    let body_ent = held_body.entity();
    let Ok(mut body_sums) = body_sums_query.get_mut(body_ent) else {
        return;
    };
    body_sums.blood = request.message.blood.max(0.0);
}

#[allow(unused_parens, )]
pub fn receive_client_debug_kill_being_request(
    request: On<FromClient<ClientDebugKillBeingRequest>>,
    held_body_query: Query<&HeldBody>,
    body_sums_query: Query<&BodySums>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
    mut being_cmd: Commands,
) {
    let being_ent = request.message.being_ent;
    let Ok(held_body) = held_body_query.get(being_ent) else {
        return;
    };
    let body_ent = held_body.entity();
    let Ok(body_sums) = body_sums_query.get(body_ent) else {
        return;
    };
    damage_messages.write(IncHealthDamageOrHeal {
        source_ent: being_ent,
        target_ent: body_ent,
        amount: body_sums.current_hp.max(0.0),
        distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
    });
    being_cmd.entity(being_ent).try_insert_if_new(Dead);
}

#[allow(unused_parens, )]
pub fn receive_client_debug_revive_being_request(
    request: On<FromClient<ClientDebugReviveBeingRequest>>,
    held_body_query: Query<&HeldBody>,
    mut body_sums_query: Query<&mut BodySums>,
    mut being_query: Query<(&mut Transform, &GlobalTilePos), With<Being>>,
    mut damage_messages: ResMut<Messages<IncHealthDamageOrHeal>>,
    mut being_cmd: Commands,
) {
    let being_ent = request.message.being_ent;
    let Ok(held_body) = held_body_query.get(being_ent) else {
        return;
    };
    let body_ent = held_body.entity();
    let Ok(body_sums) = body_sums_query.get_mut(body_ent) else {
        return;
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
