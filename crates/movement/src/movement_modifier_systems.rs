use core::f32;

use being_shared::{HeldBody, BodypartChildrenBodyparts, BodyWeightSum, ComputedLocally};
use being_shared::body_energy::BodyStrengthScale;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::ClientState;
use game_common::game_common_components::{Templ, TemplEntiRef};

use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker};
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::{InvertMovement, WalkStrength};
use tilemap_shared::*;

use being_shared::movement_shared_components::*;

#[allow(unused_parens, )]
pub fn process_input_direction_modifiers(
    mut being_query: Query<(
        Entity,
        Option<&HeldBody>,
        &mut InputInvMul,
    )>,
    applied_mods_query: Query<&AppliedModifiers, >,
    modifiers_query: Query<(Entity, Option<&TemplEntiRef>, ), (Without<Templ>, )>,
    curr_values_query: Query<&CurrEffectiveValue, >,
    apply_modes_query: Query<&ApplyMode, >,
    invert_markers_query: Query<(), With<InvertMovement>>,
    bodyparts_query: Query<&BodypartChildrenBodyparts, >,
    templ_refs_query: Query<&TemplEntiRef>,
    mut effects: Local<EntityHashSet>,
) {
    for (being_ent, body, mut input_vec_modi_mul, ) in being_query.iter_mut()
    {
        if input_vec_modi_mul.is_non_default() {
            input_vec_modi_mul.0 = 1.0;
        }
        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;
        effects.clear();
        let being_templ_ref = templ_refs_query.get(being_ent).ok();
        collect_applied_modifier_entities(&mut effects, being_ent, being_templ_ref, &applied_mods_query);
        let Some(body) = body else {
            continue;
        };
        let body_ent = body.entity();
        let body_templ_ref = templ_refs_query.get(body_ent).ok();
        collect_applied_modifier_entities(&mut effects, body_ent, body_templ_ref, &applied_mods_query);
        let Ok(bodyparts) = bodyparts_query.get(body_ent) else {
            continue;
        };
        for bodypart_ent in bodyparts.iter() {
            let part_templ_ref = templ_refs_query.get(bodypart_ent).ok();
            collect_applied_modifier_entities(&mut effects, bodypart_ent, part_templ_ref, &applied_mods_query);
        }
        for effect in effects.iter() {
            let Ok((modifier_ent, templ_ref, )) = modifiers_query.get(*effect)
            else {
                continue;
            };
            if !modifier_has_marker::<InvertMovement>(modifier_ent, templ_ref, &invert_markers_query) {
                continue;
            }
            let Ok(curr_value) = curr_values_query.get(modifier_ent).or_else(|_| {
                let Some(templ_ref) = templ_ref else {
                    return Err(());
                };
                curr_values_query.get(templ_ref.0).map_err(|_| ())
            }) else {
                continue;
            };
            let Ok(optype) = apply_modes_query.get(modifier_ent).or_else(|_| {
                let Some(templ_ref) = templ_ref else {
                    return Err(());
                };
                apply_modes_query.get(templ_ref.0).map_err(|_| ())
            }) else {
                continue;
            };
            let val = curr_value.0;
            match optype {
                ApplyMode::Add => invert_sum += val,
                ApplyMode::Mul => invert_scale *= val.max(0.0),
                _ => {}
            }
        }
        if invert_sum * invert_scale > 1.0 {
            input_vec_modi_mul.0 = -1.0;
        }
    }
}

#[allow(unused_parens, )]
pub fn apply_input_vec_modi_mul_to_final_norm_move_dir(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        &InputMoveDir,
        &InputInvMul,
        &mut FinalNormMoveDir,
        Has<ComputedLocally>,
    )>,
) {
    let is_client = state.get() == &ClientState::Connected;
    for (input_move_dir, input_vec_modi_mul, mut final_norm_move_dir, controlled_locally) in being_query.iter_mut() {
        if is_client && !controlled_locally {
            continue;
        }
        let new_val = input_move_dir.0 * input_vec_modi_mul.0;
        if final_norm_move_dir.new_val_is_different(new_val) {
            final_norm_move_dir.0 = new_val;
        }
    }
}

