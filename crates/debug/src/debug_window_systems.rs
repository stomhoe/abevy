use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy::prelude::*;
use common::common_states::*;
use game_common::game_common_states::*;
use tilemap_shared::GlobalGenSettings;

use crate::debug_resources::DubugWindowsVisibility;

#[allow(unused_parens)]
pub fn debug_toggle_states_window(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if keys.just_pressed(KeyCode::F12) {
        window_visible.states = !window_visible.states;
    }
}

#[allow(unused_parens)]
pub fn debug_toggle_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if keys.just_pressed(KeyCode::F11) {
        window_visible.main_menu = !window_visible.main_menu;
    }
}

#[allow(unused_parens)]
pub fn states_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    app_state: Res<State<AppState>>,
    pre_game_state: Res<State<PreGameState>>,
    game_phase: Res<State<GamePhase>>,
    asset_loading: Res<State<AssetLoading>>,
    game_setup_screen: Option<Res<State<GameSetupScreen>>>,
    simulation_state: Option<Res<State<SimulationState>>>,
) {
    if !window_visible.states {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 300.0; // 300 pixels from right edge

    egui::Window::new("States Inspector")
        .default_pos([default_x, 10.0])
        .resizable(true)
        .movable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Main States");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.states = false;
                    }
                });
            });
            ui.separator();

            ui.label(format!("AppState: {:?}", app_state.get()));
            ui.label(format!("PreGameState: {:?}", pre_game_state.get()));
            ui.label(format!("GamePhase: {:?}", game_phase.get()));

            ui.separator();
            ui.label(format!("AssetLoading: {:?}", asset_loading.get()));

            ui.separator();
            ui.heading("Sub States (Setup)");
            if let Some(setup_screen) = game_setup_screen {
                ui.label(format!("GameSetupScreen: {:?}", setup_screen.get()));
            } else {
                ui.label("GameSetupScreen: Not active");
            }

            ui.separator();
            ui.heading("Sub States (Active Game)");
            if let Some(sim_state) = simulation_state {
                ui.label(format!("SimulationState: {:?}", sim_state.get()));
            } else {
                ui.label("SimulationState: Not active");
            }
        });
}

#[allow(unused_parens)]
pub fn main_menu_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if !window_visible.main_menu {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right();
    let default_y = screen_rect.top();

    egui::Window::new("Debug Menu")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Debug Windows");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.main_menu = false;
                    }
                });
            });
            ui.separator();

            if ui.button(egui::RichText::new("🔍 States Inspector (F12)").size(16.0)).clicked() {
                window_visible.states = !window_visible.states;
            }

            if ui.button(egui::RichText::new("▢▢  Chunking").size(16.0)).clicked() {
                window_visible.chunks_list = !window_visible.chunks_list;
            }

            if ui.button(egui::RichText::new("⬜ Regions").size(16.0)).clicked() {
                window_visible.regions_list = !window_visible.regions_list;
            }

            if ui.button(egui::RichText::new("👥 Beings list").size(16.0)).clicked() {
                window_visible.beings_list = !window_visible.beings_list;
            }

            if ui.button(egui::RichText::new("🌀 Portals").size(16.0)).clicked() {
                window_visible.portals_list = !window_visible.portals_list;
            }

            if ui.button(egui::RichText::new("� Sprite Configs").size(16.0)).clicked() {
                window_visible.sprite_configs_list = !window_visible.sprite_configs_list;
            }

            if ui.button(egui::RichText::new("�🌍 Terrain generation editor").size(16.0)).clicked() {
                window_visible.terrgen_editor = !window_visible.terrgen_editor;
            }

            if ui.button(egui::RichText::new("🧮 Terrain values grid").size(16.0)).clicked() {
                window_visible.terrgen_values = !window_visible.terrgen_values;
            }

            if ui.button(egui::RichText::new("🌐 Global generation settings").size(16.0)).clicked() {
                window_visible.settings_editor = !window_visible.settings_editor;
            }

            if ui.button(egui::RichText::new("📍 Registered Tile Positions").size(16.0)).clicked() {
                window_visible.registered_positions = !window_visible.registered_positions;
            }

            ui.separator();
            ui.label("F11: Toggle this menu");
        });
}

#[allow(unused_parens)]
pub fn global_gen_settings_editor_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut gen_settings: Query<&mut GlobalGenSettings>,
) {
    if !window_visible.settings_editor {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Ok(mut gen_settings) = gen_settings.single_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 10.0;
    let default_y = screen_rect.top() + 10.0;

    egui::Window::new("Global gen settings editor")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Global gen settings editor");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.settings_editor = false;
                    }
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Seed:");
                ui.add(egui::Slider::new(&mut gen_settings.seed, -1000..=1000));
            });

            ui.horizontal(|ui| {
                ui.label("World Frequency:");
                ui.add(egui::Slider::new(&mut gen_settings.world_freq, 0.01..=0.20));
            });

            ui.horizontal(|ui| {
                ui.label("Structure Build Timeout (s):");
                ui.add(egui::Slider::new(&mut gen_settings.structure_build_timeout_secs, 0.1..=60.0));
            });
        });
}
