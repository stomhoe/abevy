use ac_input::ac_input_actions::*;
use ::being_shared::*;
use ::being_shared::body_energy::*;
use std::collections::BTreeMap;

use being::body::{BodySums, DamageDistributeMode, HeldBody, IncHealthDamageOrHeal};
use being::being_nav::RetainedChasePathSnapshot;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Action, Actions};
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;
use bevy_replicon::prelude::ClientState;
use common::common_components::{DisplayName, StrId};
use common::common_components::HashId;
use common::log_targets::DEBUG;
use game_common::game_common_components::Dead;
use game_common::game_common_components::{Templ, TemplEntiRef};
use ::item_shared::*;
use ::modifier_shared::*;
use player_shared::{player_components::*, };
use ::sprite_shared::*;
use ::tilemap_shared::*;

use crate::debug_messages::{
    ClientDebugSetBeingCurrentBloodRequest, ClientDebugSetBeingCurrentHpRequest, ClientDebugSetBeingDimensionRequest,
    ClientDebugKillBeingRequest, ClientDebugReviveBeingRequest, ClientDebugTeleportBeingRequest,
};
use debug_shared::{
    DebugBeingLocationEditorState, DebugBeingVitalsAdjustState, DebugSelectedEntities,
    DebugUiConfig, DubugWindowsVisibility,
};

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

fn render_body_vitals_controls(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_being_entity: Entity,
    body_entity: Entity,
    body_sums: &BodySums,
) {
    let route_via_client = world
        .get_resource::<DebugUiConfig>()
        .is_some_and(|cfg| cfg.client_debug)
        && world.get_resource::<State<ClientState>>().is_some_and(|state| *state.get() == ClientState::Connected);

    let mut hp_commit_target = None;
    let mut hp_commit_amount = 0.0;
    let mut hp_commit_via_client = false;
    let mut blood_commit_target = None;
    let mut blood_commit_via_client = false;
    let mut kill_requested = false;
    let mut revive_requested = false;

    {
        let mut adjust_state = world.resource_mut::<DebugBeingVitalsAdjustState>();
        let state = adjust_state.states.entry(body_entity).or_default();

        let hp_max = body_sums.total_hp.max(0.0);
        let live_hp = body_sums.current_hp.clamp(0.0, hp_max);
        if let Some(target) = state.hp_pending_target {
            if (live_hp - target).abs() <= 0.01 {
                state.hp_pending_target = None;
            }
        }
        if !state.hp_dragging && state.hp_pending_target.is_none() {
            state.current_hp = live_hp;
        }

        let hp_response = ui.add_enabled(
            hp_max > 0.0,
            egui::Slider::new(&mut state.current_hp, 0.0..=hp_max).text("Current HP"),
        );
        if hp_response.dragged() {
            state.hp_dragging = true;
        }
        if state.hp_dragging && !ui.input(|input| input.pointer.primary_down()) {
            let target_hp = state.current_hp.clamp(0.0, hp_max);
            if (live_hp - target_hp).abs() > 0.01 {
                state.hp_pending_target = Some(target_hp);
                hp_commit_target = Some(target_hp);
                hp_commit_amount = live_hp - target_hp;
                hp_commit_via_client = route_via_client;
            }
            state.hp_dragging = false;
        }

        let blood_max = body_sums.blood_capacity.max(0.0);
        let live_blood = if body_sums.blood.is_nan() {
            blood_max
        } else {
            body_sums.blood.clamp(0.0, blood_max)
        };
        if let Some(target) = state.blood_pending_target {
            if (live_blood - target).abs() <= 0.01 {
                state.blood_pending_target = None;
            }
        }
        if !state.blood_dragging && state.blood_pending_target.is_none() {
            state.blood = live_blood;
        }

        let blood_response = ui.add_enabled(
            blood_max > 0.0,
            egui::Slider::new(&mut state.blood, 0.0..=blood_max).text("Blood"),
        );
        if blood_response.dragged() {
            state.blood_dragging = true;
        }
        if state.blood_dragging && !ui.input(|input| input.pointer.primary_down()) {
            let target_blood = state.blood.clamp(0.0, blood_max);
            if (live_blood - target_blood).abs() > 0.01 {
                state.blood_pending_target = Some(target_blood);
                blood_commit_target = Some(target_blood);
                blood_commit_via_client = route_via_client;
            }
            state.blood_dragging = false;
        }

        ui.horizontal(|ui| {
            if ui.button("Kill").clicked() {
                kill_requested = true;
            }
            if ui.button("Revive").clicked() {
                revive_requested = true;
            }
        });
    }

    if let Some(target_hp) = hp_commit_target {
        if hp_commit_via_client {
            world
                .resource_mut::<Messages<ClientDebugSetBeingCurrentHpRequest>>()
                .write(ClientDebugSetBeingCurrentHpRequest {
                    being_ent: selected_being_entity,
                    current_hp: target_hp,
                });
        } else {
            world
                .resource_mut::<Messages<IncHealthDamageOrHeal>>()
                .write(IncHealthDamageOrHeal {
                    source_ent: selected_being_entity,
                    target_ent: body_entity,
                    amount: hp_commit_amount,
                    distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
                });
        }
        debug!(
            target: DEBUG,
            "Committed current HP adjustment for {:?} body {:?} to {:.2}",
            selected_being_entity,
            body_entity,
            target_hp,
        );
    }

    if let Some(target_blood) = blood_commit_target {
        if blood_commit_via_client {
            world
                .resource_mut::<Messages<ClientDebugSetBeingCurrentBloodRequest>>()
                .write(ClientDebugSetBeingCurrentBloodRequest {
                    being_ent: selected_being_entity,
                    blood: target_blood,
                });
        } else if let Some(mut body_sums) = world.get_mut::<BodySums>(body_entity) {
            body_sums.blood = target_blood.max(0.0);
        }
        debug!(
            target: DEBUG,
            "Committed blood adjustment for {:?} body {:?} to {:.2}",
            selected_being_entity,
            body_entity,
            target_blood,
        );
    }

    if kill_requested {
        if route_via_client {
            world
                .resource_mut::<Messages<ClientDebugKillBeingRequest>>()
                .write(ClientDebugKillBeingRequest {
                    being_ent: selected_being_entity,
                });
        } else {
            let current_hp = body_sums.current_hp.max(0.0);
            world
                .resource_mut::<Messages<IncHealthDamageOrHeal>>()
                .write(IncHealthDamageOrHeal {
                    source_ent: selected_being_entity,
                    target_ent: body_entity,
                    amount: current_hp,
                    distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
                });
            if let Some(mut body_sums) = world.get_mut::<BodySums>(body_entity) {
                body_sums.blood = 0.0;
            }
            world.entity_mut(selected_being_entity).insert_if_new(Dead);
        }
    }

    if revive_requested {
        if route_via_client {
            world
                .resource_mut::<Messages<ClientDebugReviveBeingRequest>>()
                .write(ClientDebugReviveBeingRequest {
                    being_ent: selected_being_entity,
                });
        } else {
            let current_hp = body_sums.current_hp.max(0.0);
            let total_hp = body_sums.total_hp.max(0.0);
            world
                .resource_mut::<Messages<IncHealthDamageOrHeal>>()
                .write(IncHealthDamageOrHeal {
                    source_ent: selected_being_entity,
                    target_ent: body_entity,
                    amount: current_hp - total_hp,
                    distribute_mode: DamageDistributeMode::EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
                });
            if let Some(mut body_sums) = world.get_mut::<BodySums>(body_entity) {
                body_sums.blood = body_sums.blood_capacity.max(0.0);
            }
            world.entity_mut(selected_being_entity).remove::<Dead>();
            if let Some(gpos) = world.get::<GlobalTilePos>(selected_being_entity).copied() {
                if let Some(mut transform) = world.get_mut::<Transform>(selected_being_entity) {
                    let z = transform.translation.z;
                    *transform = Transform::from_translation(gpos.to_translation(z));
                }
            }
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

pub(crate) fn part_label(entity: Entity, display_name: Option<&DisplayName>, str_id: Option<&StrId>) -> String {
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

fn dimension_label(name: Option<&Name>, dim_ref: DimensionRef) -> String {
    name.map(|name| name.to_string())
        .unwrap_or_else(|| format!("{:?}", dim_ref))
}

fn paint_collision_mask_preview(ui: &mut egui::Ui, collision_zone: &tilemap_shared::InteractionZone, facing: CardinalDirection) {
    let mut zone_positions = Vec::new();
    collision_zone.gather_zone_positions(facing, Vec2::ZERO, &mut zone_positions);
    if zone_positions.is_empty() {
        ui.label("<empty>");
        return;
    }

    let mut min_x = zone_positions[0].0.x;
    let mut max_x = zone_positions[0].0.x;
    let mut min_y = zone_positions[0].0.y;
    let mut max_y = zone_positions[0].0.y;
    for pos in zone_positions.iter().copied().skip(1) {
        min_x = min_x.min(pos.0.x);
        max_x = max_x.max(pos.0.x);
        min_y = min_y.min(pos.0.y);
        max_y = max_y.max(pos.0.y);
    }

    let cell_size = 18.0;
    let width = (max_x - min_x + 1) as f32 * cell_size;
    let height = (max_y - min_y + 1) as f32 * cell_size;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let background = ui.visuals().extreme_bg_color;
    let stroke = egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
    painter.rect_filled(rect, 3.0, background);

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let cell = GlobalTilePos::new(x, y);
            let occupied = zone_positions.contains(&cell);
            let x_idx = (x - min_x) as f32;
            let y_idx = (max_y - y) as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + x_idx * cell_size, rect.min.y + y_idx * cell_size),
                egui::vec2(cell_size, cell_size),
            )
            .shrink(1.0);
            painter.rect_stroke(cell_rect, 2.0, stroke, egui::StrokeKind::Inside);
            if occupied {
                painter.rect_filled(cell_rect.shrink(1.0), 1.0, ui.visuals().selection.bg_fill);
            }
        }
    }
}

