use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use common::common_components::{DisplayName, StrId};

use ::being_shared::{
    Being,
    BeingNavDebugField,
    BeingNavDebugKind,
    BeingNavDebugLine,
    BeingNavDebugValue,
    DebuggingBeingNav,
};

use crate::debug_resources::{DebugBeingNavUiState, DubugWindowsVisibility};

fn nav_kind_label(kind: &BeingNavDebugKind) -> String {
    match kind {
        BeingNavDebugKind::State => "State".to_string(),
        BeingNavDebugKind::Decision => "Decision".to_string(),
        BeingNavDebugKind::Repath => "Repath".to_string(),
        BeingNavDebugKind::Path => "Path".to_string(),
        BeingNavDebugKind::Target => "Target".to_string(),
        BeingNavDebugKind::Clear => "Clear".to_string(),
        BeingNavDebugKind::Track => "Track".to_string(),
        BeingNavDebugKind::Info => "Info".to_string(),
        BeingNavDebugKind::Other(kind) => kind.clone(),
    }
}

fn format_nav_value(value: &BeingNavDebugValue) -> String {
    match value {
        BeingNavDebugValue::Bool(value) => value.to_string(),
        BeingNavDebugValue::I32(value) => value.to_string(),
        BeingNavDebugValue::I64(value) => value.to_string(),
        BeingNavDebugValue::U32(value) => value.to_string(),
        BeingNavDebugValue::U64(value) => value.to_string(),
        BeingNavDebugValue::F32(value) => format!("{value:.3}"),
        BeingNavDebugValue::F64(value) => format!("{value:.3}"),
        BeingNavDebugValue::Text(value) => value.clone(),
        BeingNavDebugValue::Entity(value) => format!("{value:?}"),
        BeingNavDebugValue::GPos(value) => format!("[{}, {}]", value.0.x, value.0.y),
        BeingNavDebugValue::Chunk(value) => format!("{value:?}"),
        BeingNavDebugValue::EntityList(values) => format!("{values:?}"),
        BeingNavDebugValue::GPosList(values) => format!("{values:?}"),
        BeingNavDebugValue::MaybeEntity(value) => value.map_or_else(|| "missing".to_string(), |value| format!("{value:?}")),
        BeingNavDebugValue::MaybeGPos(value) => value.map_or_else(|| "missing".to_string(), |value| format!("[{}, {}]", value.0.x, value.0.y)),
        BeingNavDebugValue::MaybeI32(value) => value.map_or_else(|| "missing".to_string(), |value| value.to_string()),
        BeingNavDebugValue::MaybeU32(value) => value.map_or_else(|| "missing".to_string(), |value| value.to_string()),
        BeingNavDebugValue::MaybeF32(value) => value.map_or_else(|| "missing".to_string(), |value| format!("{value:.3}")),
        BeingNavDebugValue::MaybeText(value) => value.clone().unwrap_or_else(|| "missing".to_string()),
    }
}

fn format_nav_fields(fields: &[BeingNavDebugField]) -> String {
    let mut row = String::new();
    for (ix, field) in fields.iter().enumerate() {
        if ix > 0 {
            row.push_str(" | ");
        }
        row.push_str(&field.key);
        row.push('=');
        row.push_str(&format_nav_value(&field.value));
    }
    row
}

fn being_label(
    being_ent: Entity,
    display_name_query: &Query<&DisplayName>,
    str_id_query: &Query<&StrId>,
    being_query: &Query<(Entity, Has<Being>, ), (With<Being>, )>,
) -> String {
    let exists = being_query.get(being_ent).is_ok_and(|(_, has_being, )| has_being);
    let mut label = display_name_query
        .get(being_ent)
        .map(|display_name| display_name.0.clone())
        .or_else(|_| str_id_query.get(being_ent).map(|str_id| str_id.to_string()))
        .unwrap_or_else(|_| format!("{being_ent:?}"));
    if !exists {
        label.push_str(" (missing)");
    }
    label
}

#[allow(unused_parens, )]
pub fn collect_being_nav_debug_messages(
    mut reader: MessageReader<BeingNavDebugLine>,
    mut nav_debug: ResMut<DebuggingBeingNav>,
) {
    for line in reader.read() {
        nav_debug.push_line(line.clone());
    }
}

