use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Action, Actions};
use bevy_replicon::prelude::Replicated;
use common::log_targets::MOVEMENT_SYSTEM;
use common::prelude::*;
use modifier_shared::modifier_components::AppliedModifiers;
use ::sprite_animation_shared::*;
use ::tilemap_shared::*;

use ::being_shared::*;
use ac_input::ac_input_actions::*;
use player::player_components::{Mine, Player};

use crate::movement_components::*;

pub const INPUT_DEADZONE: f32 = 0.2;

pub fn add_movement_components_to_beings(
    mut commands: Commands,
    beings: Query<Entity, Added<Being>>,
) {
    for being_ent in beings.iter() {
        commands.entity(being_ent).try_insert_if_new((
            InputMoveDir::default(),
            NormMoveDir::default(),
            SpeedMagnitude::default(),
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
    mut query: Query<(Entity, &NormMoveDir, Option<&GridLockedMovement>, &mut CardinalDirection), (With<ComputedLocally>)>,
    mut writer: MessageWriter<MatchHeldSpritesAnimStateToBeingState>,
    mut messages: Local<HashSet<MatchHeldSpritesAnimStateToBeingState>>,
) {
    for (being_ent, norm_move_dir, glm, mut facing_dir) in query.iter_mut() {
        let dir = glm
            .and_then(|glm| {
                if glm.step_dir == IVec2::ZERO {
                    None
                } else {
                    Some(glm.step_dir)
                }
            })
            .unwrap_or_else(|| InputMoveDir(norm_move_dir.0).normalize_to_axis_dir());
        let next = if dir == IVec2::ZERO {
            *facing_dir
        } else {
            CardinalDirection::from_dir_vec(dir)
        };
        if *facing_dir != next {
            *facing_dir = next;
            trace!(target: MOVEMENT_SYSTEM, "Facing updated for {:?} to {:?}", being_ent, next);
            messages.insert(MatchHeldSpritesAnimStateToBeingState(being_ent));
        }
    }
    writer.write_batch(messages.drain());
}
#[allow(unused_parens, )]
pub fn copy_player_move_input_to_beings(
    move_action_query: Query<&Action<BeingWasdAction>>,
    player_query: Query<(&Actions<BeingDirectControlInputContext>, &ComputedBeings), (With<Mine>, With<Player>)>,
    mut beings: Query<(&ComputedBy, &mut InputMoveDir), (LocalHumanControlled)>,
) {
    let mut found_player = false;
    if beings.is_empty(){
        return;
    }
    for (actions, computed_beings) in player_query.iter() {
        found_player = true;
        let Some(move_action) = move_action_query.iter_many(actions).next() else {
            error_once!(
                target: MOVEMENT_SYSTEM,
                "copy_player_move_input_to_beings: Mine+Player entity missing linked Action<BeingMoveAction>"
            );
            continue;
        };
        let vec = if move_action.length() <= INPUT_DEADZONE {
            Vec2::ZERO
        } else {
            InputMoveDir(move_action.normalize()).normalize_to_axis_dir().as_vec2()
        };
        for &being_ent in computed_beings.being_ents() {
            let Ok((computed_by, mut input_move_dir)) = beings.get_mut(being_ent) else {
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

/// Emits a `UpdateSpriteAnimState` message when the speed magnitude changes.
pub fn emit_move_state_on_movevecmag_speed_mag_change(
    query: Query<(Entity, &SpeedMagnitude)>,
    mut writer: MessageWriter<MatchHeldSpritesAnimStateToBeingState>,
    mut prev_by_ent: Local<EntityHashMap<SpeedMagnitude>>,
    mut messages: Local<Vec<MatchHeldSpritesAnimStateToBeingState>>,
) {
    for (ent, &speed_magnitude) in query.iter() {
        let Some(&prev) = prev_by_ent.get(&ent) else {
            prev_by_ent.insert(ent, speed_magnitude);
            continue;
        };
        if prev != speed_magnitude {
            messages.push(MatchHeldSpritesAnimStateToBeingState(ent));
            prev_by_ent.insert(ent, speed_magnitude);
        }
    }
    writer.write_batch(messages.drain(..));
}
