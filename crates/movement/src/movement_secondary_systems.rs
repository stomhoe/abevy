use bevy::ecs::{entity::{EntityHashMap, EntityHashSet}, entity_disabling::Disabled};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Action, Actions};
use bevy_replicon::prelude::Replicated;
use ::common::*;
use modifier_shared::modifier_components::AppliedModifiers;
use ::sprite_animation_shared::*;
use ::tilemap_shared::*;

use ::being_shared::*;
use ac_input::ac_input_actions::*;
use player_shared::player_components::{Mine, Player};

pub const INPUT_DEADZONE: f32 = 0.2;

#[allow(unused_parens, )]
pub fn add_grid_locked_movement_requirements(
    mut commands: Commands,
    query: Query<(Entity, Has<Being>, Has<Unloaded>), With<GridLockedMovement>>,
    added_grid_locked_movements: Query<Entity, Added<GridLockedMovement>>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut removed_unloaded: RemovedComponents<Unloaded>,
    mut to_process: Local<EntityHashSet>,
) {
    let mut loaded_move_anims = 0usize;
    let mut total = 0usize;
    let added_grid_locked_movements = added_grid_locked_movements.iter();
    to_process.reserve(
        removed_disabled.len()
            + removed_unloaded.len()
            + added_grid_locked_movements
                .size_hint()
                .1
                .unwrap_or(added_grid_locked_movements.size_hint().0),
    );
    to_process.extend(removed_disabled.read());
    to_process.extend(removed_unloaded.read());
    to_process.extend(added_grid_locked_movements);

    for being_ent in to_process.drain() {
        let Ok((_, has_being, has_unloaded)) = query.get(being_ent) else {
            continue;
        };
        total += 1;
        if has_being && !has_unloaded {
            loaded_move_anims += 1;
            commands.entity(being_ent).try_insert_if_new((
                GridLockedMovementRequirementsBundle::default(),
                MoveVisualsBundle::default(),
            ));
        } else {
            commands
                .entity(being_ent)
                .try_insert_if_new(GridLockedMovementRequirementsBundle::default());
        }
    }
    if total > 0 {
        debug!(
            target: MOVEMENT_SYSTEM,
            "Backfilled GridLockedMovement requirements for {} entities (MoveAnimActive on {} loaded beings)",
            total,
            loaded_move_anims
        );
    }
}

pub fn add_movement_components_to_beings(
    mut commands: Commands,
    added_beings: Query<Entity, Added<Being>>,
    beings: Query<(), (With<Being>, Without<Disabled>, Without<Unloaded>)>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut beings_to_update: Local<EntityHashSet>,
) {
    let added_iter = added_beings.iter();
    beings_to_update.reserve(removed_disabled.len() + added_iter.size_hint().1.unwrap_or(added_iter.size_hint().0));
    beings_to_update.extend(removed_disabled.read());
    beings_to_update.extend(added_iter);
    let mut rng = rand::rng();

    for being_ent in beings_to_update.drain() {
        if beings.get(being_ent).is_err() {
            continue;
        }
        commands.entity(being_ent).try_insert_if_new((
            Replicated,
            Grounding::default(),
            Visibility::default(),
            CardinalDirection::random(&mut rng),
            AppliedModifiers::default(),
            InputMaxSpeed::default(),
            InputSpeedThrottleMult::default(),
            AssetScoped,
            GridLockedMovement::default(),
        ));
    }
}

#[allow(unused_parens, )]
pub fn update_facing_dir(
    mut query: Query<(Entity, &FinalNormMoveDir, Option<&GridLockedMovement>, &mut CardinalDirection), (With<ComputedLocally>)>,
    mut messages: Local<Vec<MirrorHolderStateForSprite>>,
    mut writer: MessageWriter<MirrorHolderStateForSprite>,
) {
    for (being_ent, norm_move_dir, glm, mut facing_dir) in query.iter_mut() {
        let dir = glm
            .filter(|glm| glm.is_stepping())
            .map(|glm| glm.step_dir)
            .unwrap_or_else(|| norm_move_dir.normalize_to_axis_dir());
        let next = if dir == IVec2::ZERO {
            *facing_dir
        } else {
            CardinalDirection::from_dir_vec(dir)
        };
        if *facing_dir != next {
            *facing_dir = next;
            messages.push(MirrorHolderStateForSprite(being_ent));
        }
    }
    writer.write_batch(messages.drain(..));
}
#[allow(unused_parens, )]
pub fn copy_client_move_input_to_controlled_beings(
    player_query: Query<(&Actions<BeingDirectControlInputContext>, &ComputedBeings), (With<Mine>, With<Player>)>,
    move_action_query: Query<&Action<DcWasdAction>>,
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
                "copy_client_move_input_to_controlled_beings: Mine+Player entity missing linked Action<BeingMoveAction>"
            );
            continue;
        };
        let vec = if move_action.length() <= INPUT_DEADZONE {
            Vec2::ZERO
        } else {
            FinalNormMoveDir(move_action.normalize()).normalize_to_axis_dir().as_vec2()
        };
        for &being_ent in computed_beings.being_ents() {
            let Ok((computed_by, mut input_move_dir)) = beings.get_mut(being_ent) else {
                continue;
            };
            if !computed_by.human_dc_input {
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
            "copy_client_move_input_to_controlled_beings: no Mine+Player entity with Actions<BeingInputContext> found"
        );
    }
}
#[allow(unused_parens, )]
pub fn emit_move_state_on_movevecmag_speed_mag_change(
    query: Query<(Entity, &SpeedMagnitude), (Changed<SpeedMagnitude>, )>,
    mut prev_by_ent: Local<EntityHashMap<SpeedMagnitude>>,
    mut messages: Local<Vec<MirrorHolderStateForSprite>>,
    mut writer: MessageWriter<MirrorHolderStateForSprite>,
) {
    for (ent, &speed_magnitude) in query.iter() {
        let Some(&prev) = prev_by_ent.get(&ent) else {
            prev_by_ent.insert(ent, speed_magnitude);
            continue;
        };
        if prev != speed_magnitude {
            messages.push(MirrorHolderStateForSprite(ent));
            prev_by_ent.insert(ent, speed_magnitude);
        }
    }
    writer.write_batch(messages.drain(..));
}
