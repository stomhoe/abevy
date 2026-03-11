use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_enhanced_input::actions;
use bevy_enhanced_input::bindings;
use bevy_enhanced_input::prelude::{Action, Actions, Axial, Bindings, Cardinal, DeadZone};
use bevy_replicon::prelude::Replicated;
use common::log_targets::MOVEMENT_SYSTEM;
use common::prelude::*;
use modifier_shared::modifier_components::AppliedModifiers;
use sprite_animation_shared::MoveAnimActive;
use sprite_animation_shared::BeingChangedMoveState;
use tilemap_shared::{CardinalDirection, DimensionStrIdRef, GlobalTilePos};

use ::being_shared::*;
use ac_input::ac_input_actions::*;
use player::player_components::{Mine, Player};

use crate::movement_components::{GridLockedMovement, InputMoveDir, MoveVecMag};
use crate::movement_helpers::normalize_to_axis_dir;

pub const INPUT_DEADZONE: f32 = 0.2;

pub fn add_movement_components_to_beings(
    mut commands: Commands,
    beings: Query<Entity, Added<Being>>,
) {
    for being_ent in beings.iter() {
        commands.entity(being_ent).try_insert_if_new((
            InputMoveDir::default(),
            MoveVecMag::default(),
            Replicated,
            MoveAnimActive::default(),
            Grounding::default(),
            Visibility::default(),
            CardinalDirection::default(),
            AppliedModifiers::default(),
            Prefix::trunc("Being"),
            DimensionStrIdRef::overworld_fallback(),
            AssetScoped,
            GlobalTilePos::default(),
            GridLockedMovement::default(),
        ));
    }
}

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
    player_query: Query<Entity, (With<Mine>, With<Player>, Without<Actions<BeingInputContext>>)>,
) {
    for player_ent in player_query.iter() {
        commands.entity(player_ent).try_insert((
            BeingInputContext,
            actions!(BeingInputContext[
                (
                    Action::<BeingMoveAction>::new(),
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

pub fn copy_player_move_input_to_beings(
    move_action_query: Query<&Action<BeingMoveAction>>,
    player_query: Query<(&Actions<BeingInputContext>, &ComputedBeings), (With<Mine>, With<Player>)>,
    mut beings: Query<(&ComputedBy, &mut InputMoveDir)>,
) {
    let mut found_player = false;
    for (actions, computed_beings) in player_query.iter() {
        found_player = true;
        let Some(move_action) = move_action_query.iter_many(actions).next() else {
            error!(
                target: MOVEMENT_SYSTEM,
                "copy_player_move_input_to_beings: Mine+Player entity missing linked Action<BeingMoveAction>"
            );
            continue;
        };
        let vec = if move_action.length() <= INPUT_DEADZONE {
            Vec2::ZERO
        } else {
            normalize_to_axis_dir(move_action.normalize()).as_vec2()
        };
        for &being_ent in computed_beings.being_ents() {
            let Ok((computed_by, mut input_move_dir)) = beings.get_mut(being_ent) else {
                error!(
                    target: MOVEMENT_SYSTEM,
                    "copy_player_move_input_to_beings: computed being {:?} missing InputMoveDir/ComputedBy",
                    being_ent
                );
                continue;
            };
            if !computed_by.human_input {
                continue;
            }
            if input_move_dir.0 != vec {
                input_move_dir.0 = vec;
            }
        }
    }
    if !found_player {
        error!(
            target: MOVEMENT_SYSTEM,
            "copy_player_move_input_to_beings: no Mine+Player entity with Actions<BeingInputContext> found"
        );
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
