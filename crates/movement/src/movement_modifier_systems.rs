use core::f32;

use being_shared::{Body, BodypartChildrenBodyparts, BodyTreeWeightSum, ComputedLocally};
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::ClientState;
use game_common::game_common_components::{EntityZero, EntityZeroRef};

use modifier_shared::{modifier_has_marker, resolve_modifier_component};
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::{InvertMovement, WalkSpeed};
use tilemap_shared::*;

use crate::movement_components::{InputMaxSpeed, InputMoveDir, InputSpeedThrottleMult, FinalNormMoveDir, SpeedMagnitude};

pub fn process_input_direction_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &AppliedModifiers,
        Option<&Body>,
        &InputMoveDir,
        &mut FinalNormMoveDir,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<(Entity, &ModifierTarget, Option<&EntityZeroRef>, ), (Without<EntityZero>, )>,
    curr_values_query: Query<&CurrEffectiveValue, >,
    apply_modes_query: Query<&ApplyMode, >,
    invert_markers_query: Query<(), With<InvertMovement>>,
    bodyparts_query: Query<&BodypartChildrenBodyparts, >,
    children_query: Query<&Children, >,
    ezero_refs_query: Query<&EntityZeroRef>,
    mut effects: Local<EntityHashSet>,

) {
    let is_client = state.get() == &ClientState::Connected;
    for (being_ent, applied, body, input_move_dir, mut norm_move_dir, controlled_locally) in being_query.iter_mut()
    {
        if is_client && !controlled_locally {
            continue;
        }
        let input_dir = input_move_dir.0;

        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;
        effects.clear();
        applied.iter().for_each(|ent| {
            effects.insert(ent);
        });
        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }
        let Some(body) = body else {
            continue;
        };
        let Ok(bodyparts) = bodyparts_query.get(body.entity()) else {
            continue;
        };
        for bodypart_ent in bodyparts.iter() {
            let Ok(part_ezero_ref) = ezero_refs_query.get(bodypart_ent) else {
                continue;
            };
            let Ok(children) = children_query.get(part_ezero_ref.0) else {
                continue;
            };
            for child_ent in children.iter() {
                effects.insert(child_ent);
            }
        }
        for effect in effects.iter() {
            let Ok((modifier_ent, _, ezero_ref, )) = modifiers_query.get(*effect)
            else {
                continue;
            };
            if !modifier_has_marker::<InvertMovement>(modifier_ent, ezero_ref, &invert_markers_query) {
                continue;
            }
            let Some(curr_value) = resolve_modifier_component(modifier_ent, ezero_ref, &curr_values_query) else {
                continue;
            };
            let Some(optype) = resolve_modifier_component(modifier_ent, ezero_ref, &apply_modes_query) else {
                continue;
            };
            let val = curr_value.0;
            match optype {
                ApplyMode::Add => invert_sum += val,
                ApplyMode::Mul => invert_scale *= val.max(0.0),
                _ => {}
            }
        }
        norm_move_dir.0 = if input_dir == Vec2::ZERO {
            Vec2::ZERO
        } else if invert_sum * invert_scale > 1.0 {
            -input_dir.normalize()
        } else {
            input_dir.normalize()
        };
    }
}

