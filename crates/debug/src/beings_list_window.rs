use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use ::tilemap_shared::*;
use std::collections::BTreeMap;

use being::being_components::Being;
use camera::camera_components::CameraTarget;

use crate::debug_resources::*;

#[allow(unused_parens)]
pub fn beings_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    being_query: Query<(Entity, &Being, Option<&Name>), With<Being>>,
    dimension_ref_query: Query<&DimensionRef>,
    dimension_query: Query<&Name>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
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

    // Group beings by dimension
    let mut beings_by_dimension: BTreeMap<String, Vec<(Entity, Option<&Name>)>> = BTreeMap::new();

    for (entity, _being, name) in being_query.iter() {
        if let Ok(dim_ref) = dimension_ref_query.get(entity) {
            let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
                format!("{}", n)
            } else {
                format!("{:?}", dim_ref)
            };
            beings_by_dimension
                .entry(dim_name)
                .or_insert_with(Vec::new)
                .push((entity, name));
        }
    }

    // Get camera target dimension if available
    let camera_dim_ref = camera_dimension.iter().next();

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
                if let Some(beings) = beings_by_dimension.get(dim_key) {
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
                    for (entity, name) in beings.iter() {
                        let label = if let Some(n) = name {
                            format!("{} ({:?})", n, entity)
                        } else {
                            format!("Unnamed ({:?})", entity)
                        };
                        let is_selected = selected_entities.selected_being == Some(*entity);
                        if ui.selectable_label(is_selected, label).clicked() {
                            selected_entities.selected_being = Some(*entity);
                            window_visible.being_details = true;
                        }
                    }
                });
                }
            }
        });
    window_visible.beings_list = open;
}
