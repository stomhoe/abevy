use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn being_details_inspector(world: &mut World) {
    let selected_being_entity = if let Some(selected_entities) = world.get_resource::<DebugSelectedEntities>() {
        selected_entities.selected_being
    } else {
        None
    };

    if selected_being_entity.is_none() {
        return;
    }

    let selected_being_entity = selected_being_entity.unwrap();
    let window_visible = world.resource::<DubugWindowsVisibility>();

    if !window_visible.being_details {
        return;
    }

    let _ = window_visible;  // Release the resource borrow

    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();

    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };

    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let world_ptr = world as *mut World;
    let mut is_open = true;

    egui::Window::new("Selected Being Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.heading(format!("Being Entity: {:?}", selected_being_entity));
            ui.separator();

            ui.label("All Components on this Being:");
            ui.separator();

            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_being_entity, ui);
            }

            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_being = None;
                }
            }
        });

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.being_details = false;
        }
    }
}