fn paint_interaction_zone_preview(
    ui: &mut egui::Ui,
    interaction_zone: &InteractionZone,
    facing: CardinalDirection,
    anchor_gpos: GlobalTilePos,
) {
    let mut zone_positions = Vec::new();
    interaction_zone.gather_zone_positions(facing, anchor_gpos.to_pixelpos(), &mut zone_positions);
    let zone_positions = zone_positions
        .into_iter()
        .map(|pos| GlobalTilePos(pos.0 - anchor_gpos.0))
        .collect::<Vec<_>>();

    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;
    for pos in zone_positions.iter().copied() {
        min_x = min_x.min(pos.0.x);
        max_x = max_x.max(pos.0.x);
        min_y = min_y.min(pos.0.y);
        max_y = max_y.max(pos.0.y);
    }

    let cell_size = 18.0;
    let width = (max_x - min_x + 1) as f32 * cell_size;
    let height = (max_y - min_y + 1) as f32 * cell_size;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let background = ui.visuals().extreme_bg_color;
    let stroke = egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
    let anchor_stroke = egui::Stroke::new(2.0, egui::Color32::YELLOW);
    painter.rect_filled(rect, 3.0, background);

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let cell = GlobalTilePos::new(x, y);
            let occupied = zone_positions.contains(&cell);
            let is_anchor = cell == GlobalTilePos::default();
            let x_idx = (x - min_x) as f32;
            let y_idx = (max_y - y) as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + x_idx * cell_size, rect.min.y + y_idx * cell_size),
                egui::vec2(cell_size, cell_size),
            )
            .shrink(1.0);
            painter.rect_stroke(cell_rect, 2.0, stroke, egui::StrokeKind::Inside);
            if is_anchor {
                painter.rect_stroke(cell_rect, 2.0, anchor_stroke, egui::StrokeKind::Inside);
            }
            if occupied {
                painter.rect_filled(cell_rect.shrink(1.0), 1.0, ui.visuals().selection.bg_fill);
            }
        }
    }
}

fn paint_meeting_slots_preview(
    ui: &mut egui::Ui,
    leader_gpos: GlobalTilePos,
    reserved_targets: &[GlobalTilePos],
) {
    let mut rel_positions = reserved_targets
        .iter()
        .map(|target| target.0 - leader_gpos.0)
        .collect::<Vec<_>>();
    rel_positions.sort_unstable_by_key(|rel| (rel.x, rel.y));
    rel_positions.dedup();

    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;
    for rel in rel_positions.iter().copied() {
        min_x = min_x.min(rel.x);
        max_x = max_x.max(rel.x);
        min_y = min_y.min(rel.y);
        max_y = max_y.max(rel.y);
    }

    let cell_size = 14.0;
    let width = (max_x - min_x + 1) as f32 * cell_size;
    let height = (max_y - min_y + 1) as f32 * cell_size;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let background = ui.visuals().extreme_bg_color;
    let stroke = egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
    painter.rect_filled(rect, 3.0, background);

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let x_idx = (x - min_x) as f32;
            let y_idx = (max_y - y) as f32;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + x_idx * cell_size, rect.min.y + y_idx * cell_size),
                egui::vec2(cell_size, cell_size),
            )
            .shrink(1.0);
            painter.rect_stroke(cell_rect, 2.0, stroke, egui::StrokeKind::Inside);
        }
    }

    let leader_rel = IVec2::ZERO;
    let leader_x_idx = (leader_rel.x - min_x) as f32;
    let leader_y_idx = (max_y - leader_rel.y) as f32;
    let leader_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.min.x + leader_x_idx * cell_size,
            rect.min.y + leader_y_idx * cell_size,
        ),
        egui::vec2(cell_size, cell_size),
    )
    .shrink(2.0);
    painter.rect_filled(leader_rect, 1.0, egui::Color32::YELLOW);

    for rel in rel_positions.iter().copied() {
        if rel == IVec2::ZERO {
            continue;
        }
        let x_idx = (rel.x - min_x) as f32;
        let y_idx = (max_y - rel.y) as f32;
        let cell_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + x_idx * cell_size, rect.min.y + y_idx * cell_size),
            egui::vec2(cell_size, cell_size),
        )
        .shrink(2.0);
        painter.rect_filled(cell_rect, 1.0, ui.visuals().selection.bg_fill);
    }
}

fn resolve_interaction_zones_with_source(
    world: &World,
    being_ent: Entity,
    bit_ref_ent: Option<Entity>,
    race_ref_ent: Option<Entity>,
    interaction_zones_query: &mut QueryState<&InteractionZones>,
) -> Option<(InteractionZones, &'static str, Entity)> {
    if let Ok(zones) = interaction_zones_query.get(world, being_ent) {
        return Some((zones.clone(), "being", being_ent));
    }
    if let Some(bit_ent) = bit_ref_ent {
        if let Ok(zones) = interaction_zones_query.get(world, bit_ent) {
            return Some((zones.clone(), "bit", bit_ent));
        }
    }
    if let Some(race_ent) = race_ref_ent {
        if let Ok(zones) = interaction_zones_query.get(world, race_ent) {
            return Some((zones.clone(), "race", race_ent));
        }
    }
    None
}

fn modifier_values(
    world: &World,
    modifier_ent: Entity,
    templ_ref: Option<&TemplEntiRef>,
    base_values_query: &mut QueryState<&BaseValue>,
    curr_values_query: &mut QueryState<&CurrEffectiveValue>,
) -> StatSummary {
    let base = resolve_modifier_component(modifier_ent, templ_ref, &base_values_query.query(world))
        .map(|value| value.0)
        .unwrap_or(0.0);
    let effective = resolve_modifier_component(modifier_ent, templ_ref, &curr_values_query.query(world))
        .map(|value| value.0)
        .unwrap_or(base);
    StatSummary { base, effective }
}

