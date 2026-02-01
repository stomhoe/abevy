use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};
use tilemap::tile::tile_components::TileStrId;

#[allow(unused_parens)]
pub fn tile_details_inspector(world: &mut World) {
    let selected_tile_entity = world.resource::<DebugSelectedEntities>().selected_tile;

    if selected_tile_entity.is_none() {
        return;
    }

    let selected_tile_entity = selected_tile_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();

    if !window_visible.tile_details {
        return;
    }

    // Try to get the TileStrId from the referenced EntityZero
    let tile_str_id = if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        if let Some(ezero_ref) = entity_ref.get::<game_common::game_common_components::EntityZeroRef>() {
            if let Ok(ezero_entity) = world.get_entity(ezero_ref.0) {
                if let Some(str_id) = ezero_entity.get::<TileStrId>() {
                    Some(format!("{}", str_id))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();

    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };

    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let world_ptr = world as *mut World;
    let mut is_open = true;

    egui::Window::new("Selected Tile Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            // Display TileStrId if available, otherwise show Entity ID
            if let Some(str_id) = tile_str_id {
                ui.heading(format!("Tile: {}", str_id));
            } else {
                ui.heading(format!("Tile Entity: {:?}", selected_tile_entity));
            }
            ui.separator();

            ui.label("All Components on this Tile:");
            ui.separator();

            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_tile_entity, ui);
            }

            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_tile = None;
                }
            }
        });

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.tile_details = false;
        }
    }
}
