use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use std::collections::BTreeMap;

use camera::camera_components::CameraTarget;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use tilemap::tile::tile_components::{PortalTo, TileStrId};
use ::tilemap_shared::*;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn portals_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    portal_query: Query<(Entity, &DimensionRef, &GlobalTilePos, Option<&EntityZeroRef>, &PortalTo), With<PortalTo>>,
    dimension_query: Query<&Name>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    ezero_query: Query<&TileStrId, With<EntityZero>>,
    target_query: Query<Entity>,
) {
    if !window_visible.portals_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 400.0;
    let default_y = screen_rect.top() + 10.0;
    let mut open = window_visible.portals_list;

    // Get camera position and dimension
    let camera_info = camera_query.iter().next();
    let camera_pos = camera_info.map(|(_, transform)| transform.translation().xy());
    let camera_dim_ref = camera_info.map(|(dim_ref, _)| dim_ref);

    // Group portals by dimension
    let mut portals_by_dimension: BTreeMap<String, Vec<(Entity, GlobalTilePos, Option<EntityZeroRef>, Vec2, bool, f32)>> = BTreeMap::new();

    for (entity, dim_ref, global_pos, ezero_ref, portal_to) in portal_query.iter() {
        let dim_name = if let Ok(n) = dimension_query.get(dim_ref.0) {
            format!("{}", n)
        } else {
            format!("{:?}", dim_ref)
        };

        // Calculate direction vector if in same dimension
        let direction = if camera_dim_ref.map(|c| c == dim_ref).unwrap_or(false) {
            if let Some(cam_pos) = camera_pos {
                let portal_pixel_pos: Vec2 = (*global_pos).into();
                portal_pixel_pos - cam_pos
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        let distance = direction.length();

        // Check if the target entity exists
        let target_exists = target_query.get(portal_to.dest_tile).is_ok();

        portals_by_dimension
            .entry(dim_name)
            .or_insert_with(Vec::new)
            .push((entity, *global_pos, ezero_ref.copied(), direction, target_exists, distance));
    }

    // Sort dimensions with camera dimension first
    let mut sorted_dims: Vec<_> = portals_by_dimension.keys().cloned().collect();
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

    // Helper function to get directional arrow
    let get_arrow = |dir: Vec2| -> &'static str {
        if dir == Vec2::ZERO {
            "?"
        } else {
            let angle = dir.y.atan2(dir.x);
            let normalized = ((angle * 4.0 / std::f32::consts::PI + 8.5) as i32 % 8) as usize;
            match normalized {
                0 => "→",
                1 => "↗",
                2 => "↑",
                3 => "↖",
                4 => "←",
                5 => "↙",
                6 => "↓",
                7 => "↘",
                _ => "?",
            }
        }
    };

    egui::Window::new("Portals List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(400.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Portals: {}", portal_query.iter().count()));
            ui.separator();

            for dim_key in sorted_dims.iter() {
                if let Some(mut portals) = portals_by_dimension.remove(dim_key) {
                    // Sort portals by distance (closest first)
                    portals.sort_by(|a, b| {
                        a.5.partial_cmp(&b.5).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    egui::CollapsingHeader::new(format!("{} ({})", dim_key, portals.len()))
                        .default_open(true)
                        .show(ui, |ui| {
                            for (entity, _global_pos, ezero_ref, direction, target_exists, distance) in portals.iter() {
                                // Check if this portal is selected
                                let is_selected = selected_entities.selected_portals.contains(entity);

                                // Get the StrId from EntityZero
                                let str_id_str = if let Some(ezero_ref) = ezero_ref {
                                    if let Ok(str_id) = ezero_query.get(ezero_ref.0) {
                                        format!("{}", str_id)
                                    } else {
                                        "Unknown".to_string()
                                    }
                                } else {
                                    "NoType".to_string()
                                };

                                let arrow = get_arrow(*direction);
                                let portal_label = format!("{} {} {:?} [{}]", arrow, str_id_str, entity, distance.round() as i32);

                                let text = egui::RichText::new(&portal_label);
                                let text = if !target_exists {
                                    text.color(egui::Color32::RED)
                                } else if is_selected {
                                    text.color(egui::Color32::YELLOW)
                                } else {
                                    text.color(egui::Color32::WHITE)
                                };

                                if ui.selectable_label(is_selected, text).clicked() {
                                    // Single select: clear previous selection and select new portal
                                    selected_entities.selected_portals.clear();
                                    selected_entities.selected_portals.insert(*entity);
                                    window_visible.portal_details = true;
                                }
                            }
                        });
                }
            }
        });
    window_visible.portals_list = open;
}
