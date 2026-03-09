use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use sprite_shared::prelude::SpriteConfig;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn sprites_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    sprite_query: Query<(Entity, Option<&Name>), With<SpriteConfig>>,
) {
    if !window_visible.sprite_configs_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 300.0;
    let default_y = screen_rect.bottom() - 400.0;
    let mut open = window_visible.sprite_configs_list;

    let sprites: Vec<(Entity, Option<Name>)> = sprite_query
        .iter()
        .map(|(entity, name)| (entity, name.map(|n| n.clone())))
        .collect();

    egui::Window::new("Sprite Configs List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(300.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Sprite configs: {}", sprites.len()));
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (entity, name) in sprites.iter() {
                        let label = if let Some(n) = name {
                            format!("{} ({:?})", n, entity)
                        } else {
                            format!("Sprite ({:?})", entity)
                        };
                        let is_selected = selected_entities.selected_sprite == Some(*entity);
                        if ui.selectable_label(is_selected, label).clicked() {
                            selected_entities.selected_sprite = Some(*entity);
                            window_visible.sprite_details = true;
                        }
                    }
                });
        });
    window_visible.sprite_configs_list = open;
}