pub fn process_speed_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &DimensionRef,
        &GlobalTilePos,
        &AppliedModifiers,
        Option<&Body>,
        &mut SpeedMagnitude,
        Option<&BodyTreeWeightSum>,
        Option<&InputSpeedThrottleMult>,
        Option<&InputMaxSpeed>,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<(Entity, &ModifierTarget, Option<&EntityZeroRef>, ), (Without<EntityZero>, )>,
    curr_values_query: Query<&CurrEffectiveValue>,
    apply_modes_query: Query<&ApplyMode, >,
    walk_markers_query: Query<(), With<WalkSpeed>>,
    mitigating_only_markers_query: Query<(), With<MitigatingOnly>>,
    bodyparts_query: Query<&BodypartChildrenBodyparts, >,
    children_query: Query<&Children, >,
    ezero_refs_query: Query<&EntityZeroRef>,
    tile_walk_speed_mults: Query<&WalkSpeedMultIfOnTop>,
    mut tile_gathering: TileGatheringParamSet,
) {
    for (
        being_ent,
        &dim_ref,
        tile_pos,
        applied,
        body,
        mut speed_magnitude,
        body_weight_sum,
        input_speed_throttle_mult,
        input_max_speed,
        controlled_locally,
    ) in being_query.iter_mut()
    {
        let is_client = state.get() == &ClientState::Connected;
        if is_client && !controlled_locally {
            continue;
        }
        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;
        let mut speed_scale: f32 = 1.0;
        let mut speed_substractors_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0;
        let mut speed_sum: f32 = 0.0;

        let mut effects = EntityHashSet::default();
        applied.iter().for_each(|ent| {
            effects.insert(ent);
        });
        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }
        let Some(body) = body else {
            continue;
        };
        let Ok(bodyparts) = bodyparts_query.get(body.entity()) else {
            continue;
        };
        for bodypart_ent in bodyparts.iter() {
            let Ok(part_ezero_ref) = ezero_refs_query.get(bodypart_ent) else {
                continue;
            };
            let Ok(children) = children_query.get(part_ezero_ref.0) else {
                continue;
            };
            for child_ent in children.iter() {
                effects.insert(child_ent);
            }
        }
        for effect in effects.iter() {
            let Ok((modifier_ent, _, ezero_ref, )) =
                modifiers_query.get(*effect)
            else {
                continue;
            };
            if !modifier_has_marker::<WalkSpeed>(modifier_ent, ezero_ref, &walk_markers_query) {
                continue;
            }
            let Some(curr_value) = resolve_modifier_component(modifier_ent, ezero_ref, &curr_values_query) else {
                continue;
            };
            let Some(optype) = resolve_modifier_component(modifier_ent, ezero_ref, &apply_modes_query) else {
                continue;
            };
            let mitigating = modifier_has_marker::<MitigatingOnly>(
                modifier_ent,
                ezero_ref,
                &mitigating_only_markers_query,
            );
            let val = curr_value.0;
            match optype {
                ApplyMode::Add => {
                    if val > 0.0 {
                        if mitigating {
                            slowdown_mitigators_sum += val;
                        } else {
                            speed_sum += val;
                        }
                    } else {
                        speed_substractors_sum += val;
                    }
                }
                ApplyMode::Mul => speed_scale *= val.max(0.0),
                ApplyMode::Min => speed_min = speed_min.max(val),
                ApplyMode::Max => speed_max = speed_max.min(val).max(0.0),
            }
        }
        speed_sum += speed_substractors_sum + slowdown_mitigators_sum;
        let total_weight_newtons = body_weight_sum
            .map(|sum| sum.0)
            .unwrap_or_default()
            .max(1.0);
        let mut final_speed = (speed_sum * speed_scale)
            .max(speed_min)
            .min(speed_max)
            .max(0.0);
        final_speed /= total_weight_newtons;

        let mut tile_walk_mult: f32 = 1.0;
        let tile_ents = tile_gathering.gather_tiles_at(dim_ref, *tile_pos).to_vec();
        for tile_ent in tile_ents {
            let Ok(tile_cfg_ref) = ezero_refs_query.get(tile_ent) else {
                continue;
            };
            let Ok(tile_walk_mult_cfg) = tile_walk_speed_mults.get(tile_cfg_ref.0) else {
                continue;
            };
            tile_walk_mult = tile_walk_mult.min(tile_walk_mult_cfg.0);
        }
        let final_speed = final_speed * tile_walk_mult.max(0.0);
        let mut final_speed = final_speed * 5000.;
        let speed_throttle = input_speed_throttle_mult.map(|v| v.0).unwrap_or(1.0).clamp(0.0, 1.0);
        final_speed *= speed_throttle;
        if let Some(input_max_speed) = input_max_speed {
            final_speed = final_speed.min(input_max_speed.0.max(0.0));
        }
        if (speed_magnitude.0 - final_speed).abs() > f32::EPSILON {
            speed_magnitude.0 = final_speed;
        }
    }
}
