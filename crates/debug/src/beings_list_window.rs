use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use ::tilemap_shared::*;
use std::collections::BTreeMap;

use being::being_components::Being;
use camera::camera_components::CameraTarget;

use crate::debug_ui_helpers::direction_arrow;
use crate::debug_resources::*;

#[allow(unused_parens)]
pub fn beings_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    being_query: Query<(Entity, Option<&Name>, &DimensionRef, &GlobalTilePos), With<Being>>,
    dimension_query: Query<&Name>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
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
    let mut beings_by_dimension: BTreeMap<String, Vec<(Entity, Option<&Name>, Vec2, f32)>> = BTreeMap::new();

    for (entity, name, dim_ref, global_pos) in being_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };
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
            .push((entity, name, direction, distance));
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
                            for (entity, name, direction, distance) in beings.iter() {
                                let label = if let Some(n) = name {
                                    format!("{} {} ({:?}) [{}]", n, direction_arrow(*direction), entity, distance.round() as i32)
                                } else {
                                    format!("Unnamed {} ({:?}) [{}]", direction_arrow(*direction), entity, distance.round() as i32)
                                };
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