#[allow(unused_parens, )]
pub fn nav_log_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut nav_debug: ResMut<DebuggingBeingNav>,
    mut ui_state: ResMut<DebugBeingNavUiState>,
    display_name_query: Query<&DisplayName>,
    str_id_query: Query<&StrId>,
    being_query: Query<(Entity, Has<Being>, ), (With<Being>, )>,
) {
    if !window_visible.being_nav_log {
        nav_debug.clear_all();
        ui_state.track_new_being = false;
        ui_state.last_clicked_dim = None;
        ui_state.last_clicked_gpos = None;
        ui_state.last_selected_being = None;
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if ui_state.track_new_being {
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
    }

    let screen_rect = ctx.content_rect();
    let mut open = window_visible.being_nav_log;
    let mut clear_all = false;

    egui::Window::new("NavLog")
        .default_pos([screen_rect.left() + 20.0, screen_rect.bottom() - 360.0])
        .default_size([980.0, 320.0])
        .resizable(true)
        .movable(true)
        .collapsible(true)
        .open(&mut open)
        .show(ctx, |ui| {
            if ui_state.track_new_being && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                ui_state.track_new_being = false;
            }
            ui.horizontal(|ui| {
                let track_label = if ui_state.track_new_being {
                    "Stop Tracking"
                } else {
                    "Track New BeingNav"
                };
                let mut track_button = egui::Button::new(track_label);
                if ui_state.track_new_being {
                    track_button = track_button.fill(egui::Color32::from_rgb(80, 120, 60));
                }
                if ui.add(track_button).clicked() {
                    ui_state.track_new_being = !ui_state.track_new_being;
                }
                if ui.button("Clear All").clicked() {
                    clear_all = true;
                }
                let pause_all_label = if nav_debug.pause_all {
                    "Resume All Logs"
                } else {
                    "Pause All Logs"
                };
                if ui.button(pause_all_label).clicked() {
                    nav_debug.toggle_pause_all();
                }
                ui.label(format!(
                    "Tracked {}/{}",
                    nav_debug.tracked_count(),
                    ::being_shared::MAX_TRACKED_BEING_NAV_COLUMNS
                ));
                if ui_state.track_new_being {
                    ui.label("Track mode: click a being with nav state or press Esc");
                }
            });

            ui.separator();

            let tracked_beings = nav_debug.tracked_beings.clone();
            if tracked_beings.is_empty() {
                ui.label("No beings are being tracked.");
                return;
            }

            ui.columns(tracked_beings.len(), |columns| {
                for (ix, being_ent) in tracked_beings.iter().copied().enumerate() {
                    let column = &mut columns[ix];
                    column.vertical(|ui| {
                        let paused = nav_debug.is_column_paused(being_ent);
                        ui.horizontal(|ui| {
                            ui.monospace(being_label(being_ent, &display_name_query, &str_id_query, &being_query));
                            if paused {
                                ui.label("paused");
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.add_enabled(ix > 0, egui::Button::new("←")).clicked() {
                                nav_debug.move_left(being_ent);
                            }
                            if ui.add_enabled(ix + 1 < tracked_beings.len(), egui::Button::new("→")).clicked() {
                                nav_debug.move_right(being_ent);
                            }
                            let pause_label = if paused { "Resume" } else { "Pause" };
                            if ui.button(pause_label).clicked() {
                                nav_debug.toggle_column_pause(being_ent);
                            }
                            if ui.button("Clear").clicked() {
                                nav_debug.clear_column(being_ent);
                            }
                            if ui.button("Remove").clicked() {
                                nav_debug.remove_being(being_ent);
                            }
                        });
                        ui.separator();

                        let lines = nav_debug
                            .columns
                            .get(&being_ent)
                            .map(|column| column.lines.iter().cloned().collect::<Vec<_>>())
                            .unwrap_or_default();
                        egui::ScrollArea::vertical()
                            .id_salt(("being_nav_log", being_ent))
                            .show(ui, |ui| {
                                if lines.is_empty() {
                                    ui.label("No lines yet.");
                                    return;
                                }
                                for line in lines.iter() {
                                    ui.group(|ui| {
                                        ui.monospace(format!(
                                            "[{:.3}] {} {:?} {}",
                                            line.timestamp_secs,
                                            line.system,
                                            nav_kind_label(&line.kind),
                                            line.summary
                                        ));
                                        if !line.fields.is_empty() {
                                            ui.label(format_nav_fields(&line.fields));
                                        }
                                    });
                                }
                            });
                    });
                }
            });
        });

    if clear_all {
        nav_debug.clear_all();
    }

    if !open {
        nav_debug.clear_all();
        ui_state.track_new_being = false;
        ui_state.last_clicked_dim = None;
        ui_state.last_clicked_gpos = None;
        ui_state.last_selected_being = None;
    }

    window_visible.being_nav_log = open;
}
