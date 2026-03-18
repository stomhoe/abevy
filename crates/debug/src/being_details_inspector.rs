use being::body::{Bodies, BodyPartDamage, BodyParts, BodySums};
use being_shared::{ComputedBy, ComputedLocally};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Action, Actions};
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;
use ac_input::ac_input_actions::*;
use common::common_components::{DisplayName, StrId};
use item_shared::{item_components::{HeldItems, SlottedItemHolder}, ItemOperation};
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::*;
use movement::movement_components::*;
use player::player_components::{Mine, Player};
use tilemap_shared::CardinalDirection;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

fn part_label(entity: Entity, display_name: Option<&DisplayName>, str_id: Option<&StrId>) -> String {
    if let Some(display_name) = display_name {
        if !display_name.0.is_empty() {
            return format!("{} ({:?})", display_name.0, entity);
        }
    }
    if let Some(str_id) = str_id {
        if !str_id.as_str().is_empty() {
            return format!("{} ({:?})", str_id, entity);
        }
    }
    format!("{:?}", entity)
}

#[allow(unused_parens)]
pub fn being_details_inspector(world: &mut World) {
    let Some(window_visible) = world.get_resource::<DubugWindowsVisibility>() else {
        return;
    };
    if !window_visible.being_details {
        return;
    }
    let _ = window_visible;

    let Some(selected_entities) = world.get_resource::<DebugSelectedEntities>() else {
        return;
    };
    let Some(selected_being_entity) = selected_entities.selected_being else {
        return;
    };
    let mut selected_part = selected_entities.selected_being_bodypart;
    let mut show_full_components = selected_entities.show_full_being_components;
    let _ = selected_entities;

    let mut egui_context_query = world.query_filtered::<
        &bevy_inspector_egui::bevy_egui::EguiContext,
        With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>,
    >();
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let mut body_query = world.query::<&Bodies>();
    let mut body_sums_query = world.query::<&BodySums>();
    let mut display_name_query = world.query::<&DisplayName>();
    let mut str_id_query = world.query::<&StrId>();
    let mut body_parts_query = world.query::<&BodyParts>();
    let mut body_part_damage_query = world.query::<&BodyPartDamage>();
    let mut held_items_query = world.query::<&HeldItems>();
    let mut slot_holder_query = world.query::<&SlottedItemHolder>();
    let mut norm_move_dir_query = world.query::<&FinalNormMoveDir>();
    let mut speed_magnitude_query = world.query::<&SpeedMagnitude>();
    let mut input_move_dir_query = world.query::<&InputMoveDir>();
    let mut gtrans_query = world.query::<&GlobalTransform>();
    let mut computed_by_query = world.query::<&ComputedBy>();
    let mut computed_locally_query = world.query::<&ComputedLocally>();
    let mut player_actions_query =
        world.query_filtered::<&Actions<BeingDirectControlInputContext>, (With<Mine>, With<Player>)>();
    let mut player_move_action_query = world.query::<&Action<DcWasdAction>>();
    let mut grid_move_query = world.query::<&GridLockedMovement>();
    let mut gpos_query = world.query::<&tilemap_shared::GlobalTilePos>();
    let mut facing_query = world.query::<&CardinalDirection>();
    let mut modifiers_query = world.query::<(
        &ModifierTarget,
        Option<&BaseValue>,
        Option<&CurrEffectiveValue>,
        Has<HitpointsCapacity>,
        Has<HitpointRegenRate>,
        Has<BloodCapacity>,
        Has<BleedRate>,
        Has<PainSensitivity>,
        Has<Vision>,
        Has<ManipulationDexterity>,
        Has<ManipulationStrength>,
    )>();
    let mut walk_modifiers_query = world.query_filtered::<(
        &ModifierTarget,
        Option<&CurrEffectiveValue>,
        &ApplyMode,
        Has<modifier_shared::modifier_components::MitigatingOnly>,
    ), With<WalkSpeed>>();

    let mut body_infos = Vec::new();
    let mut part_infos: Vec<(Entity, String)> = Vec::new();
    let Ok(bodies) = body_query.get(world, selected_being_entity) else {
        return;
    };
    let mut inventory_holders = vec![(
        selected_being_entity,
        part_label(
        selected_being_entity,
            display_name_query.get(world, selected_being_entity).ok(),
            str_id_query.get(world, selected_being_entity).ok(),
        ),
    )];
    for body_entity in bodies.iter() {
        let label = part_label(
            body_entity,
            display_name_query.get(world, body_entity).ok(),
            str_id_query.get(world, body_entity).ok(),
        );
        inventory_holders.push((body_entity, label.clone()));
        if let Ok(sums) = body_sums_query.get(world, body_entity) {
            body_infos.push((body_entity, label, sums.clone()));
        }
        let Ok(parts) = body_parts_query.get(world, body_entity) else {
            continue;
        };
        for part_entity in parts.iter() {
            let label = part_label(
                part_entity,
                display_name_query.get(world, part_entity).ok(),
                str_id_query.get(world, part_entity).ok(),
            );
            inventory_holders.push((part_entity, label.clone()));
            part_infos.push((part_entity, label));
        }
    }
    if selected_part.is_none() {
        selected_part = part_infos.first().map(|(entity, _)| *entity);
    } else if !part_infos
        .iter()
        .any(|(entity, _)| Some(*entity) == selected_part)
    {
        selected_part = part_infos.first().map(|(entity, _)| *entity);
    }

    let mut clear_selection = false;
    let mut is_open = true;
    let world_ptr = world as *mut World;

    egui::Window::new("Selected Being Details")
        .default_width(700.0)
        .default_height(560.0)
        .default_pos([screen_rect.right() - 720.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.heading(format!("Being Entity: {:?}", selected_being_entity));
            ui.horizontal(|ui| {
                if ui.button("Show Full Components").clicked() {
                    show_full_components = !show_full_components;
                }
                if ui.button("Clear Selection").clicked() {
                    clear_selection = true;
                }
            });
            ui.separator();

            if show_full_components {
                ui.label("All Components on this Being:");
                ui.separator();
                unsafe {
                    bevy_inspector::ui_for_entity(&mut *world_ptr, selected_being_entity, ui);
                }
                return;
            }

            ui.heading("Body Sums");
            for (body_entity, body_label, sums) in &body_infos {
                ui.collapsing(format!("{} [{:?}]", body_label, body_entity), |ui| {
                    ui.label(format!("HP: {:.2}/{:.2}", sums.current_hp, sums.total_hp));
                    ui.label(format!(
                        "Blood: {:.2}/{:.2}",
                        sums.blood, sums.blood_capacity
                    ));
                    ui.label(format!("Bleed rate: {:.2}", sums.bleed_rate));
                    ui.label(format!("Consciousness: {:.2}", sums.consciousness));
                    ui.label(format!("Pain: {:.2}", sums.pain));
                    ui.label(format!("Vision: {:.2}", sums.vision));
                    ui.label(format!("Manip dex: {:.2}", sums.manip_dex));
                    ui.label(format!("Manip str: {:.2}", sums.manip_str));
                });
            }
            ui.separator();

            ui.collapsing("Movement Details", |ui| {
                let mut is_human_input = false;
                if let Ok(computed_by) = computed_by_query.get(world, selected_being_entity) {
                    is_human_input = computed_by.human_dc_input;
                    ui.label(format!(
                        "ComputedBy: client_ent={:?}, human_input={}",
                        computed_by.client_ent, computed_by.human_dc_input
                    ));
                }
                if let Ok(computed_locally) = computed_locally_query.get(world, selected_being_entity) {
                    ui.label(format!("ComputedLocally: {:?}", computed_locally));
                }
                if is_human_input {
                    if let Some(player_actions) = player_actions_query.iter(world).next() {
                        if let Some(player_move_action) =
                            player_move_action_query.iter_many(world, player_actions).next()
                        {
                            ui.label(format!(
                                "Player BeingMoveAction: [{:.2}, {:.2}]",
                                player_move_action.x, player_move_action.y
                            ));
                        } else {
                            ui.label("Player BeingMoveAction: missing");
                        }
                    } else {
                        ui.label("Player BeingMoveAction: missing context");
                    }
                }
                if let Ok(norm_move_dir) = norm_move_dir_query.get(world, selected_being_entity) {
                    ui.label(format!(
                        "NormMoveDir: [{:.2}, {:.2}]",
                        norm_move_dir.0.x, norm_move_dir.0.y
                    ));
                } else {
                    ui.label("NormMoveDir: missing");
                }
                if let Ok(speed_magnitude) = speed_magnitude_query.get(world, selected_being_entity) {
                    ui.label(format!("SpeedMagnitude: {:.2}", speed_magnitude.0));
                } else {
                    ui.label("SpeedMagnitude: missing");
                }
                if let Ok(input_move_dir) = input_move_dir_query.get(world, selected_being_entity) {
                    ui.label(format!(
                        "InputMoveDir: [{:.2}, {:.2}]",
                        input_move_dir.0.x, input_move_dir.0.y
                    ));
                } else {
                    ui.label("InputMoveDir: missing");
                }
                if let Ok(gtrans) = gtrans_query.get(world, selected_being_entity) {
                    let translation = gtrans.translation();
                    ui.label(format!(
                        "GlobalTransform: [{:.0}, {:.0}, {}]",
                        translation.x, translation.y, translation.z
                    ));
                }
                if let Ok(grid_move) = grid_move_query.get(world, selected_being_entity) {
                    if let Ok(gpos) = gpos_query.get(world, selected_being_entity) {
                        ui.label(format!("GlobalTilePos: [{}, {}]", gpos.0.x, gpos.0.y));
                    }
                    ui.label(format!(
                        "GridLocked.origin: [{}, {}]",
                        grid_move.visual_origin_tile.x, grid_move.visual_origin_tile.y
                    ));
                    ui.label(format!(
                        "GridLocked.progress: {} / {}",
                        grid_move.progress_ticks, grid_move.step_ticks_total
                    ));
                    ui.label(format!(
                        "GridLocked.step_dir: [{}, {}]",
                        grid_move.step_dir.x, grid_move.step_dir.y
                    ));
                }
                if let Ok(facing) = facing_query.get(world, selected_being_entity) {
                    ui.label(format!("Facing: {:?}", facing));
                }
                let mut walk_add: f32 = 0.0;
                let mut walk_mul: f32 = 1.0;
                let mut walk_min: f32 = 0.0;
                let mut walk_max: f32 = f32::INFINITY;
                let mut walk_count = 0usize;
                for (target, value, op, _mitigating_only) in walk_modifiers_query.iter(world) {
                    if target.0 != selected_being_entity {
                        continue;
                    }
                    walk_count += 1;
                    let value = value.map_or(0.0, |value| value.0);
                    match op {
                        ApplyMode::Add => walk_add += value,
                        ApplyMode::Mul => walk_mul *= value.max(0.0),
                        ApplyMode::Min => walk_min = walk_min.max(value),
                        ApplyMode::Max => walk_max = walk_max.min(value).max(0.0),
                    }
                }
                ui.label(format!("WalkSpeed modifiers: {}", walk_count));
                ui.label(format!(
                    "WalkSpeed combine: add={:.3}, mul={:.3}, min={:.3}, max={:.3}",
                    walk_add, walk_mul, walk_min, walk_max
                ));
            });
            ui.separator();
            ui.collapsing("Inventory", |ui| {
                for (holder_entity, holder_label) in &inventory_holders {
                    let Ok(held_items) = held_items_query.get(world, *holder_entity) else {
                        continue;
                    };
                    let Ok(slot_holder) = slot_holder_query.get(world, *holder_entity) else {
                        continue;
                    };
                    let slot_entries = slot_holder
                        .0
                        .iter()
                        .map(|(slot, (entities, limit))| {
                            (
                                slot.clone(),
                                entities.iter().copied().collect::<Vec<_>>(),
                                *limit,
                            )
                        })
                        .collect::<Vec<_>>();
                    let has_available_slots = slot_holder
                        .0
                        .iter()
                        .any(|(_, (entities, limit))| entities.len() < *limit as usize);
                    let held_item_count = held_items.len();
                    let held_item_entities = held_items.iter().collect::<Vec<_>>();
                    if held_item_count == 0 && !has_available_slots {
                        continue;
                    }
                    ui.collapsing(holder_label, |ui| {
                        ui.label(format!("Held items: {}", held_item_count));
                        if held_item_count == 0 {
                            ui.label("No held items.");
                        } else {
                            for &item_entity in held_item_entities.iter() {
                                let item_label = part_label(
                                    item_entity,
                                    display_name_query.get(world, item_entity).ok(),
                                    str_id_query.get(world, item_entity).ok(),
                                );
                                ui.horizontal(|ui| {
                                    ui.label(item_label);
                                    if ui.button("Drop").clicked() {
                                        world
                                            .resource_mut::<Messages<ItemOperation>>()
                                            .write(ItemOperation::drop_preexisting_on_holder_position(item_entity));
                                    }
                                });
                            }
                        }

                        ui.label(format!("Equip slots: {}", slot_entries.len()));
                        for (slot, entities, limit) in slot_entries {
                            ui.collapsing(format!("{} [{}/{}]", slot, entities.len(), limit), |ui| {
                                if entities.is_empty() {
                                    ui.label("Empty");
                                } else {
                                    for item_entity in entities {
                                        ui.label(part_label(
                                            item_entity,
                                            display_name_query.get(world, item_entity).ok(),
                                            str_id_query.get(world, item_entity).ok(),
                                        ));
                                    }
                                }
                            });
                        }
                    });
                }
            });
            ui.separator();
            ui.collapsing("Body Part Stats", |ui| {

                if part_infos.is_empty() {
                    ui.label("No body parts found.");
                    return;
                }

                egui::ComboBox::from_label("Body part")
                    .selected_text(
                        part_infos
                            .iter()
                            .find(|(entity, _)| Some(*entity) == selected_part)
                            .map_or_else(|| "None".to_string(), |(_, label)| label.clone()),
                    )
                    .show_ui(ui, |ui| {
                        for (part_entity, label) in &part_infos {
                            ui.selectable_value(&mut selected_part, Some(*part_entity), label);
                        }
                    });

                let Some(selected_part_entity) = selected_part else {
                    return;
                };
                let part_damage = body_part_damage_query
                    .get(world, selected_part_entity)
                    .map_or(0.0, |damage| damage.0);
                ui.label(format!("Part damage: {:.2}", part_damage));
                ui.separator();

                let mut hp_capacity = (0.0, 0.0);
                let mut hp_regen = (0.0, 0.0);
                let mut blood_capacity = (0.0, 0.0);
                let mut bleed_rate = (0.0, 0.0);
                let mut pain_sensitivity = (0.0, 0.0);
                let mut vision = (0.0, 0.0);
                let mut manip_dex = (0.0, 0.0);
                let mut manip_str = (0.0, 0.0);

                for (
                    target,
                    base,
                    effective,
                    has_hp_capacity,
                    has_hp_regen,
                    has_blood_capacity,
                    has_bleed_rate,
                    has_pain_sensitivity,
                    has_vision,
                    has_manip_dex,
                    has_manip_str,
                ) in modifiers_query.iter(world)
                {
                    if target.0 != selected_part_entity {
                        continue;
                    }
                    let base = base.map_or(0.0, |value| value.0);
                    let effective = effective.map_or(base, |value| value.0);

                    if has_hp_capacity {
                        hp_capacity.0 += base;
                        hp_capacity.1 += effective;
                    }
                    if has_hp_regen {
                        hp_regen.0 += base;
                        hp_regen.1 += effective;
                    }
                    if has_blood_capacity {
                        blood_capacity.0 += base;
                        blood_capacity.1 += effective;
                    }
                    if has_bleed_rate {
                        bleed_rate.0 += base;
                        bleed_rate.1 += effective;
                    }
                    if has_pain_sensitivity {
                        pain_sensitivity.0 += base;
                        pain_sensitivity.1 += effective;
                    }
                    if has_vision {
                        vision.0 += base;
                        vision.1 += effective;
                    }
                    if has_manip_dex {
                        manip_dex.0 += base;
                        manip_dex.1 += effective;
                    }
                    if has_manip_str {
                        manip_str.0 += base;
                        manip_str.1 += effective;
                    }
                }

                for (label, (base, effective)) in [
                    ("hp_capacity", hp_capacity),
                    ("hp_regen_rate", hp_regen),
                    ("blood_capacity", blood_capacity),
                    ("bleed_rate", bleed_rate),
                    ("pain_sensitivity", pain_sensitivity),
                    ("vision", vision),
                    ("manip_dex", manip_dex),
                    ("manip_str", manip_str),
                ] {
                    ui.label(format!(
                        "{}: {:.2} (synergy {:+.2})",
                        label,
                        effective,
                        effective - base
                    ));
                }
            });
        });

    if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
        if clear_selection {
            selected_entities.selected_being = None;
            selected_entities.selected_being_bodypart = None;
            selected_entities.show_full_being_components = false;
        } else {
            selected_entities.selected_being_bodypart = selected_part;
            selected_entities.show_full_being_components = show_full_components;
        }
    }

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.being_details = false;
        }
    }
}
