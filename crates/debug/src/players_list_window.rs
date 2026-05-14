use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use common::common_components::StrId;
use player_shared::player_components::{Mine, Player};

use debug_shared::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn players_list_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    players_query: Query<(Entity, Has<Mine>), With<Player>>,
    name_query: Query<&Name>,
    str_id_query: Query<&StrId>,
) {
    if !window_visible.players_list {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 350.0;
    let default_y = screen_rect.top() + 40.0;
    let mut open = window_visible.players_list;

    let mut players: Vec<(Entity, bool, String)> = players_query
        .iter()
        .map(|(entity, is_mine)| {
            let label = if let Ok(strid) = str_id_query.get(entity) {
                strid.to_string()
            } else if let Ok(name) = name_query.get(entity) {
                name.to_string()
            } else {
                "Unnamed".to_string()
            };
            (entity, is_mine, label)
        })
        .collect();

    players.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));

    egui::Window::new("Players List")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(350.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Players: {}", players.len()));
            ui.separator();

            for (entity, is_mine, label) in players.iter() {
                let mine_suffix = if *is_mine { " [Mine]" } else { "" };
                let text = format!("{} ({:?}){}", label, entity, mine_suffix);
                let is_selected = selected_entities.selected_player == Some(*entity);
                if ui.selectable_label(is_selected, text).clicked() {
                    selected_entities.selected_player = Some(*entity);
                    window_visible.player_details = true;
                }
            }
        });
    window_visible.players_list = open;
}
