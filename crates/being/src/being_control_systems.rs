use ::being_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_CONTROL;
use game_common::game_common_components::CameraTarget;
use ::tilemap_shared::*;
use player_shared::player_components::{HostPlayer, Mine, MyPlayer, Player};

pub fn add_activates_chunks(
    mut cmd: Commands,
    query: Query<Entity, (With<Being>, Added<HumanControlled>)>,
    mut removed: RemovedComponents<HumanControlled>,
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

pub fn sync_being_chunk_ranges_to_resource(
    default_chunk_range_for_player_beings: Res<LoadChunksAround>,
    mut query: Query<&mut LoadChunksAround, With<Being>>,
) {
    for mut chunk_range in query.iter_mut() {
        *chunk_range = *default_chunk_range_for_player_beings;
    }
}

pub type ToRemoveBundle = (HumanControlled, CameraTarget, LoadChunksAround, ActivatingChunks, ClientChunkLoader, );

#[allow(unused_parens, )]
pub fn on_control_change(
    mut commands: Commands,
    self_player: Query<(Entity, Has<HostPlayer>), (MyPlayer)>,
    self_player_became_mine: Query<(), (MyPlayer, Added<Mine>)>,
    changed_query: Query<(Entity, &ComputedBy), (Changed<ComputedBy>, )>,
    computed_by_query: Query<(Entity, &ComputedBy)>,
    mut input_dirs: Query<&mut InputMoveDir>,
    mut input_speed_query: Query<(&mut InputSpeedThrottleMult, &mut InputMaxSpeed, ), (), >,
    mut removed_controlled_by: RemovedComponents<ComputedBy>,
    default_chunk_range_for_player_beings: Res<LoadChunksAround>,
) {
    for being_ent in removed_controlled_by.read() {
        commands.entity(being_ent).try_remove::<ToRemoveBundle>();
    }
    let Ok((self_entity, am_i_host)) = self_player.single() else {
        trace_once!(target: BEING_CONTROL, "Skipping control refresh until local player is marked Mine");
        return;
    };
    let mut apply_control_change = |being_ent: Entity, controlled_by: &ComputedBy| {
        if let Ok(mut input_dir) = input_dirs.get_mut(being_ent) {
            input_dir.0 = Vec2::ZERO;
        }
        if controlled_by.client_ent == self_entity {
            commands.entity(being_ent).try_insert_if_new((ComputedLocally, )).try_remove::<ClientChunkLoader>();
            if controlled_by.human_dc_input {
                commands.entity(being_ent).try_insert((HumanControlled, CameraTarget::default(), *default_chunk_range_for_player_beings, ));
                if let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) {
                    input_speed_throttle_mult.0 = 1.0;
                    input_max_speed.0 = f32::MAX;
                }
            } else {
                commands.entity(being_ent).try_remove::<ToRemoveBundle>();
            }
        } else {
            commands.entity(being_ent).try_remove::<(ComputedLocally, CameraTarget)>();
            if am_i_host {
                if controlled_by.human_dc_input {
                    commands.entity(being_ent).try_insert((HumanControlled, *default_chunk_range_for_player_beings, ClientChunkLoader, ));
                } else {
                    commands.entity(being_ent).try_remove::<ToRemoveBundle>();
                }
            } else {
                commands.entity(being_ent).try_remove::<ToRemoveBundle>();
            }
        }
    };
    if !self_player_became_mine.is_empty() {
        for (being_ent, controlled_by) in computed_by_query.iter() {
            apply_control_change(being_ent, controlled_by);
        }
        return;
    }
    for (being_ent, controlled_by) in changed_query.iter() {
        apply_control_change(being_ent, controlled_by);
    }
}

#[allow(unused_parens, )]
pub fn add_melee_target_comp_to_ai_controlled(
    mut commands: Commands,
    ai_controlled_beings: Query<
        Entity,
        (With<Being>, LocalAiControlled, Without<AiAutoMeleeTargets>, ),
    >,
    ceased_2b_ai_controlled: Query<Entity, Added<HumanControlled>>,
) {
    for being_ent in ai_controlled_beings.iter() {
        commands.entity(being_ent).try_insert_if_new(AiAutoMeleeTargets::default());
    }
    for being_ent in ceased_2b_ai_controlled.iter() {
        commands.entity(being_ent).try_remove::<AiAutoMeleeTargets>();
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
