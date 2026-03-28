use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;
use common::common_tag_components::TagSet;
use game_common::game_common_components::TemplEntiRef;
use tilemap::tile::tile_components::{TileStrId};
use tilemap_shared::DeleteOtherTilesInSamePos;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn tile_details_inspector(world: &mut World) {
    let selected_entities = world.resource::<DebugSelectedEntities>();
    let selected_tile_entity = selected_entities.selected_tile.or(selected_entities.selected_exempted_entity);

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
        if let Some(templ_ref) = entity_ref.get::<TemplEntiRef>() {
            if let Ok(templ_entity) = world.get_entity(templ_ref.0) {
                if let Some(str_id) = templ_entity.get::<TileStrId>() {
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

    let mut delete_other_tiles_here = None;
    let mut delete_other_tiles_templ = None;
    let mut tags_here = None;
    let mut tags_templ = None;
    let mut referenced_templ_entity = None;
    if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        delete_other_tiles_here = entity_ref.get::<DeleteOtherTilesInSamePos>().cloned();
        tags_here = entity_ref.get::<TagSet>().cloned();
        if let Some(templ_ref) = entity_ref.get::<TemplEntiRef>() {
            referenced_templ_entity = Some(templ_ref.0);
            if let Ok(templ_entity_ref) = world.get_entity(templ_ref.0) {
                delete_other_tiles_templ = templ_entity_ref.get::<DeleteOtherTilesInSamePos>().cloned();
                tags_templ = templ_entity_ref.get::<TagSet>().cloned();
            }
        }
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
                ui.heading(format!("Entity: {:?}", selected_tile_entity));
            }
            ui.separator();

            ui.label("All Components on this Entity:");
            ui.separator();

            // Use unsafe to access world for full component inspection with values
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_tile_entity, ui);
            }

            ui.separator();
            ui.heading("Manual component details");
            render_tagset_section(ui, "TagSet on selected entity", tags_here.as_ref());
            render_delete_other_tiles_section(
                ui,
                "DeleteOtherTilesInSamePos on selected entity",
                delete_other_tiles_here.as_ref(),
            );

            if let Some(templ_entity) = referenced_templ_entity {
                ui.separator();
                ui.label(format!("TemplEntiRef target: {:?}", templ_entity));
                render_tagset_section(ui, "TagSet on EntityZero", tags_templ.as_ref());
                render_delete_other_tiles_section(
                    ui,
                    "DeleteOtherTilesInSamePos on EntityZero",
                    delete_other_tiles_templ.as_ref(),
                );
            }

            ui.separator();
            if ui.button("Clear Selection").clicked() {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_tile = None;
                    selected_entities.selected_exempted_entity = None;
                }
            }
        });

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.tile_details = false;
        }
    }
}

fn render_tagset_section(ui: &mut egui::Ui, title: &str, tags: Option<&TagSet>) {
    ui.collapsing(title, |ui| {
        let Some(tags) = tags else {
            ui.label("Missing");
            return;
        };
        if tags.is_empty() {
            ui.label("Empty");
            return;
        }

        let mut values: Vec<String> = tags.iter().map(|tag| format!("{:?}", tag)).collect();
        values.sort_unstable();
        ui.label(format!("count: {}", values.len()));
        ui.label(values.join(", "));
    });
}

fn render_delete_other_tiles_section(
    ui: &mut egui::Ui,
    title: &str,
    spec: Option<&DeleteOtherTilesInSamePos>,
) {
    ui.collapsing(title, |ui| {
        let Some(spec) = spec else {
            ui.label("Missing");
            return;
        };

        let mut spared_z: Vec<f32> = spec.spared_z.iter().map(|z| z.0).collect();
        spared_z.sort_by(|a, b| a.total_cmp(b));
        let mut targeted_z: Vec<f32> = spec.targeted_z.iter().map(|z| z.0).collect();
        targeted_z.sort_by(|a, b| a.total_cmp(b));

        let mut spared_tags: Vec<String> = spec.spared_tags.iter().map(|tag| format!("{:?}", tag)).collect();
        spared_tags.sort_unstable();
        let mut targeted_tags: Vec<String> = spec.targeted_tags.iter().map(|tag| format!("{:?}", tag)).collect();
        targeted_tags.sort_unstable();

        ui.label(format!("priority: {:.3}", spec.priority));
        ui.label(format!("extra_radius: {}", spec.extra_radius));
        ui.label(format!("displacement: ({}, {})", spec.displacement.x, spec.displacement.y));
        ui.label(format!("spared_z: {:?}", spared_z));
        ui.label(format!("targeted_z: {:?}", targeted_z));
        ui.label(format!("spared_tags: {:?}", spared_tags));
        ui.label(format!("targeted_tags: {:?}", targeted_tags));
        ui.label(format!("is_empty(): {}", spec.is_empty()));
    });
}
