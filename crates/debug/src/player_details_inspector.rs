use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn player_details_inspector(world: &mut World) {
    let selected_player_entity = world.resource::<DebugSelectedEntities>().selected_player;

    if selected_player_entity.is_none() {
        return;
    }

    let selected_player_entity = selected_player_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();

    if !window_visible.player_details {
        return;
    }

    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();

    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };

    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let world_ptr = world as *mut World;
    let mut is_open = true;

    egui::Window::new("Selected Player Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            if let Ok(entity_ref) = world.get_entity(selected_player_entity) {
                if let Some(name) = entity_ref.get::<Name>() {
                    ui.heading(format!("Player: {}", name));
                } else {
                    ui.heading(format!("Player Entity: {:?}", selected_player_entity));
                }
            }
            ui.separator();

            ui.label("All Components on this Player:");
            ui.separator();

            // Reuse Beings/Sprites debug pattern to inspect all present components.
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_player_entity, ui);
            }

            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_player = None;
                }
            }
        });

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.player_details = false;
        }
    }
}
