use ac_input::ac_input_actions::*;
use ::being_shared::*;

use being::body::{Body, BodySums};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Action, Actions};
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;
use common::common_components::{DisplayName, StrId};
use common::common_tag_components::TagSet;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use item_shared::{
    item_components::{HeldItems, SlottedItemHolder},
    ItemOperation,
};
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::*;
use modifier_shared::{modifier_has_marker, resolve_modifier_component};
use movement::movement_components::*;
use player::prelude::*;
use tilemap_shared::CardinalDirection;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[derive(Default, Copy, Clone)]
struct StatSummary {
    base: f32,
    effective: f32,
}

impl StatSummary {
    fn add(&mut self, base: f32, effective: f32) {
        self.base += base;
        self.effective += effective;
    }

    fn or_fallback(self, fallback: Self) -> Self {
        if self.base != 0.0 || self.effective != 0.0 {
            self
        } else {
            fallback
        }
    }
}

#[derive(Default, Copy, Clone)]
struct PartStats {
    hp_capacity: StatSummary,
    hp_regen: StatSummary,
    blood_capacity: StatSummary,
    bleed_rate: StatSummary,
    pain_sensitivity: StatSummary,
    vision: StatSummary,
    manip_dex: StatSummary,
    manip_str: StatSummary,
}

impl PartStats {
    fn with_fallback(self, fallback: Self) -> Self {
        Self {
            hp_capacity: self.hp_capacity.or_fallback(fallback.hp_capacity),
            hp_regen: self.hp_regen.or_fallback(fallback.hp_regen),
            blood_capacity: self.blood_capacity.or_fallback(fallback.blood_capacity),
            bleed_rate: self.bleed_rate.or_fallback(fallback.bleed_rate),
            pain_sensitivity: self.pain_sensitivity.or_fallback(fallback.pain_sensitivity),
            vision: self.vision.or_fallback(fallback.vision),
            manip_dex: self.manip_dex.or_fallback(fallback.manip_dex),
            manip_str: self.manip_str.or_fallback(fallback.manip_str),
        }
    }
}

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

fn modifier_values(
    world: &World,
    modifier_ent: Entity,
    ezero_ref: Option<&EntityZeroRef>,
    base_values_query: &mut QueryState<&BaseValue>,
    curr_values_query: &mut QueryState<&CurrEffectiveValue>,
) -> StatSummary {
    let base = resolve_modifier_component(modifier_ent, ezero_ref, &base_values_query.query(world))
        .map(|value| value.0)
        .unwrap_or(0.0);
    let effective = resolve_modifier_component(modifier_ent, ezero_ref, &curr_values_query.query(world))
        .map(|value| value.0)
        .unwrap_or(base);
    StatSummary { base, effective }
}

