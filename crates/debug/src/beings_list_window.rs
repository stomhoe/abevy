use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use common::common_components::{DisplayName, StrId};
use ::tilemap_shared::*;
use std::collections::BTreeMap;

use camera::camera_components::CameraTarget;
use ::being_shared::*;
use crate::debug_ui_helpers::direction_arrow;
use crate::debug_resources::*;

fn ref_id_label(entity: Entity, id_query: &Query<&StrId>) -> String {
    id_query
        .get(entity)
        .map(|str_id| str_id.as_str().to_string())
        .unwrap_or_else(|_| format!("{:?}", entity))
}

fn being_list_entry_label(
    display_name: Option<&DisplayName>,
    name: Option<&Name>,
    race_ref: Option<&RaceRef>,
    bit_ref: Option<&BitRef>,
    id_query: &Query<&StrId>,
) -> String {
    let mut parts = Vec::with_capacity(3);

    let main_name = display_name
        .map(|dn| dn.0.as_str())
        .or_else(|| name.map(|n| n.as_str()))
        .filter(|s| !s.is_empty() && *s != "Being");

    if let Some(n) = main_name {
        parts.push(n.to_string());
    }

    parts.push(
        race_ref
            .map(|race_ref| ref_id_label(race_ref.0, id_query))
            .unwrap_or_else(|| "-".to_string()),
    );
    parts.push(
        bit_ref
            .map(|bit_ref| ref_id_label(bit_ref.0, id_query))
            .unwrap_or_else(|| "-".to_string()),
    );
    parts.join(" | ")
}

#[allow(unused_parens)]
pub fn beings_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    being_query: Query<
        (
            Entity,
            Option<&DisplayName>,
            Option<&Name>,
            Option<&StrId>,
            Option<&RaceRef>,
            Option<&BitRef>,
            &DimensionRef,
            &GlobalTilePos,
        ),
        With<Being>,
    >,
    dimension_query: Query<&Name>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    id_query: Query<&StrId>,
) {
    if !window_visible.beings_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 350.0;
    let default_y = screen_rect.top() + 10.0;
    let mut open = window_visible.beings_list;

    let camera_info = camera_query.iter().next();
    let camera_pos = camera_info.map(|(_, transform)| transform.translation().xy());
    let camera_dim_ref = camera_info.map(|(dim_ref, _)| dim_ref);

    // Group beings by dimension
    let mut beings_by_dimension: BTreeMap<String, Vec<(Entity, String, Vec2, f32)>> = BTreeMap::new();

    for (entity, display_name, name, _, race_ref, bit_ref, dim_ref, global_pos) in being_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };
        let label = being_list_entry_label(display_name, name, race_ref, bit_ref, &id_query);
        let direction = if camera_dim_ref.map(|camera_ref| camera_ref == dim_ref).unwrap_or(false) {
            if let Some(cam_pos) = camera_pos {
                let being_pixel_pos: Vec2 = (*global_pos).into();
                being_pixel_pos - cam_pos
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };
        let distance = direction.length();
        beings_by_dimension
            .entry(dim_name)
            .or_insert_with(Vec::new)
            .push((entity, label, direction, distance));
    }

    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = beings_by_dimension.keys().cloned().collect();
    if let Some(camera_ref) = camera_dim_ref {
        if let Ok(camera_name) = dimension_query.get(camera_ref.0) {
            let camera_dim_str = format!("{}", camera_name);
            sorted_dims.sort_by(|a, b| {
                if a == &camera_dim_str { std::cmp::Ordering::Less }
                else if b == &camera_dim_str { std::cmp::Ordering::Greater }
                else { a.cmp(b) }
            });
        }
    }

    egui::Window::new("Beings List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(350.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Beings: {}", being_query.iter().count()));
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(mut beings) = beings_by_dimension.remove(dim_key) {
                    beings.sort_by(|a, b| {
                        a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let is_camera_dim = camera_dim_ref.map_or(false, |camera_ref| {
                        if let Ok(camera_name) = dimension_query.get(camera_ref.0) {
                            dim_key == &format!("{}", camera_name)
                        } else {
                            false
                        }
                    });
                    egui::CollapsingHeader::new(format!("{} ({})", dim_key, beings.len()))
                        .default_open(is_camera_dim)
                        .show(ui, |ui| {
                            for (entity, base_label, direction, distance) in beings.iter() {
                                let label = format!(
                                    "{} {} [{:?}] [{}]",
                                    base_label,
                                    direction_arrow(*direction),
                                    entity,
                                    distance.round() as i32
                                );
                                let is_selected = selected_entities.selected_being == Some(*entity);
                                if ui.selectable_label(is_selected, label).clicked() {
                                    selected_entities.selected_being = Some(*entity);
                                    selected_entities.selected_being_bodypart = None;
                                    selected_entities.show_full_being_components = false;
                                    window_visible.being_details = true;
                                }
                            }
                        });
                }
            }
        });
    window_visible.beings_list = open;
}
