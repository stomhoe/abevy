use core::f32;

use being_shared::{BodyTreeWeightSum, ComputedLocally};
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::ClientState;
use game_common::game_common_components::EntityZeroRef;

use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::{InvertMovement, WalkSpeed};
use tilemap_shared::*;

use crate::movement_components::{InputMoveDir, NormMoveDir, SpeedMagnitude};

pub fn process_input_direction_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &AppliedModifiers,
        &InputMoveDir,
        &mut NormMoveDir,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<(
        Entity,
        &ModifierTarget,
        &CurrEffectiveValue,
        &ApplyMode,
        Has<InvertMovement>,
    )>,
) {
    let is_client = state.get() == &ClientState::Connected;
    for (being_ent, applied, input_move_dir, mut norm_move_dir, controlled_locally) in being_query.iter_mut()
    {
        if is_client && !controlled_locally {
            continue;
        }
        let input_dir = input_move_dir.0;

        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;
        let mut effects = EntityHashSet::default();
        applied.entities().iter().for_each(|&ent| {
            effects.insert(ent);
        });
        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }
        for effect in effects.iter() {
            let Ok((_, _, &CurrEffectiveValue(val), optype, invert)) = modifiers_query.get(*effect)
            else {
                continue;
            };
            match optype {
                ApplyMode::Add if invert => invert_sum += val,
                ApplyMode::Mul if invert => invert_scale *= val.max(0.0),
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
        &mut SpeedMagnitude,
        Option<&BodyTreeWeightSum>,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<
        (
            Entity,
            &ModifierTarget,
            &CurrEffectiveValue,
            &ApplyMode,
            Has<MitigatingOnly>,
        ),
        With<WalkSpeed>,
    >,
    tile_entity_zero_refs: Query<&EntityZeroRef>,
    tile_walk_speed_mults: Query<&WalkSpeedMultIfOnTop>,
    mut tile_gathering: TileGatheringParamSet,
) {
    for (
        being_ent,
        &dim_ref,
        tile_pos,
        applied,
        mut speed_magnitude,
        body_weight_sum,
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
        applied.entities().iter().for_each(|&ent| {
            effects.insert(ent);
        });
        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }
        for effect in effects.iter() {
            let Ok((_, _, &CurrEffectiveValue(val), optype, mitigating)) =
                modifiers_query.get(*effect)
            else {
                continue;
            };
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
        for tile_ent in tile_gathering.gather_tiles_at_to_drain(dim_ref, *tile_pos) {
            let Ok(tile_cfg_ref) = tile_entity_zero_refs.get(*tile_ent) else {
                continue;
            };
            let Ok(tile_walk_mult_cfg) = tile_walk_speed_mults.get(tile_cfg_ref.0) else {
                continue;
            };
            tile_walk_mult = tile_walk_mult.min(tile_walk_mult_cfg.0);
        }
        let final_speed = final_speed * tile_walk_mult.max(0.0);
        let final_speed = final_speed * 5000.;
        if (speed_magnitude.0 - final_speed).abs() > f32::EPSILON {
            speed_magnitude.0 = final_speed;
        }
    }
}