fn collect_part_stats(
    world: &World,
    target_ent: Entity,
    applied_mods: &AppliedModifiers,
    modifiers_query: &mut QueryState<(Entity, &ModifierTarget, Option<&EntityZeroRef>), Without<EntityZero>>,
    base_values_query: &mut QueryState<&BaseValue>,
    curr_values_query: &mut QueryState<&CurrEffectiveValue>,
    hp_capacity_query: &mut QueryState<(), With<HitpointsCapacity>>,
    hp_regen_query: &mut QueryState<(), With<HitpointRegenRate>>,
    blood_capacity_query: &mut QueryState<(), With<BloodCapacity>>,
    bleed_rate_query: &mut QueryState<(), With<BleedRate>>,
    pain_sensitivity_query: &mut QueryState<(), With<PainSensitivity>>,
    vision_query: &mut QueryState<(), With<Vision>>,
    manip_dex_query: &mut QueryState<(), With<ManipulationDexterity>>,
    manip_str_query: &mut QueryState<(), With<ManipulationStrength>>,
) -> PartStats {
    let mut stats = PartStats::default();

    for modifier_ent in applied_mods.iter() {
        let Ok((modifier_ent, target, ezero_ref)) = modifiers_query.get(world, modifier_ent) else {
            continue;
        };
        if target.0 != target_ent {
            continue;
        }
        let values = modifier_values(
            world,
            modifier_ent,
            ezero_ref,
            base_values_query,
            curr_values_query,
        );
        if modifier_has_marker::<HitpointsCapacity>(
            modifier_ent,
            ezero_ref,
            &hp_capacity_query.query(world),
        ) {
            stats.hp_capacity.add(values.base, values.effective);
        }
        if modifier_has_marker::<HitpointRegenRate>(
            modifier_ent,
            ezero_ref,
            &hp_regen_query.query(world),
        ) {
            stats.hp_regen.add(values.base, values.effective);
        }
        if modifier_has_marker::<BloodCapacity>(
            modifier_ent,
            ezero_ref,
            &blood_capacity_query.query(world),
        ) {
            stats.blood_capacity.add(values.base, values.effective);
        }
        if modifier_has_marker::<BleedRate>(
            modifier_ent,
            ezero_ref,
            &bleed_rate_query.query(world),
        ) {
            stats.bleed_rate.add(values.base, values.effective);
        }
        if modifier_has_marker::<PainSensitivity>(
            modifier_ent,
            ezero_ref,
            &pain_sensitivity_query.query(world),
        ) {
            stats.pain_sensitivity.add(values.base, values.effective);
        }
        if modifier_has_marker::<Vision>(
            modifier_ent,
            ezero_ref,
            &vision_query.query(world),
        ) {
            stats.vision.add(values.base, values.effective);
        }
        if modifier_has_marker::<ManipulationDexterity>(
            modifier_ent,
            ezero_ref,
            &manip_dex_query.query(world),
        ) {
            stats.manip_dex.add(values.base, values.effective);
        }
        if modifier_has_marker::<ManipulationStrength>(
            modifier_ent,
            ezero_ref,
            &manip_str_query.query(world),
        ) {
            stats.manip_str.add(values.base, values.effective);
        }
    }

    stats
}

fn format_summary(label: &str, values: StatSummary) -> String {
    format!(
        "{}: {:.2} (synergy {:+.2})",
        label,
        values.effective,
        values.effective - values.base
    )
}

fn format_value_pair(values: StatSummary) -> String {
    if (values.base - values.effective).abs() <= f32::EPSILON {
        format!("{:.1}", values.base)
    } else {
        format!("base {:.1} curr {:.1}", values.base, values.effective)
    }
}