#[allow(unused_parens, )]
pub fn process_speed_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &DimensionRef,
        &GlobalTilePos,
        Option<&HeldBody>,
        &mut SpeedMagnitude,
        Option<&BodyWeightSum>,
        Option<&BodyStrengthScale>,
        Option<&InputSpeedThrottleMult>,
        Option<&InputMaxSpeed>,
        Has<ComputedLocally>,
    )>,
    applied_mods_query: Query<&AppliedModifiers, >,
    modifiers_query: Query<(Entity, Option<&TemplEntiRef>, ), (Without<Templ>, )>,
    curr_values_query: Query<&CurrEffectiveValue>,
    apply_modes_query: Query<&ApplyMode, >,
    walk_markers_query: Query<(), With<WalkStrength>>,
    mitigating_only_markers_query: Query<(), With<MitigatingOnly>>,
    bodyparts_query: Query<&BodypartChildrenBodyparts, >,
    templ_refs_query: Query<&TemplEntiRef>,
    tile_walk_speed_mults: Query<&WalkSpeedMultIfOnTop>,
    mut effects: Local<EntityHashSet>,
    mut tile_gathering: TileGatheringParamSet,
) {
    let is_client = state.get() == &ClientState::Connected;
    for (
        being_ent,
        &dim_ref,
        tile_pos,
        body,
        mut speed_magnitude,
        body_weight_sum,
        body_strength_scale,
        input_speed_throttle_mult,
        input_max_speed,
        controlled_locally,
    ) in being_query.iter_mut()
    {
        if is_client && !controlled_locally {
            continue;
        }
        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;
        let mut speed_scale: f32 = 1.0;
        let mut speed_substractors_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0;
        let mut speed_sum: f32 = 0.0;

        effects.clear();
        let being_templ_ref = templ_refs_query.get(being_ent).ok();
        collect_applied_modifier_entities(&mut effects, being_ent, being_templ_ref, &applied_mods_query);
        let Some(body) = body else {
            continue;
        };
        let body_ent = body.entity();
        let body_templ_ref = templ_refs_query.get(body_ent).ok();
        collect_applied_modifier_entities(&mut effects, body_ent, body_templ_ref, &applied_mods_query);
        let Ok(bodyparts) = bodyparts_query.get(body_ent) else {
            continue;
        };
        for bodypart_ent in bodyparts.iter() {
            let part_templ_ref = templ_refs_query.get(bodypart_ent).ok();
            collect_applied_modifier_entities(&mut effects, bodypart_ent, part_templ_ref, &applied_mods_query);
        }
        for effect in effects.iter() {
            let Ok((modifier_ent, templ_ref, )) =
                modifiers_query.get(*effect)
            else {
                continue;
            };
            if !modifier_has_marker::<WalkStrength>(modifier_ent, templ_ref, &walk_markers_query) {
                continue;
            }
            let Ok(curr_value) = curr_values_query.get(modifier_ent).or_else(|_| {
                let Some(templ_ref) = templ_ref else {
                    return Err(());
                };
                curr_values_query.get(templ_ref.0).map_err(|_| ())
            }) else {
                continue;
            };
            let Ok(optype) = apply_modes_query.get(modifier_ent).or_else(|_| {
                let Some(templ_ref) = templ_ref else {
                    return Err(());
                };
                apply_modes_query.get(templ_ref.0).map_err(|_| ())
            }) else {
                continue;
            };
            let mitigating = modifier_has_marker::<MitigatingOnly>(
                modifier_ent,
                templ_ref,
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
            .map(|sum| sum.0.max(1.0))
            .unwrap_or(1.0);
        let mut final_speed = (speed_sum * speed_scale)
            .max(speed_min)
            .min(speed_max)
            .max(0.0);
        final_speed /= total_weight_newtons;
        if let Some(body_strength_scale) = body_strength_scale {
            final_speed *= body_strength_scale.0.max(0.0);
        }

        let mut tile_walk_mult: f32 = 1.0;
        let tile_ents = tile_gathering.gather_tiles(dim_ref, *tile_pos);
        for &tile_ent in tile_ents {
            let Ok(tile_cfg_ref) = templ_refs_query.get(tile_ent) else {
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
