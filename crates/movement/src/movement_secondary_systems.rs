use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_enhanced_input::actions;
use bevy_enhanced_input::bindings;
use bevy_enhanced_input::prelude::{Action, Actions, Axial, Bindings, Cardinal, DeadZone, Down};
use common::log_targets::MOVEMENT_SYSTEM;
use sprite_animation_shared::BeingChangedMoveState;
use tilemap_shared::CardinalDirection;

use ::being_shared::*;
use ac_input::ac_input_actions::*;

use crate::movement_components::{GridLockedMovement, MoveVecMag};
use crate::movement_helpers::normalize_to_axis_dir;

pub const INPUT_DEADZONE: f32 = 0.2;

pub fn update_facing_dir(
    mut query: Query<(Entity, &MoveVecMag, Option<&GridLockedMovement>, &mut CardinalDirection)>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut messages: Local<HashSet<BeingChangedMoveState>>,
) {
    for (being_ent, move_state, glm, mut facing_dir) in query.iter_mut() {
        let dir = glm
            .and_then(|glm| {
                if glm.step_dir == IVec2::ZERO {
                    None
                } else {
                    Some(glm.step_dir)
                }
            })
            .unwrap_or_else(|| normalize_to_axis_dir(move_state.norm_move_dir));
        let next = if dir == IVec2::ZERO {
            *facing_dir
        } else {
            CardinalDirection::from_dir_vec(dir)
        };
        if *facing_dir != next {
            *facing_dir = next;
            trace!(target: MOVEMENT_SYSTEM, "Facing updated for {:?} to {:?}", being_ent, next);
            messages.insert(BeingChangedMoveState(being_ent));
        }
    }
    writer.write_batch(messages.drain());
}

pub fn add_being_input_context(
    mut commands: Commands,
    being_query: Query<
        Entity,
        (
            Or<(With<ComputedLocally>, With<ControlledBy>)>,
            Without<Actions<BeingInputContext>>,
        ),
    >,
) {
    for being_ent in being_query.iter() {
        commands.entity(being_ent).try_insert((
            BeingInputContext,
            actions!(BeingInputContext[
                (
                    Action::<BeingMoveAction>::new(),
                    Down::default(),
                    DeadZone::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Cardinal::arrows(),
                        Cardinal::dpad(),
                        Axial::left_stick(),
                    )),
                ),
                (
                    Action::<BeingMeleeAttackAction>::new(),
                    bindings![KeyCode::ControlLeft],
                ),
            ]),
        ));
    }
}

/// Emits a `BeingChangedMoveState` message when the speed magnitude changes.
pub fn emit_move_state_on_movevecmag_speed_mag_change(
    query: Query<(Entity, &MoveVecMag)>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut prev_by_ent: Local<EntityHashMap<(Vec2, f32)>>,
    mut messages: Local<Vec<BeingChangedMoveState>>,
) {
    for (ent, move_vec_mag) in query.iter() {
        let curr = (move_vec_mag.norm_move_dir, move_vec_mag.speed_magnitude);
        let Some(prev) = prev_by_ent.get(&ent) else {
            prev_by_ent.insert(ent, curr);
            continue;
        };
        if prev != &curr {
            messages.push(BeingChangedMoveState(ent));
            prev_by_ent.insert(ent, curr);
        }
    }
    writer.write_batch(messages.drain(..));
}