#[allow(unused_parens)]
pub fn being_details_inspector(world: &mut World) {
    let Some(window_visible) = world.get_resource::<DubugWindowsVisibility>() else {
        return;
    };
    if !window_visible.being_details {
        return;
    }

    let Some(selected_entities) = world.get_resource::<DebugSelectedEntities>() else {
        return;
    };
    let Some(selected_being_entity) = selected_entities.selected_being else {
        return;
    };
    let mut selected_part = selected_entities.selected_being_bodypart;
    let mut show_full_components = selected_entities.show_full_being_components;

    let mut egui_context_query = world.query_filtered::<
        &bevy_inspector_egui::bevy_egui::EguiContext,
        With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>,
    >();
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let mut body_query = world.query::<&Body>();
    let mut body_sums_query = world.query::<&BodySums>();
    let mut display_name_query = world.query::<&DisplayName>();
    let mut str_id_query = world.query::<&StrId>();
    let mut bodyparts_query = world.query::<&BodypartChildrenBodyparts>();
    let mut bodypart_damage_query = world.query::<&BodypartDamage>();
    let mut held_items_query = world.query::<&HeldItems>();
    let mut slot_holder_query = world.query::<&SlottedItemHolder>();
    let mut norm_move_dir_query = world.query::<&FinalNormMoveDir>();
    let mut speed_magnitude_query = world.query::<&SpeedMagnitude>();
    let mut input_move_dir_query = world.query::<&InputMoveDir>();
    let mut computed_by_query = world.query::<&ComputedBy>();
    let mut computed_locally_query = world.query::<&ComputedLocally>();
    let mut player_actions_query =
        world.query_filtered::<&Actions<BeingDirectControlInputContext>, MyPlayer>();
    let mut player_move_action_query = world.query::<&Action<DcWasdAction>>();
    let mut grid_move_query = world.query::<&GridLockedMovement>();
    let mut gpos_query = world.query::<&tilemap_shared::GlobalTilePos>();
    let mut facing_query = world.query::<&CardinalDirection>();
    let mut ezero_refs_query = world.query::<&EntityZeroRef>();
    let mut applied_mods_query = world.query::<&AppliedModifiers>();
    let mut modifiers_query =
        world.query_filtered::<(Entity, &ModifierTarget, Option<&EntityZeroRef>), Without<EntityZero>>();
    let mut base_values_query = world.query::<&BaseValue>();
    let mut curr_values_query = world.query::<&CurrEffectiveValue>();
    let mut apply_modes_query = world.query::<&ApplyMode>();
    let mut hp_capacity_query = world.query_filtered::<(), With<HitpointsCapacity>>();
    let mut hp_regen_query = world.query_filtered::<(), With<HitpointRegenRate>>();
    let mut blood_capacity_query = world.query_filtered::<(), With<BloodCapacity>>();
    let mut bleed_rate_query = world.query_filtered::<(), With<BleedRate>>();
    let mut pain_sensitivity_query = world.query_filtered::<(), With<PainSensitivity>>();
    let mut vision_query = world.query_filtered::<(), With<Vision>>();
    let mut manip_dex_query = world.query_filtered::<(), With<ManipulationDexterity>>();
    let mut manip_str_query = world.query_filtered::<(), With<ManipulationStrength>>();
    let mut walk_modifiers_query = world.query_filtered::<
        (
            Entity,
            &ModifierTarget,
            Option<&ChildOf>,
            Option<&EntityZeroRef>,
            Option<&TagSet>,
            Has<PainSlowdown>,
        ),
        (With<WalkSpeed>, Without<EntityZero>),
    >();

    let Ok(body) = body_query.get(world, selected_being_entity) else {
        return;
    };
    let body_entity = body.entity();
    let body_label = part_label(
        body_entity,
        display_name_query.get(world, body_entity).ok(),
        str_id_query.get(world, body_entity).ok(),
    );
    let body_sums = body_sums_query.get(world, body_entity).ok().cloned();
    let body_ezero_ref = ezero_refs_query.get(world, body_entity).ok().copied();

    let mut inventory_holders = vec![(
        selected_being_entity,
        part_label(
            selected_being_entity,
            display_name_query.get(world, selected_being_entity).ok(),
            str_id_query.get(world, selected_being_entity).ok(),
        ),
    )];
    inventory_holders.push((body_entity, body_label.clone()));

    let mut part_infos: Vec<(Entity, String)> = Vec::new();
    let Ok(parts) = bodyparts_query.get(world, body_entity) else {
        return;
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
    if selected_part.is_none() || !part_infos.iter().any(|(entity, _)| Some(*entity) == selected_part) {
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

            ui.heading("Body");
            ui.label(format!("Body: {} [{:?}]", body_label, body_entity));
            ui.label(format!(
                "Body ezero ref: {}",
                body_ezero_ref.map_or("missing".to_string(), |refe| format!("{:?}", refe.0))
            ));
            if let Some(sums) = body_sums {
                ui.label(format!("HP: {:.2}/{:.2}", sums.current_hp, sums.total_hp));
                ui.label(format!("Blood: {:.2}/{:.2}", sums.blood, sums.blood_capacity));
                ui.label(format!("Bleed rate: {:.2}", sums.bleed_rate));
                ui.label(format!("Consciousness: {:.2}", sums.consciousness));
                ui.label(format!("Pain: {:.2}", sums.pain));
                ui.label(format!("Vision: {:.2}", sums.vision));
                ui.label(format!("Manip dex: {:.2}", sums.manip_dex));
                ui.label(format!("Manip str: {:.2}", sums.manip_str));
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
                }
                if let Ok(speed_magnitude) = speed_magnitude_query.get(world, selected_being_entity) {
                    ui.label(format!("SpeedMagnitude: {:.2}", speed_magnitude.0));
                }
                if let Ok(input_move_dir) = input_move_dir_query.get(world, selected_being_entity) {
                    ui.label(format!(
                        "InputMoveDir: [{:.2}, {:.2}]",
                        input_move_dir.0.x, input_move_dir.0.y
                    ));
                }
                if grid_move_query.get(world, selected_being_entity).is_ok() {
                    if let Ok(gpos) = gpos_query.get(world, selected_being_entity) {
                        ui.label(format!("GlobalTilePos: [{}, {}]", gpos.0.x, gpos.0.y));
                    }
                }
                if let Ok(facing) = facing_query.get(world, selected_being_entity) {
                    ui.label(format!("Facing: {:?}", facing));
                }

                let mut walk_add: f32 = 0.0;
                let mut walk_mul: f32 = 1.0;
                let mut walk_min: f32 = 0.0;
                let mut walk_max: f32 = f32::INFINITY;
                let mut walk_rows = Vec::new();
                let mut source_bodyparts = Vec::new();
                if let Ok(bodyparts) = bodyparts_query.get(world, body_entity) {
                    for bodypart_ent in bodyparts.iter() {
                        let Ok(part_ezero_ref) = ezero_refs_query.get(world, bodypart_ent) else {
                            continue;
                        };
                        source_bodyparts.push(part_ezero_ref.0);
                    }
                }
                for (modifier_ent, target, child_of, ezero_ref, tagset, pain_slowdown) in walk_modifiers_query.iter(world) {
                    let from_source_bodypart = child_of
                        .map(|child| source_bodyparts.contains(&child.parent()))
                        .unwrap_or(false);
                    if target.0 != selected_being_entity && !from_source_bodypart {
                        continue;
                    }
                    let values = modifier_values(
                        world,
                        modifier_ent,
                        ezero_ref,
                        &mut base_values_query,
                        &mut curr_values_query,
                    );
                    let Some(op) = resolve_modifier_component(
                        modifier_ent,
                        ezero_ref,
                        &apply_modes_query.query(world),
                    ) else {
                        continue;
                    };
                    match op {
                        ApplyMode::Add => walk_add += values.effective,
                        ApplyMode::Mul => walk_mul *= values.effective.max(0.0),
                        ApplyMode::Min => walk_min = walk_min.max(values.effective),
                        ApplyMode::Max => walk_max = walk_max.min(values.effective).max(0.0),
                    }
                    let mut row = format!(
                        "{:?} ChOf {} {} {:?}",
                        modifier_ent,
                        child_of.map_or("missing".to_string(), |child| format!("{:?}", child.parent())),
                        format_value_pair(values),
                        op,
                    );
                    if let Some(tags) = tagset {
                        row.push(' ');
                        row.push_str(&format!("{:?}", tags));
                    }
                    if pain_slowdown {
                        row.push_str(" PainMult");
                    }
                    if let Some(ezero_ref) = ezero_ref {
                        row.push_str(&format!(" EzRef {:?}", ezero_ref.0));
                    }
                    walk_rows.push(row);
                }
                ui.collapsing(
                    format!(
                        "WalkSpeed Modifiers: {} | add {:.1} mul {:.1} min {:.1} max {:.1}",
                        walk_rows.len(),
                        walk_add,
                        walk_mul,
                        walk_min,
                        walk_max,
                    ),
                    |ui| {
                        if walk_rows.is_empty() {
                            ui.label("No active WalkSpeed modifiers.");
                            return;
                        }
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                            for row in &walk_rows {
                                ui.label(row);
                            }
                        });
                    },
                );
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
                    let held_item_entities = held_items.iter().collect::<Vec<_>>();
                    let held_item_count = held_item_entities.len();
                    let slot_entries = slot_holder
                        .0
                        .iter()
                        .map(|(slot, (entities, limit))| {
                            (slot.clone(), entities.iter().copied().collect::<Vec<_>>(), *limit)
                        })
                        .collect::<Vec<_>>();
                    let has_available_slots = slot_holder
                        .0
                        .iter()
                        .any(|(_, (entities, limit))| entities.len() < *limit as usize);
                    if held_item_count == 0 && !has_available_slots {
                        continue;
                    }
                    ui.collapsing(holder_label, |ui| {
                        ui.label(format!("Held items: {}", held_item_count));
                        for item_entity in held_item_entities.iter().copied() {
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
                        for (slot, entities, limit) in slot_entries {
                            ui.collapsing(format!("{} [{}/{}]", slot, entities.len(), limit), |ui| {
                                if entities.is_empty() {
                                    ui.label("Empty");
                                    return;
                                }
                                for item_entity in entities {
                                    ui.label(part_label(
                                        item_entity,
                                        display_name_query.get(world, item_entity).ok(),
                                        str_id_query.get(world, item_entity).ok(),
                                    ));
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
                ui.label(format!("Part: {:?}", selected_part_entity));
                let part_damage = bodypart_damage_query
                    .get(world, selected_part_entity)
                    .map_or(0.0, |damage| damage.0);
                ui.label(format!("Part damage: {:.2}", part_damage));

                let source_part_ent = ezero_refs_query.get(world, selected_part_entity).ok().map(|refe| refe.0);
                ui.label(format!(
                    "Part ezero ref: {}",
                    source_part_ent.map_or("missing".to_string(), |ent| format!("{:?}", ent))
                ));
                ui.separator();

                let runtime_stats = applied_mods_query
                    .get(world, selected_part_entity)
                    .map(|applied_mods| {
                        collect_part_stats(
                            world,
                            selected_part_entity,
                            applied_mods,
                            &mut modifiers_query,
                            &mut base_values_query,
                            &mut curr_values_query,
                            &mut hp_capacity_query,
                            &mut hp_regen_query,
                            &mut blood_capacity_query,
                            &mut bleed_rate_query,
                            &mut pain_sensitivity_query,
                            &mut vision_query,
                            &mut manip_dex_query,
                            &mut manip_str_query,
                        )
                    })
                    .unwrap_or_default();
                let source_stats = source_part_ent
                    .and_then(|source_part_ent| {
                        let Ok(applied_mods) = applied_mods_query.get(world, source_part_ent) else {
                            return None;
                        };
                        Some(collect_part_stats(
                            world,
                            source_part_ent,
                            applied_mods,
                            &mut modifiers_query,
                            &mut base_values_query,
                            &mut curr_values_query,
                            &mut hp_capacity_query,
                            &mut hp_regen_query,
                            &mut blood_capacity_query,
                            &mut bleed_rate_query,
                            &mut pain_sensitivity_query,
                            &mut vision_query,
                            &mut manip_dex_query,
                            &mut manip_str_query,
                        ))
                    })
                    .unwrap_or_default();
                let stats = runtime_stats.with_fallback(source_stats);

                for line in [
                    format_summary("hp_capacity", stats.hp_capacity),
                    format_summary("hp_regen_rate", stats.hp_regen),
                    format_summary("blood_capacity", stats.blood_capacity),
                    format_summary("bleed_rate", stats.bleed_rate),
                    format_summary("pain_sensitivity", stats.pain_sensitivity),
                    format_summary("vision", stats.vision),
                    format_summary("manip_dex", stats.manip_dex),
                    format_summary("manip_str", stats.manip_str),
                ] {
                    ui.label(line);
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
