use ::being_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_CONTROL;
use faction_shared::BelongsToAPlayerFaction;
use game_common::game_common_components::CameraTarget;
use movement::movement_components::{InputMaxSpeed, InputMoveDir, InputSpeedThrottleMult};
use player::{player_components::{HostPlayer, Mine, Player}, prelude::MyPlayer};
use tilemap::chunking::chunking_components::ActivatingChunks;
use tilemap_shared::LoadChunksAround;

pub fn add_activates_chunks(
    mut cmd: Commands,
    query: Query<Entity, (With<Being>, Added<BelongsToAPlayerFaction>)>,
    mut removed: RemovedComponents<BelongsToAPlayerFaction>,
    chunk_range: Res<LoadChunksAround>,
) {
    let iter = query.iter();
    let mut activates_chunks = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
    iter.for_each(|ent| activates_chunks.push((ent, (*chunk_range, ))));
    for ent in removed.read() {
        cmd.entity(ent).try_remove::<(LoadChunksAround, ActivatingChunks)>();
    }
    cmd.try_insert_batch(activates_chunks);
}

pub fn sync_player_being_chunk_ranges(
    default_chunk_range_for_player_beings: Res<LoadChunksAround>,
    mut query: Query<&mut LoadChunksAround, (With<Being>, With<BelongsToAPlayerFaction>)>,
) {
    if !default_chunk_range_for_player_beings.is_changed() {
        return;
    }
    for mut chunk_range in query.iter_mut() {
        *chunk_range = *default_chunk_range_for_player_beings;
    }
}
#[allow(unused_parens, )]
pub fn on_control_change(
    mut commands: Commands,
    self_player: Query<(Entity, Has<HostPlayer>), (MyPlayer)>,
    self_player_became_mine: Query<(), (MyPlayer, Added<Mine>)>,
    changed_query: Query<(Entity, &ComputedBy, Has<CameraTarget>), (Changed<ComputedBy>, )>,
    computed_by_query: Query<(Entity, &ComputedBy, Has<CameraTarget>)>,
    mut input_dirs: Query<&mut InputMoveDir>,
    mut removed_controlled_by: RemovedComponents<ComputedBy>,
    default_chunk_range_for_player_beings: Res<LoadChunksAround>,
) {
    for being_ent in removed_controlled_by.read() {
        commands.entity(being_ent).try_remove::<(ComputedLocally, HumanControlled, CameraTarget)>();
    }
    let Ok((self_entity, is_host)) = self_player.single() else {
        debug_once!(target: BEING_CONTROL, "Skipping control refresh until local player is marked Mine");
        return;
    };
    let mut apply_control_change = |being_ent: Entity, controlled_by: &ComputedBy, is_camera_target: bool| {
        if let Ok(mut input_dir) = input_dirs.get_mut(being_ent) {
            trace!(target: BEING_CONTROL, "InputMoveDir reset on control change for {:?}: {:?} -> {:?}", being_ent, input_dir.0, Vec2::ZERO);
            input_dir.0 = Vec2::ZERO;
        }
        if controlled_by.client_ent == self_entity {
            info!(target: BEING_CONTROL, "debug {:?} is now computed locally by self", being_ent);
            commands.entity(being_ent).try_insert_if_new((ComputedLocally, ));
            if controlled_by.human_dc_input {
                debug!(target: BEING_CONTROL, "Entity {:?} is now a CameraTarget due to human input", being_ent);
                commands.entity(being_ent).try_insert((HumanControlled, CameraTarget::default(), *default_chunk_range_for_player_beings, ));
                commands.entity(being_ent).try_insert(InputSpeedThrottleMult(1.0));
                commands.entity(being_ent).try_remove::<InputMaxSpeed>();
            } else {
                debug!(target: BEING_CONTROL, "Entity {:?} is no longer a CameraTarget", being_ent);
                commands.entity(being_ent).try_remove::<(CameraTarget, HumanControlled, )>();
            }
        } else {
            commands.entity(being_ent).try_remove::<(ComputedLocally, CameraTarget)>();
            if !is_host {
                commands.entity(being_ent).try_remove::<HumanControlled>();
                if !is_camera_target {
                    commands.entity(being_ent).try_remove::<(LoadChunksAround, ActivatingChunks)>();
                }
            } else {
                commands.entity(being_ent).try_insert(HumanControlled);
            }
        }
    };
    if !self_player_became_mine.is_empty() {
        for (being_ent, controlled_by, is_camera_target) in computed_by_query.iter() {
            apply_control_change(being_ent, controlled_by, is_camera_target);
        }
        return;
    }
    for (being_ent, controlled_by, is_camera_target) in changed_query.iter() {
        apply_control_change(being_ent, controlled_by, is_camera_target);
    }
}

pub fn assign_uncomputed_beings_to_host(
    mut commands: Commands,
    self_player: Query<Entity, (With<Mine>, With<Player>, With<HostPlayer>)>,
    beings: Query<Entity, (With<Being>, Without<ComputedBy>)>,
) {
    let Ok(self_entity) = self_player.single() else {
        return;
    };
    for being_ent in beings.iter() {
        commands.entity(being_ent).try_insert(ComputedBy {
            client_ent: self_entity,
            human_dc_input: false,
        });
    }
}