fn collect_part_stats(
    world: &World,
    target_ent: Entity,
    applied_mods: &AppliedModifiers,
    modifiers_query: &mut QueryState<(Entity, &ModifierTarget, Option<&TemplEntiRef>), Without<Templ>>,
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
        let Ok((modifier_ent, target, templ_ref)) = modifiers_query.get(world, modifier_ent) else {
            continue;
        };
        if target.0 != target_ent {
            continue;
        }
        let values = modifier_values(
            world,
            modifier_ent,
            templ_ref,
            base_values_query,
            curr_values_query,
        );
        if modifier_has_marker::<HitpointsCapacity>(
            modifier_ent,
            templ_ref,
            &hp_capacity_query.query(world),
        ) {
            stats.hp_capacity.add(values.base, values.effective);
        }
        if modifier_has_marker::<HitpointRegenRate>(
            modifier_ent,
            templ_ref,
            &hp_regen_query.query(world),
        ) {
            stats.hp_regen.add(values.base, values.effective);
        }
        if modifier_has_marker::<BloodCapacity>(
            modifier_ent,
            templ_ref,
            &blood_capacity_query.query(world),
        ) {
            stats.blood_capacity.add(values.base, values.effective);
        }
        if modifier_has_marker::<BleedRate>(
            modifier_ent,
            templ_ref,
            &bleed_rate_query.query(world),
        ) {
            stats.bleed_rate.add(values.base, values.effective);
        }
        if modifier_has_marker::<PainSensitivity>(
            modifier_ent,
            templ_ref,
            &pain_sensitivity_query.query(world),
        ) {
            stats.pain_sensitivity.add(values.base, values.effective);
        }
        if modifier_has_marker::<Vision>(
            modifier_ent,
            templ_ref,
            &vision_query.query(world),
        ) {
            stats.vision.add(values.base, values.effective);
        }
        if modifier_has_marker::<ManipulationDexterity>(
            modifier_ent,
            templ_ref,
            &manip_dex_query.query(world),
        ) {
            stats.manip_dex.add(values.base, values.effective);
        }
        if modifier_has_marker::<ManipulationStrength>(
            modifier_ent,
            templ_ref,
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

fn format_resolved_f32(label: &str, entity_value: Option<f32>, templ_value: Option<f32>) -> String {
    if let Some(value) = entity_value {
        return format!("{}: {:.3} (anim's)", label, value);
    }
    if let Some(value) = templ_value {
        return format!("{}: {:.3} (SpriteConfig's)", label, value);
    }
    format!("{}: missing", label)
}

fn format_resolved_acz(
    anim_value: Option<f32>,
    templ_value: Option<f32>,
    add_up_anim_and_sc_acz: bool,
) -> String {
    match (anim_value, templ_value, add_up_anim_and_sc_acz) {
        (Some(anim), Some(templ), true) => format!("AcZ: {:.3} (anim+SpriteConfig)", anim + templ),
        (Some(anim), _, _) => format!("AcZ: {:.3} (anim)", anim),
        (None, Some(templ), _) => format!("AcZ: {:.3} (SpriteConfig)", templ),
        (None, None, _) => "AcZ: missing".to_string(),
    }
}

fn render_held_sprite_entry(
    ui: &mut egui::Ui,
    world: &mut World,
    sprite_ent: Entity,
) {
    let highlighted_sprite_original_color = world
        .get_entity_mut(sprite_ent)
        .ok()
        .and_then(|mut sprite_entity| {
            let Some(mut sprite) = sprite_entity.get_mut::<Sprite>() else {
                return None;
            };
            let original_color = sprite.color;
            sprite.color = Color::srgb(0.0, 1.0, 1.0);
            Some(original_color)
        });

    let Some((templ_ref, templ_display_name, templ_str_id, sprite_display_name, sprite_str_id, templ_add_up_anim_and_sc_acz, sprite_visibility, sprite_acz, sprite_y_sort_origin, sprite_transform, sprite_global_transform, sprite, base_holder_ref, templ_acz, templ_y_sort_origin, template_flags, template_detail_lines)) = (|| {
        let Ok(sprite_ref) = world.get_entity(sprite_ent) else {
            ui.label(format!("{:?}: missing entity", sprite_ent));
            return None;
        };

        let Some(templ_ref) = sprite_ref.get::<TemplEntiRef>().copied() else {
            ui.label(format!("{:?}: TemplEntiRef missing", sprite_ent));
            return None;
        };

        let Ok(templ_ref_entity) = world.get_entity(templ_ref.0) else {
            ui.label(format!("{:?}: TemplEntiRef.0 entity missing", sprite_ent));
            return None;
        };

        let mut template_flags = Vec::with_capacity(8);
        if templ_ref_entity.get::<SpriteConfig>().is_some() {
            template_flags.push("SpriteConfig");
        }
        if templ_ref_entity.get::<MovementBased>().is_some() {
            template_flags.push("MovementBased");
        }
        if templ_ref_entity.get::<GroundingBased>().is_some() {
            template_flags.push("GroundingBased");
        }
        if templ_ref_entity.get::<UseFallbackSprite>().is_some() {
            template_flags.push("UseFallbackSprite");
        }
        if templ_ref_entity.get::<Exclusive>().is_some() {
            template_flags.push("Exclusive");
        }
        if templ_ref_entity.get::<ExcludedFromBaseAnimPickingSystem>().is_some() {
            template_flags.push("ExcludedFromBaseAnimPickingSystem");
        }
        if templ_ref_entity.get::<ExcludedFromNormalSizeModifier>().is_some() {
            template_flags.push("ExcludedFromNormalSizeModifier");
        }

        let mut template_detail_lines = Vec::with_capacity(8);
        if let Some(base_movement_speed) = templ_ref_entity.get::<BaseMovementSpeed>() {
            template_detail_lines.push(format!("BaseMovementSpeed: {:.3}", base_movement_speed.0));
        }
        if let Some(flip_horiz_if_dir) = templ_ref_entity.get::<FlipHorizIfDir>() {
            template_detail_lines.push(format!("FlipHorizIfDir: {:?}", flip_horiz_if_dir));
        }
        if let Some(color_holder) = templ_ref_entity.get::<ColorHolder>() {
            template_detail_lines.push(format!("ColorHolder: {:?}", color_holder.0));
        }
        if let Some(become_child_of_sprite_with_tag) = templ_ref_entity.get::<BecomeChildOfSpriteWithTag>() {
            template_detail_lines.push(format!("BecomeChildOfSpriteWithTag: {:?}", become_child_of_sprite_with_tag.0));
        }
        if let Some(offset_for_children) = templ_ref_entity.get::<OffsetForChildren>() {
            template_detail_lines.push(format!("OffsetForChildren: {} tags", offset_for_children.0.len()));
        }
        if let Some(mapped_anims) = templ_ref_entity.get::<MappedAnimations>() {
            template_detail_lines.push(format!("MappedAnimations: {} entries", mapped_anims.0.len()));
        }
        if let Some(sprite_anim_sfx) = templ_ref_entity.get::<SpriteAnimSfx>() {
            template_detail_lines.push(format!("SpriteAnimSfx: {} paths, every_n_frame_changes {:.3}", sprite_anim_sfx.sound_paths.len(), sprite_anim_sfx.every_n_frame_changes));
        }
        if let Some(sprite_loop_sfx) = templ_ref_entity.get::<SpriteLoopSfx>() {
            template_detail_lines.push(format!("SpriteLoopSfx: {} paths, condition {:?}", sprite_loop_sfx.sound_paths.len(), sprite_loop_sfx.condition));
        }
        if let Some(sprite_timed_sfx) = templ_ref_entity.get::<SpriteTimedSfx>() {
            template_detail_lines.push(format!("SpriteTimedSfx: {} paths, condition {:?}, interval {:.3}, scale_with_animation_speed {}", sprite_timed_sfx.sound_paths.len(), sprite_timed_sfx.condition, sprite_timed_sfx.time_interval_secs, sprite_timed_sfx.scale_interval_with_animation_speed));
        }

        Some((
            templ_ref,
            templ_ref_entity.get::<DisplayName>().cloned(),
            templ_ref_entity.get::<StrId>().cloned(),
            templ_ref_entity.get::<DisplayName>().cloned(),
            templ_ref_entity.get::<StrId>().cloned(),
            templ_ref_entity.get::<AddUpAnimAndScAcZ>().is_some(),
            sprite_ref.get::<Visibility>().copied(),
            sprite_ref.get::<AcZ>().copied().map(|value| value.0),
            sprite_ref.get::<YSortOrigin>().copied().map(|value| value.0),
            sprite_ref.get::<Transform>().cloned(),
            sprite_ref.get::<GlobalTransform>().copied(),
            sprite_ref.get::<Sprite>().cloned(),
            sprite_ref.get::<BaseHolderRef>().copied(),
            templ_ref_entity.get::<AcZ>().copied().map(|value| value.0),
            templ_ref_entity.get::<YSortOrigin>().copied().map(|value| value.0),
            template_flags,
            template_detail_lines,
        ))
    })() else {
        return;
    };

    let world_ptr = world as *mut World;

    let sprite_label = part_label(
        sprite_ent,
        sprite_display_name.as_ref(),
        sprite_str_id.as_ref(),
    );

    egui::CollapsingHeader::new(sprite_label)
        .default_open(false)
        .show(ui, |ui| {
        if let Some(visibility) = sprite_visibility {
            ui.label(format!("Visibility: {:?}", visibility));
        }
        if let Some(transform) = sprite_transform {
            ui.label(format!(
                "Transform: x={:.1} y={:.1} z={}",
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ));
        }
        if let Some(transform) = sprite_global_transform {
            ui.label(format!(
                "GlobalTransform: x={:.1} y={:.1} z={}",
                transform.translation().x,
                transform.translation().y,
                transform.translation().z,
            ));
        }
        if let Some(sprite) = sprite {
            ui.label(format!("Color: {:?}", sprite.color));
            ui.label(format!("flip: x={} y={}", sprite.flip_x, sprite.flip_y));
        }
        if let Some(base_holder_ref) = base_holder_ref {
            ui.label(format!("BaseHolderRef.base: {:?}", base_holder_ref.base));
        }
        if sprite_acz.is_some() || templ_acz.is_some() {
            ui.label(format_resolved_acz(sprite_acz, templ_acz, templ_add_up_anim_and_sc_acz));
        }
        if sprite_y_sort_origin.is_some() || templ_y_sort_origin.is_some() {
            ui.label(format_resolved_f32("YSortOrigin", sprite_y_sort_origin, templ_y_sort_origin));
        }

        let templ_str_id_label = templ_str_id
            .as_ref()
            .map(|str_id| str_id.as_str())
            .unwrap_or("<no-strid>");
        let templ_display_name_label = templ_display_name
            .as_ref()
            .map(|display_name| display_name.0.clone());
        let templ_collapsing_label = if let Some(display_name) = templ_display_name_label {
            format!("SpriteCfg {} {} {}", templ_str_id_label, templ_ref.0.index(), display_name)
        } else {
            format!("SpriteCfg {} {:?}", templ_str_id_label, templ_ref.0)
        };

        ui.collapsing(templ_collapsing_label, |ui| {
            if world.get_entity(templ_ref.0).is_ok() {
                unsafe {
                    bevy_inspector::ui_for_entity(&mut *world_ptr, templ_ref.0, ui);
                }
            } else {
                ui.label("SpriteConfigRef target entity missing");
            }
        });

        if !template_flags.is_empty() {
            ui.label(format!("Template markers: {}", template_flags.join(", ")));
        }

        for line in template_detail_lines {
            ui.label(line);
        }
    });

    if let Some(original_color) = highlighted_sprite_original_color {
        if let Ok(mut sprite_entity) = world.get_entity_mut(sprite_ent) {
            if let Some(mut sprite) = sprite_entity.get_mut::<Sprite>() {
                sprite.color = original_color;
            }
        }
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

    let multi_beings_active = world
        .get_resource::<debug_shared::ClickInspectorState>()
        .is_some_and(|state| state.mult_being_windows);

    if multi_beings_active {
        let mut beings: Vec<Entity> = selected_entities.selected_beings.iter().copied().collect();
        if beings.is_empty() {
            if let Some(selected_being_entity) = selected_entities.selected_being.or(selected_entities.selected_exempted_entity) {
                beings.push(selected_being_entity);
            }
        }
        beings.sort_unstable_by_key(|entity| entity.index());

        if !beings.is_empty() {
            let mut egui_context_query = world.query_filtered::<
                &bevy_inspector_egui::bevy_egui::EguiContext,
                With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>,
            >();
            let Some(egui_context) = egui_context_query.iter(world).next() else {
                return;
            };
            let mut egui_context = egui_context.clone();
            let screen_rect = egui_context.get_mut().content_rect();
            let world_ptr = world as *mut World;
            let mut is_open = true;
            let mut remove_being_requested = None;
            let mut clear_all_requested = false;

            egui::Window::new("Selected Being Details")
                .default_width(980.0)
                .default_height(620.0)
                .default_pos([screen_rect.right() - 1000.0, screen_rect.top() + 10.0])
                .open(&mut is_open)
                .vscroll(true)
                .show(egui_context.get_mut(), |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Being Details");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear all selections").clicked() {
                                clear_all_requested = true;
                            }
                        });
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.columns(beings.len(), |columns| {
                            for (column, entity) in columns.iter_mut().zip(beings.iter().copied()) {
                                column.push_id(entity, |ui| {
                                    render_multi_being_details_column(ui, world, world_ptr, entity, &mut remove_being_requested);
                                });
                            }
                        });
                    });
                });

            if clear_all_requested {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_beings.clear();
                    selected_entities.selected_being = None;
                    selected_entities.selected_exempted_entity = None;
                }
            }

            if let Some(being_to_remove) = remove_being_requested {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_beings.remove(&being_to_remove);
                    if selected_entities.selected_being == Some(being_to_remove) {
                        selected_entities.selected_being = selected_entities.selected_beings.iter().next().copied();
                    }
                    if selected_entities.selected_beings.is_empty() {
                        selected_entities.selected_being = None;
                    }
                }
            }

            if !is_open {
                if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
                    window_visible.being_details = false;
                }
            }
            return;
        }
    }

    let Some(selected_being_entity) = selected_entities.selected_being.or(selected_entities.selected_exempted_entity) else {
        return;
    };

    let mut selected_part = selected_entities.selected_being_bodypart;
    let mut selected_interaction_zone = selected_entities.selected_being_interaction_zone;

    let mut egui_context_query = world.query_filtered::<
        &bevy_inspector_egui::bevy_egui::EguiContext,
        With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>,
    >();
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let mut body_query = world.query::<&HeldBody>();
    let mut body_sums_query = world.query::<&BodySums>();
    let mut display_name_query = world.query::<&DisplayName>();
    let mut str_id_query = world.query::<&StrId>();
    let mut bodyparts_query = world.query::<&BodypartChildrenBodyparts>();
    let mut bodypart_damage_query = world.query::<&AccuDamage>();
    let mut held_items_query = world.query::<&HeldItems>();
    let mut held_sprites_query = world.query::<&HeldSprites>();
    let mut slot_holder_query = world.query::<&SlottedItemHolder>();
    let mut norm_move_dir_query = world.query::<&FinalNormMoveDir>();
    let mut speed_potential_query = world.query::<&SpeedPotential>();
    let mut speed_magnitude_query = world.query::<&SpeedMagnitude>();
    let mut input_move_dir_query = world.query::<&InputMoveDir>();
    let mut computed_by_query = world.query::<&ComputedBy>();
    let mut computed_locally_query = world.query::<&ComputedLocally>();
    let mut bit_ref_query = world.query::<&BitRef>();
    let mut race_ref_query = world.query::<&RaceRef>();
    let mut interaction_zones_query = world.query::<&InteractionZones>();
    let mut player_actions_query =
        world.query_filtered::<&Actions<BeingDirectControlInputContext>, MyPlayer>();
    let mut player_move_action_query = world.query::<&Action<DcWasdAction>>();
    let mut grid_move_query = world.query::<&GridLockedMovement>();
    let mut gpos_query = world.query::<&tilemap_shared::GlobalTilePos>();
    let mut facing_query = world.query::<&CardinalDirection>();
    let mut wander_state_query = world.query::<&WanderState>();
    let mut wander_state_entity_query = world.query::<(Entity, &WanderState)>();
    let mut templ_refs_query = world.query::<&TemplEntiRef>();
    let mut applied_mods_query = world.query::<&AppliedModifiers>();
    let mut modifiers_query =
        world.query_filtered::<(Entity, &ModifierTarget, Option<&TemplEntiRef>), Without<Templ>>();
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
            Option<&TemplEntiRef>,
            Option<&ModifierTags>,
            Has<PainSlowdown>,
        ),
        (With<WalkStrength>, Without<Templ>),
    >();
    let mut body_energy_store_query = world.query::<&BodyEnergyStore>();
    let mut body_energy_balance_query = world.query::<&BodyEnergyBalance>();
    let mut body_energy_profile_query = world.query::<&BodyEnergyProfile>();
    let mut starvation_config_query = world.query::<&StarvationConfig>();
    let mut body_condition_query = world.query::<&BodyCondition>();
    let mut body_strength_scale_query = world.query::<&BodyStrengthScale>();
    let mut body_weight_sum_query = world.query::<&BodyWeightSum>();
    let mut predator_query = world.query::<&Predator>();
    let mut predator_cfg_query = world.query::<&PredatorCfg>();
    let mut go_to_query = world.query::<&GoTo>();
    let mut chasing_query = world.query::<&NavChasing>();
    let mut fleeing_query = world.query::<&Fleeing>();
    let mut lod_level_query = world.query::<&LodLevel>();
    let mut behavorial_nav_state_query = world.query::<Has<BehavorialNavState>>();
    let mut retained_chase_path_snapshot_query = world.query::<&RetainedChasePathSnapshot>();
    let mut dimension_query = world.query_filtered::<(Entity, &HashId, Option<&Name>), With<Dimension>>();
    let mut dimension_ref_query = world.query::<&DimensionRef>();

    let body = body_query.get(world, selected_being_entity).ok();
    let body_available = body.is_some();
    let body_entity = body.map_or(selected_being_entity, |body| body.entity());
    let bit_map = world.get_resource::<BeingInstTemplateEntityMap>();
    let bit_ref_ent = bit_ref_query
        .get(world, selected_being_entity)
        .ok()
        .and_then(|bit_ref| bit_map.and_then(|map| map.0.get_cloned(bit_ref.0).ok()));
    let race_ref_ent = race_ref_query
        .get(world, selected_being_entity)
        .ok()
        .and_then(|race_ref| {
            world
                .get_resource::<RaceEntityMap>()
                .and_then(|map| map.0.get_cloned(race_ref.0).ok())
        });
    let body_label = if body_available {
        part_label(
            body_entity,
            display_name_query.get(world, body_entity).ok(),
            str_id_query.get(world, body_entity).ok(),
        )
    } else {
        format!("Missing body for {:?}", selected_being_entity)
    };
    let body_sums = body_available.then(|| body_sums_query.get(world, body_entity).ok().cloned()).flatten();
    let body_templ_ref = body_available.then(|| templ_refs_query.get(world, body_entity).ok().copied()).flatten();
    let current_dim_ref = dimension_ref_query.get(world, selected_being_entity).ok().copied();
    let current_gpos = gpos_query.get(world, selected_being_entity).ok().copied();

    let mut dimensions_by_label: BTreeMap<String, (DimensionRef, Entity)> = BTreeMap::default();
    for (dimension_ent, hash_id, name) in dimension_query.iter(world) {
        let dim_ref = DimensionRef(*hash_id);
        let label = format!("{} ({:?})", dimension_label(name, dim_ref), dim_ref);
        dimensions_by_label.insert(label, (dim_ref, dimension_ent));
    }

    let mut inventory_holders = vec![(
        selected_being_entity,
        part_label(
            selected_being_entity,
            display_name_query.get(world, selected_being_entity).ok(),
            str_id_query.get(world, selected_being_entity).ok(),
        ),
    )];
    if body_available && body_entity != selected_being_entity {
        inventory_holders.push((body_entity, body_label.clone()));
    }

    let mut part_infos: Vec<(Entity, String)> = Vec::new();
    if body_available && let Ok(parts) = bodyparts_query.get(world, body_entity) {
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
    if selected_part.is_none() || !part_infos.iter().any(|(entity, _)| Some(*entity) == selected_part) {
        selected_part = part_infos.first().map(|(entity, _)| *entity);
    }

    let mut clear_selection = false;
    let mut is_open = true;
    let world_ptr = world as *mut World;

    egui::Window::new("Being Debug Window (bdw)")
        .default_width(700.0)
        .default_height(560.0)
        .default_pos([screen_rect.right() - 720.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.heading(format!("Being Entity: {:?}", selected_being_entity));
            ui.collapsing("All Entity Components", |ui| {
                if world.get_entity(selected_being_entity).is_ok() {
                    unsafe {
                        bevy_inspector::ui_for_entity(&mut *world_ptr, selected_being_entity, ui);
                    }
                } else {
                    ui.label("Selected being entity missing");
                }
            });
            ui.separator();

            ui.horizontal(|ui| {
                let wallphaser_enabled = world.get::<WallPhaser>(selected_being_entity).is_some();
                if ui.button(if wallphaser_enabled { "Disable WallPhaser" } else { "Enable WallPhaser" }).clicked() {
                    unsafe {
                        let world = &mut *world_ptr;
                        if wallphaser_enabled {
                            world.entity_mut(selected_being_entity).remove::<WallPhaser>();
                        } else {
                            world.entity_mut(selected_being_entity).insert(WallPhaser);
                        }
                    }
                }
                if ui.button("Clear Selection").clicked() {
                    clear_selection = true;
                }
            });
            ui.separator();

            let mut pending_dimension_change: Option<(DimensionRef, Entity)> = None;
            let mut pending_teleport_gpos: Option<GlobalTilePos> = None;
            let mut pending_teleport_error: Option<String> = None;

            {
                let world = unsafe { &mut *world_ptr };
                let mut location_editor_state = world.resource_mut::<DebugBeingLocationEditorState>();
                if location_editor_state.last_selected_being_entity != Some(selected_being_entity) {
                    location_editor_state.last_selected_being_entity = Some(selected_being_entity);
                    location_editor_state.teleport_error = None;
                    if let Some(current_gpos) = current_gpos {
                        location_editor_state.gpos_x_text = current_gpos.0.x.to_string();
                        location_editor_state.gpos_y_text = current_gpos.0.y.to_string();
                    } else {
                        location_editor_state.gpos_x_text.clear();
                        location_editor_state.gpos_y_text.clear();
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Dimension:");
                    if dimensions_by_label.is_empty() {
                        ui.label("No dimensions are currently loaded.");
                        return;
                    }

                    let current_dimension_label = current_dim_ref
                        .and_then(|current_dim_ref| {
                            dimensions_by_label.iter().find_map(|(label, (dim_ref, _))| {
                                (*dim_ref == current_dim_ref).then_some(label.clone())
                            })
                        })
                        .unwrap_or_else(|| {
                            current_dim_ref
                                .map_or_else(|| "Missing DimensionRef".to_string(), |dim_ref| format!("{:?}", dim_ref))
                        });

                    egui::ComboBox::from_id_salt(("bdw_dimension_combo", selected_being_entity))
                        .selected_text(current_dimension_label)
                        .show_ui(ui, |ui| {
                            for (label, (dim_ref, dimension_ent)) in dimensions_by_label.iter() {
                                let is_current = current_dim_ref.is_some_and(|current_dim_ref| current_dim_ref == *dim_ref);
                                if ui.selectable_label(is_current, label).clicked() {
                                    pending_dimension_change = Some((*dim_ref, *dimension_ent));
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("GPos:");
                    ui.add_sized(
                        [80.0, 0.0],
                        egui::TextEdit::singleline(&mut location_editor_state.gpos_x_text),
                    );
                    ui.add_sized(
                        [80.0, 0.0],
                        egui::TextEdit::singleline(&mut location_editor_state.gpos_y_text),
                    );
                    if ui.button("Teleport").clicked() {
                        let parsed_x = location_editor_state.gpos_x_text.trim().parse::<i32>();
                        let parsed_y = location_editor_state.gpos_y_text.trim().parse::<i32>();
                        match (parsed_x, parsed_y) {
                            (Ok(x), Ok(y)) => {
                                pending_teleport_gpos = Some(GlobalTilePos::new(x, y));
                                location_editor_state.teleport_error = None;
                            }
                            _ => {
                                pending_teleport_error = Some("GPos must be valid integers for both x and y.".to_string());
                            }
                        }
                    }
                });

                if let Some(error) = &location_editor_state.teleport_error {
                    ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                }
            }

                if let Some((dim_ref, dimension_ent)) = pending_dimension_change {
                    let route_via_client = world
                        .get_resource::<DebugUiConfig>()
                        .is_some_and(|cfg| cfg.client_debug)
                        && world.get_resource::<State<ClientState>>().is_some_and(|state| *state.get() == ClientState::Connected);
                    if route_via_client {
                        world
                            .resource_mut::<Messages<ClientDebugSetBeingDimensionRequest>>()
                            .write(ClientDebugSetBeingDimensionRequest {
                                being_ent: selected_being_entity,
                                dim_ref,
                                dimension_ent,
                            });
                    } else {
                        unsafe {
                            let world = &mut *world_ptr;
                            world.entity_mut(selected_being_entity).insert((dim_ref, ChildOf(dimension_ent)));
                        }
                    }
                }

                if let Some(new_gpos) = pending_teleport_gpos {
                    let route_via_client = world
                        .get_resource::<DebugUiConfig>()
                        .is_some_and(|cfg| cfg.client_debug)
                        && world.get_resource::<State<ClientState>>().is_some_and(|state| *state.get() == ClientState::Connected);
                    if route_via_client {
                        world
                            .resource_mut::<Messages<ClientDebugTeleportBeingRequest>>()
                            .write(ClientDebugTeleportBeingRequest {
                                being_ent: selected_being_entity,
                                gpos: new_gpos,
                            });
                    } else {
                        let z = unsafe { (&mut *world_ptr).get::<Transform>(selected_being_entity) }
                            .map(|transform| transform.translation.z)
                            .unwrap_or_default();
                        let new_transform = Transform::from_translation(new_gpos.to_translation(z));
                        unsafe {
                            let world = &mut *world_ptr;
                            world.entity_mut(selected_being_entity).insert((
                                new_gpos,
                                new_transform,
                                GridLockedMovement::default(),
                                GridLockedMovementVisual::default(),
                            ));
                            let mut location_editor_state = world.resource_mut::<DebugBeingLocationEditorState>();
                            location_editor_state.gpos_x_text = new_gpos.0.x.to_string();
                            location_editor_state.gpos_y_text = new_gpos.0.y.to_string();
                            location_editor_state.teleport_error = None;
                        }
                    }
                }

            if let Some(error) = pending_teleport_error {
                unsafe {
                    let world = &mut *world_ptr;
                    world.resource_mut::<DebugBeingLocationEditorState>().teleport_error = Some(error);
                }
            }

            ui.separator();

            ui.heading("Body");
            ui.label(format!("Body: {} [{:?}]", body_label, body_entity));
            if !body_available {
                ui.label("HeldBody: missing on this client.");
            }
            ui.label(format!(
                "Body templ ref: {}",
                body_templ_ref.map_or("missing".to_string(), |refe| format!("{:?}", refe.0))
            ));
            if let Some(sums) = body_sums {
                render_body_vitals_controls(ui, unsafe { &mut *world_ptr }, selected_being_entity, body_entity, &sums);
                ui.label(format!("Bleed rate: {:.2}", sums.bleed_rate));
                ui.label(format!("Consciousness: {:.2}", sums.consciousness));
                ui.label(format!("Pain: {:.2}", sums.pain));
                ui.label(format!("Vision: {:.2}", sums.vision));
                ui.label(format!("Manip dex: {:.2}", sums.manip_dex));
                ui.label(format!("Manip str: {:.2}", sums.manip_str));
            }
            ui.separator();

            ui.collapsing("Body Energy", |ui| {
                let body_energy_store = body_energy_store_query.get(world, body_entity).ok().copied();
                let body_energy_balance = body_energy_balance_query.get(world, body_entity).ok().copied();
                let body_energy_profile = body_templ_ref.and_then(|body_templ_ref| {
                    body_energy_profile_query.get(world, body_templ_ref.0).ok().copied()
                });
                let starvation_config = body_templ_ref.and_then(|body_templ_ref| {
                    starvation_config_query.get(world, body_templ_ref.0).ok().copied()
                });
                let body_condition = body_condition_query.get(world, selected_being_entity).ok().copied();
                let body_strength_scale = body_strength_scale_query.get(world, selected_being_entity).ok().copied();
                let body_weight_sum = body_weight_sum_query.get(world, selected_being_entity).ok().copied();
                let predator = predator_query.get(world, selected_being_entity).is_ok();
                let predator_cfg = predator_cfg_query.get(world, selected_being_entity).ok().cloned().or_else(|| {
                    race_ref_ent.and_then(|race_ent| predator_cfg_query.get(world, race_ent).ok().cloned())
                });

                if let Some(body_energy_store) = body_energy_store {
                    ui.label(format!("Baseline lean mass: {:.2} kg", body_energy_store.baseline_mass_kg));
                    ui.label(format!("Lean mass: {:.2} kg", body_energy_store.lean_mass_kg));
                    ui.label(format!("Fat mass: {:.2} kg", body_energy_store.fat_mass_kg));
                    ui.label(format!("Stomach: {:.2} kcal", body_energy_store.stomach_kcal));
                    ui.label(format!("Burn: {:.3} kcal/s", body_energy_store.burn_kcal_per_sec));
                    ui.label(format!("Activity multiplier: {:.3}", body_energy_store.activity_multiplier));
                    ui.label(format!("Thermal multiplier: {:.3}", body_energy_store.thermal_multiplier));
                } else {
                    ui.label("BodyEnergyStore: missing");
                }

                if let Some(body_energy_balance) = body_energy_balance {
                    ui.label(format!("Last tick net kcal: {:.3}", body_energy_balance.last_tick_net_kcal));
                    ui.label(format!("Unresolved deficit kcal: {:.3}", body_energy_balance.unresolved_deficit_kcal));
                } else {
                    ui.label("BodyEnergyBalance: missing");
                }

                if let Some(body_energy_profile) = body_energy_profile {
                    ui.label(format!("Burn rate multiplier: {:.3}", body_energy_profile.burn_rate_multiplier));
                    ui.label(format!("Wasting rate multiplier: {:.3}", body_energy_profile.wasting_rate_multiplier));
                    ui.label(format!("Healthy fat capacity multiplier: {:.3}", body_energy_profile.healthy_fat_capacity_multiplier));
                } else {
                    ui.label("BodyEnergyProfile: missing");
                }

                if let Some(starvation_config) = starvation_config {
                    ui.label(format!("Max fat mobilization: {:.3} kcal/s", starvation_config.max_fat_mobilization_kcal_per_sec));
                    ui.label(format!("Max lean catabolism: {:.3} kcal/s", starvation_config.max_lean_catabolism_kcal_per_sec));
                    ui.label(format!("Damage at zero lean: {:.3} hp/s", starvation_config.damage_per_sec_at_zero_lean));
                } else {
                    ui.label("StarvationConfig: missing");
                }

                if let Some(body_condition) = body_condition {
                    ui.label(format!("Hunger ratio: {:.3}", body_condition.hunger_ratio));
                    ui.label(format!("Wasting: {:.3}", body_condition.wasting));
                    ui.label(format!("Obesity: {:.3}", body_condition.obesity));
                } else {
                    ui.label("BodyCondition: missing");
                }

                if let Some(body_strength_scale) = body_strength_scale {
                    ui.label(format!("BodyStrengthScale: {:.3}", body_strength_scale.0));
                } else {
                    ui.label("BodyStrengthScale: missing");
                }

                if let Ok(speed_potential) = speed_potential_query.get(world, selected_being_entity) {
                    ui.label(format!("SpeedPotential: {:.2}", speed_potential.0));
                }

                if let Some(body_weight_sum) = body_weight_sum {
                    ui.label(format!("BodyWeightSum: {:.2}", body_weight_sum.0));
                } else {
                    ui.label("BodyWeightSum: missing");
                }

                ui.label(format!("Predator: {}", predator));
                if let Some(predator_cfg) = predator_cfg {
                    ui.label(format!("PredatorCfg.min_hunger_to_hunt: {:.3}", predator_cfg.min_hunger_to_hunt));
                    ui.label(format!("PredatorCfg.min_hp_ratio_to_hunt: {:.3}", predator_cfg.min_hp_ratio_to_hunt));
                } else {
                    ui.label("PredatorCfg: missing on being and race");
                }
            });
            ui.separator();

            ui.collapsing("Race", |ui| {
                let race_ref = race_ref_query.get(world, selected_being_entity).ok().copied();
                let Some(race_ref) = race_ref else {
                    ui.label("RaceRef: missing");
                    return;
                };
                ui.label(format!("RaceRef.0 (hash): {:?}", race_ref.0));

                let Some(race_map) = world.get_resource::<RaceEntityMap>() else {
                    ui.label("RaceEntityMap: missing resource");
                    return;
                };
                let Ok(race_ent) = race_map.0.get_cloned(race_ref.0) else {
                    ui.label("RaceRef.0 is not present in RaceEntityMap");
                    return;
                };
                ui.label(format!("Race entity: {:?}", race_ent));
                ui.label(format!(
                    "Race StrId: {}",
                    str_id_query
                        .get(world, race_ent)
                        .map_or_else(|_| "missing".to_string(), |str_id| str_id.to_string())
                ));
                ui.label(format!(
                    "Race DisplayName: {}",
                    display_name_query
                        .get(world, race_ent)
                        .map_or_else(|_| "missing".to_string(), |display_name| display_name.0.clone())
                ));
                ui.separator();
                ui.label("Race template components:");
                unsafe {
                    bevy_inspector::ui_for_entity(&mut *world_ptr, race_ent, ui);
                }
            });
            ui.separator();

            egui::CollapsingHeader::new("Movement Details")
                .default_open(true)
                .show(ui, |ui| {
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
                if let Ok(speed_potential) = speed_potential_query.get(world, selected_being_entity) {
                    ui.label(format!("SpeedPotential: {:.2}", speed_potential.0));
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
                if let Some(input_speed_throttle_mult) =
                    world.get::<InputSpeedThrottleMult>(selected_being_entity)
                {
                    ui.label(format!("InputSpeedThrottleMult: {:.2}", input_speed_throttle_mult.0));
                }
                if let Some(input_max_speed) = world.get::<InputMaxSpeed>(selected_being_entity) {
                    ui.label(format!("InputMaxSpeed: {:.2}", input_max_speed.0));
                }
                if grid_move_query.get(world, selected_being_entity).is_ok() {
                    if let Ok(gpos) = gpos_query.get(world, selected_being_entity) {
                        ui.label(format!("GlobalTilePos: [{}, {}]", gpos.0.x, gpos.0.y));
                    }
                }
                if let Ok(facing) = facing_query.get(world, selected_being_entity) {
                    ui.label(format!("Facing: {:?}", facing));
                }
                ui.collapsing("WanderState", |ui| {
                    match wander_state_query.get(world, selected_being_entity) {
                        Ok(wander_state) => {
                            if wander_state.is_meeting() {
                                let meeting_role = if wander_state.meeting_anchor() == Some(selected_being_entity) {
                                    "Leader"
                                } else {
                                    "Subordinate"
                                };
                                ui.label(format!("Meeting Role: {meeting_role}"));
                                let leader_ent = wander_state
                                    .meeting_anchor()
                                    .unwrap_or(selected_being_entity);
                                if let Ok(&leader_gpos) = gpos_query.get(world, leader_ent) {
                                    let mut reserved_targets = Vec::new();
                                    let all_wanderers = wander_state_entity_query.iter(world);
                                    reserved_targets.reserve(all_wanderers.size_hint().1.unwrap_or(all_wanderers.size_hint().0));
                                    for (being_ent, other_wander_state, ) in all_wanderers {
                                        if !other_wander_state.is_meeting() {
                                            continue;
                                        }
                                        if other_wander_state.meeting_anchor() != Some(leader_ent) {
                                            continue;
                                        }
                                        if let Some(target) = other_wander_state.meeting_target() {
                                            reserved_targets.push(target);
                                        } else if being_ent == leader_ent {
                                            reserved_targets.push(leader_gpos);
                                        }
                                    }
                                    if reserved_targets.is_empty() {
                                        reserved_targets.push(leader_gpos);
                                    }
                                    ui.label("Meeting Slots (relative to leader):");
                                    paint_meeting_slots_preview(ui, leader_gpos, &reserved_targets);
                                } else {
                                    ui.label("Meeting Slots: leader gpos missing");
                                }
                            }
                            ui.label(format!("{wander_state:#?}"));
                        }
                        Err(_) => {
                            ui.label("WanderState: missing");
                        }
                    };
                });
                let facing = facing_query
                    .get(world, selected_being_entity)
                    .copied()
                    .unwrap_or_default();
                ui.separator();
                match resolve_interaction_zones_with_source(
                    world,
                    selected_being_entity,
                    bit_ref_ent,
                    race_ref_ent,
                    &mut interaction_zones_query,
                ) {
                    Some((interaction_zones, source_kind, source_ent)) => {
                        if source_kind == "being" {
                            ui.label(format!("Collision Mask source: being [{:?}]", source_ent));
                        } else {
                            ui.label(format!(
                                "Collision Mask source: {} fallback [{:?}]",
                                source_kind, source_ent
                            ));
                        }
                        if let Some(collision_zone) = interaction_zones.get_collision_mask() {
                            paint_collision_mask_preview(ui, collision_zone, facing);
                        } else {
                            warn_once!(
                                target: DEBUG,
                                "Being {:?} resolved InteractionZones from {} {:?} but collision_mask zone is missing",
                                selected_being_entity,
                                source_kind,
                                source_ent
                            );
                            ui.label("Collision Mask: missing collision_mask zone");
                        }
                    }
                    None => {
                        warn_once!(
                            target: DEBUG,
                            "Being {:?} has no InteractionZones on being/bit/race fallback chain",
                            selected_being_entity
                        );
                        ui.label("Collision Mask: missing on being/bit/race");
                    }
                }

                let mut walk_add: f32 = 0.0;
                let mut walk_mul: f32 = 1.0;
                let mut walk_min: f32 = 0.0;
                let mut walk_max: f32 = f32::INFINITY;
                let mut walk_rows = Vec::new();
                let mut walk_effects = EntityHashSet::default();
                let applied_mods = applied_mods_query.query(world);
                let being_templ_ref = templ_refs_query.get(world, selected_being_entity).ok();
                collect_applied_modifier_entities(
                    &mut walk_effects,
                    selected_being_entity,
                    being_templ_ref,
                    &applied_mods,
                );
                collect_applied_modifier_entities(
                    &mut walk_effects,
                    body_entity,
                    body_templ_ref.as_ref(),
                    &applied_mods,
                );
                if let Ok(bodyparts) = bodyparts_query.get(world, body_entity) {
                    for bodypart_ent in bodyparts.iter() {
                        let part_templ_ref = templ_refs_query.get(world, bodypart_ent).ok();
                        collect_applied_modifier_entities(
                            &mut walk_effects,
                            bodypart_ent,
                            part_templ_ref,
                            &applied_mods,
                        );
                    }
                }
                for effect in walk_effects.iter() {
                    let Ok((modifier_ent, target, child_of, templ_ref, tagset, pain_slowdown)) =
                        walk_modifiers_query.get(world, *effect)
                    else {
                        continue;
                    };
                    let values = modifier_values(
                        world,
                        modifier_ent,
                        templ_ref,
                        &mut base_values_query,
                        &mut curr_values_query,
                    );
                    let Some(op) = resolve_modifier_component(
                        modifier_ent,
                        templ_ref,
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
                        "{:?} Tgt {:?} ChOf {} {} {:?}",
                        modifier_ent,
                        target.0,
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
                    if let Some(templ_ref) = templ_ref {
                        row.push_str(&format!(" EzRef {:?}", templ_ref.0));
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
                ui.separator();

                ui.collapsing("Pathfinding", |ui| {
                    let has_nav_state = behavorial_nav_state_query
                        .get(world, selected_being_entity)
                        .unwrap_or(false);
                    let has_wander_state = wander_state_query.get(world, selected_being_entity).is_ok();
                    let chasing = chasing_query.get(world, selected_being_entity).ok();
                    let fleeing = fleeing_query.get(world, selected_being_entity).ok();
                    let go_to = go_to_query.get(world, selected_being_entity).ok();
                    let lod_level = lod_level_query.get(world, selected_being_entity).ok().copied();
                    let retained_chase_path_snapshot = retained_chase_path_snapshot_query
                        .get(world, selected_being_entity)
                        .ok();

                    let nav_mode = if chasing.is_some() {
                        "Chasing"
                    } else if fleeing.is_some() {
                        "Fleeing"
                    } else if has_wander_state {
                        "Wandering"
                    } else if has_nav_state {
                        "Nav-state only"
                    } else {
                        "Idle"
                    };

                    ui.label(format!("Derived nav mode: {nav_mode}"));
                    ui.label(format!("WanderState: {has_wander_state}"));
                    match chasing {
                        Some(chasing) => {
                            ui.label(format!(
                                "Chasing: target {:?}, stop {:.2}",
                                chasing.target,
                                chasing.stop_distance,
                            ));
                        }
                        None => {
                            ui.label("Chasing: missing");
                        }
                    }
                    match fleeing {
                        Some(fleeing) => {
                            ui.label(format!(
                                "Fleeing: threats {:?}, desired {:.2}",
                                fleeing.threats,
                                fleeing.desired_distance_tiles,
                            ));
                        }
                        None => {
                            ui.label("Fleeing: missing");
                        }
                    }
                    match go_to {
                        Some(go_to) => {
                            ui.label(format!(
                                "GoTo: pos [{}, {}], stop {:.2}, source {:?}, tick {}",
                                go_to.pos.0.x,
                                go_to.pos.0.y,
                                go_to.stop_distance,
                                go_to.source,
                                go_to.updated_tick,
                            ));
                        }
                        None => {
                            ui.label("GoTo: missing");
                        }
                    }
                    ui.label(format!(
                        "LodLevel: {}",
                        lod_level.map_or_else(|| "missing".to_string(), |lod_level| lod_level.0.to_string())
                    ));
                    ui.label(format!(
                        "RetainedChasePathSnapshot: {}",
                        retained_chase_path_snapshot
                            .map(|snapshot| format!(
                                "chunks={} trail={} last_target={:?}",
                                snapshot.chunk_positions.len(),
                                snapshot.target_chunk_trail.len(),
                                snapshot.last_target_chunk_pos,
                            ))
                            .unwrap_or_else(|| "missing".to_string())
                    ));
                });
            });
            ui.separator();

            ui.collapsing("InteractionZones", |ui| {
                let Some((interaction_zones, source_kind, source_ent)) = resolve_interaction_zones_with_source(
                    world,
                    selected_being_entity,
                    bit_ref_ent,
                    race_ref_ent,
                    &mut interaction_zones_query,
                ) else {
                    ui.label("Missing InteractionZones on being/bit/race.");
                    return;
                };
                let Ok(being_gpos) = gpos_query.get(world, selected_being_entity) else {
                    ui.label("Missing GlobalTilePos.");
                    return;
                };
                let facing = facing_query
                    .get(world, selected_being_entity)
                    .copied()
                    .unwrap_or_default();
                if source_kind == "being" {
                    ui.label(format!("Zone source: being [{:?}]", source_ent));
                } else {
                    ui.label(format!(
                        "Zone source: {} fallback [{:?}]",
                        source_kind, source_ent
                    ));
                }
                let zone_ids = interaction_zones.0.keys().copied().collect::<Vec<_>>();
                if zone_ids.is_empty() {
                    ui.label("No interaction zones.");
                    return;
                }
                if selected_interaction_zone.is_none()
                    || !interaction_zones.0.contains_key(selected_interaction_zone.unwrap())
                {
                    selected_interaction_zone = zone_ids.first().copied();
                }

                egui::ComboBox::from_label("Zone")
                    .selected_text(
                        selected_interaction_zone
                            .map(InteractionZones::interaction_zone_label)
                            .unwrap_or_else(|| "None".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for zone_id in &zone_ids {
                            ui.selectable_value(
                                &mut selected_interaction_zone,
                                Some(*zone_id),
                                InteractionZones::interaction_zone_label(*zone_id),
                            );
                        }
                    });

                let Some(zone_id) = selected_interaction_zone else {
                    ui.label("No zone selected.");
                    return;
                };
                let Some(zone) = interaction_zones.0.get_opt(zone_id) else {
                    ui.label("Selected zone missing on this being.");
                    return;
                };

                ui.label(format!(
                    "Selected zone: {}",
                    InteractionZones::interaction_zone_label(zone_id)
                ));
                ui.label(format!("Being gpos: [{}, {}]", being_gpos.0.x, being_gpos.0.y));
                ui.label(format!("Facing: {:?}", facing));
                paint_interaction_zone_preview(ui, zone, facing, *being_gpos);
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

            ui.collapsing("Held Sprites", |ui| {
                let Some((held_sprites_owner, held_sprites)) = held_sprites_query
                    .get(world, body_entity)
                    .ok()
                    .map(|held_sprites| (body_entity, held_sprites))
                    .or_else(|| {
                        held_sprites_query
                            .get(world, selected_being_entity)
                            .ok()
                            .map(|held_sprites| (selected_being_entity, held_sprites))
                    })
                else {
                    ui.label("HeldSprites: missing on body and being.");
                    return;
                };

                let owner_global_transform = world
                    .get_entity(held_sprites_owner)
                    .ok()
                    .and_then(|entity| entity.get::<GlobalTransform>())
                    .copied();
                ui.label(format!(
                    "SpriteHolder: {}",
                    owner_global_transform
                        .map(|transform| format!(
                            "x={:.1} y={:.1} z={}",
                            transform.translation().x,
                            transform.translation().y,
                            transform.translation().z,
                        ))
                        .unwrap_or_else(|| "missing".to_string())
                ));
                if held_sprites.is_empty() {
                    ui.label("HeldSprites: empty");
                    return;
                }

                let mut held_sprite_entities = Vec::with_capacity(held_sprites.len());
                for &held_sprite_ent in held_sprites {
                    held_sprite_entities.push(held_sprite_ent);
                }
                for held_sprite_ent in held_sprite_entities {
                    render_held_sprite_entry(
                        ui,
                        world,
                        held_sprite_ent,
                    );
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
                    .map_or((0.0, 0usize), |damage| (damage.total, damage.hits.len()));
                ui.label(format!("Part damage: {:.2} ({} hit(s))", part_damage.0, part_damage.1));

                let source_part_ent = templ_refs_query.get(world, selected_part_entity).ok().map(|refe| refe.0);
                ui.label(format!(
                    "Part templ ref: {}",
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
            selected_entities.selected_being_interaction_zone = None;
            selected_entities.selected_being_bodypart = None;
            selected_entities.show_full_being_components = false;
        } else {
            selected_entities.selected_being_interaction_zone = selected_interaction_zone;
            selected_entities.selected_being_bodypart = selected_part;
        }
    }

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.being_details = false;
        }
    }
}

fn render_multi_being_details_column(
    ui: &mut egui::Ui,
    world: &mut World,
    world_ptr: *mut World,
    selected_being_entity: Entity,
    remove_being_requested: &mut Option<Entity>,
) {
    let mut body_query = world.query::<&HeldBody>();
    let mut body_sums_query = world.query::<&BodySums>();
    let mut display_name_query = world.query::<&DisplayName>();
    let mut str_id_query = world.query::<&StrId>();
    let mut bodyparts_query = world.query::<&BodypartChildrenBodyparts>();
    let mut bodypart_damage_query = world.query::<&AccuDamage>();
    let mut held_items_query = world.query::<&HeldItems>();
    let mut held_sprites_query = world.query::<&HeldSprites>();
    let mut slot_holder_query = world.query::<&SlottedItemHolder>();
    let mut bit_ref_query = world.query::<&BitRef>();
    let mut race_ref_query = world.query::<&RaceRef>();
    let mut gpos_query = world.query::<&tilemap_shared::GlobalTilePos>();
    let mut facing_query = world.query::<&CardinalDirection>();
    let mut templ_refs_query = world.query::<&TemplEntiRef>();

    let body = body_query.get(world, selected_being_entity).ok();
    let body_available = body.is_some();
    let body_entity = body.map_or(selected_being_entity, |body| body.entity());
    let body_label = if body_available {
        part_label(
            body_entity,
            display_name_query.get(world, body_entity).ok(),
            str_id_query.get(world, body_entity).ok(),
        )
    } else {
        format!("Missing body for {:?}", selected_being_entity)
    };
    let body_sums = body_available.then(|| body_sums_query.get(world, body_entity).ok().cloned()).flatten();
    let body_templ_ref = body_available.then(|| templ_refs_query.get(world, body_entity).ok().copied()).flatten();

    ui.horizontal(|ui| {
        ui.heading(format!("Being: {:?}", selected_being_entity));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add_sized([24.0, 24.0], egui::Button::new(egui::RichText::new("X").strong().size(18.0))).clicked() {
                *remove_being_requested = Some(selected_being_entity);
            }
        });
    });
    ui.separator();

    ui.collapsing("Body", |ui| {
        ui.label(format!("Body: {} [{:?}]", body_label, body_entity));
        if !body_available {
            ui.label("HeldBody: missing on this client.");
        }
        ui.label(format!(
            "Body templ ref: {}",
            body_templ_ref.map_or("missing".to_string(), |refe| format!("{:?}", refe.0))
        ));
        if let Some(sums) = body_sums {
            render_body_vitals_controls(ui, unsafe { &mut *world_ptr }, selected_being_entity, body_entity, &sums);
            ui.label(format!("Bleed rate: {:.2}", sums.bleed_rate));
            ui.label(format!("Consciousness: {:.2}", sums.consciousness));
            ui.label(format!("Pain: {:.2}", sums.pain));
            ui.label(format!("Vision: {:.2}", sums.vision));
            ui.label(format!("Manip dex: {:.2}", sums.manip_dex));
            ui.label(format!("Manip str: {:.2}", sums.manip_str));
        }
    });

    ui.collapsing("Bit", |ui| {
        let bit_ref = bit_ref_query.get(world, selected_being_entity).ok().copied();
        let Some(bit_ref) = bit_ref else {
            ui.label("BitRef: missing");
            return;
        };
        ui.label(format!("BitRef.0 (hash): {:?}", bit_ref.0));
        let Some(bit_map) = world.get_resource::<BeingInstTemplateEntityMap>() else {
            ui.label("BeingInstTemplateEntityMap: missing resource");
            return;
        };
        let Ok(bit_ent) = bit_map.0.get_cloned(bit_ref.0) else {
            ui.label("BitRef.0 is not present in BeingInstTemplateEntityMap");
            return;
        };
        ui.label(format!("Bit entity: {:?}", bit_ent));
        ui.label(format!(
            "Bit StrId: {}",
            str_id_query
                .get(world, bit_ent)
                .map_or_else(|_| "missing".to_string(), |str_id| str_id.to_string())
        ));
        ui.label(format!(
            "Bit DisplayName: {}",
            display_name_query
                .get(world, bit_ent)
                .map_or_else(|_| "missing".to_string(), |display_name| display_name.0.clone())
        ));
        ui.separator();
        ui.label("Bit template components:");
        unsafe {
            bevy_inspector::ui_for_entity(&mut *world_ptr, bit_ent, ui);
        }
    });

    ui.collapsing("Race", |ui| {
        let race_ref = race_ref_query.get(world, selected_being_entity).ok().copied();
        let Some(race_ref) = race_ref else {
            ui.label("RaceRef: missing");
            return;
        };
        ui.label(format!("Race{:?}", race_ref.0));
        let Some(race_map) = world.get_resource::<RaceEntityMap>() else {
            ui.label("RaceEntityMap: missing resource");
            return;
        };
        let Ok(race_ent) = race_map.0.get_cloned(race_ref.0) else {
            ui.label("RaceRef.0 is not present in RaceEntityMap");
            return;
        };
        ui.label(format!("Race entity: {:?}", race_ent));
        ui.label(format!(
            "Race StrId: {}",
            str_id_query
                .get(world, race_ent)
                .map_or_else(|_| "missing".to_string(), |str_id| str_id.to_string())
        ));
        ui.label(format!(
            "Race DisplayName: {}",
            display_name_query
                .get(world, race_ent)
                .map_or_else(|_| "missing".to_string(), |display_name| display_name.0.clone())
        ));
        ui.separator();
        ui.label("Race template components:");
        unsafe {
            bevy_inspector::ui_for_entity(&mut *world_ptr, race_ent, ui);
        }
    });

    ui.collapsing("InteractionZones", |ui| {
        let Some(being_gpos) = gpos_query.get(world, selected_being_entity).ok() else {
            ui.label("Missing GlobalTilePos.");
            return;
        };
        let facing = facing_query
            .get(world, selected_being_entity)
            .copied()
            .unwrap_or_default();
        ui.label(format!("Being gpos: [{}, {}]", being_gpos.0.x, being_gpos.0.y));
        ui.label(format!("Facing: {:?}", facing));
        ui.label("InteractionZones details are omitted in multi mode to keep the column usable.");
    });

    ui.collapsing("Inventory", |ui| {
        for holder_entity in [selected_being_entity, body_entity] {
            let Ok(held_items) = held_items_query.get(world, holder_entity) else {
                continue;
            };
            let Ok(slot_holder) = slot_holder_query.get(world, holder_entity) else {
                continue;
            };
            let holder_label = part_label(
                holder_entity,
                display_name_query.get(world, holder_entity).ok(),
                str_id_query.get(world, holder_entity).ok(),
            );
            ui.collapsing(holder_label, |ui| {
                let held_item_entities = held_items.iter().collect::<Vec<_>>();
                ui.label(format!("Held items: {}", held_item_entities.len()));
                for item_entity in held_item_entities.iter().copied() {
                    ui.label(part_label(
                        item_entity,
                        display_name_query.get(world, item_entity).ok(),
                        str_id_query.get(world, item_entity).ok(),
                    ));
                }
                for (slot, (entities, limit)) in &slot_holder.0 {
                    ui.label(format!("{} [{}/{}]", slot, entities.len(), limit));
                }
            });
        }
    });

    ui.collapsing("Held Sprites", |ui| {
        let Some(held_sprites) = held_sprites_query
            .get(world, body_entity)
            .ok()
            .or_else(|| held_sprites_query.get(world, selected_being_entity).ok())
        else {
            ui.label("HeldSprites: missing on body and being.");
            return;
        };
        if held_sprites.is_empty() {
            ui.label("HeldSprites: empty");
            return;
        }
        let held_sprite_entities: Vec<Entity> = held_sprites.iter().collect();
        for held_sprite_ent in held_sprite_entities {
            render_held_sprite_entry(ui, world, held_sprite_ent);
        }
    });

    ui.collapsing("Body Part Stats", |ui| {
        let Some(parts) = bodyparts_query.get(world, body_entity).ok() else {
            ui.label("No body parts found.");
            return;
        };
        for part_entity in parts.iter() {
            let damage = bodypart_damage_query
                .get(world, part_entity)
                .map_or((0.0, 0usize), |damage| (damage.total, damage.hits.len()));
            ui.label(format!("{:?}: {:.2} ({} hit(s))", part_entity, damage.0, damage.1));
        }
    });
}
