use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy::prelude::*;
use common::common_states::*;
use game_common::game_common_states::*;
use tilemap_shared::{ForceAllChunksDespawn, GlobalGenSettings};

use crate::debug_resources::{DebugUiConfig, DubugWindowsVisibility, load_debug_ui_config_seri_defs};

#[allow(unused_parens)]
pub fn debug_toggle_hot_reload_window(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if keys.just_pressed(KeyCode::F12) && !keys.pressed(KeyCode::F11) {
        window_visible.hot_reload_menu = !window_visible.hot_reload_menu;
    }
}

pub fn apply_initial_hot_reload_visibility_from_world_settings(
    mut initialized: Local<bool>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    gen_settings: Query<&GlobalGenSettings>,
) {
    if *initialized {
        return;
    }
    let Ok(gen_settings) = gen_settings.single() else {
        return;
    };
    window_visible.hot_reload_menu = gen_settings.hot_reload_window_open_on_start;
    *initialized = true;
}

pub fn apply_debug_ui_config_once(
    mut initialized: Local<bool>,
    mut cfg: ResMut<DebugUiConfig>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selection: ResMut<HotReloadSelection>,
) {
    if *initialized {
        return;
    }

    let defs = load_debug_ui_config_seri_defs();
    let Some(def) = defs.first() else {
        *initialized = true;
        return;
    };

    *cfg = def.to_config();
    *selection = cfg.hot_reload_defaults.clone();
    *window_visible = cfg.windows_open_on_start.clone();

    *initialized = true;
}

#[allow(unused_parens)]
pub fn debug_toggle_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if keys.just_pressed(KeyCode::F11) {
        window_visible.main_menu = !window_visible.main_menu;
        // Keep F11 scoped to main menu only.
        window_visible.states = false;
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
    let mut open = window_visible.states;

    egui::Window::new("States Inspector")
        .default_pos([default_x, 10.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Main States");
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
    window_visible.states = open;
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
    let mut open = window_visible.main_menu;

    egui::Window::new("Debug Menu")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            let close_all = egui::Button::new(
                egui::RichText::new("⛔ Close All Windows")
                    .size(16.0)
                    .color(egui::Color32::WHITE)
                    .strong(),
            )
            .fill(egui::Color32::from_rgb(150, 28, 28))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 110, 110)));
            if ui.add_sized([ui.available_width(), 28.0], close_all).clicked() {
                window_visible.states = false;
                window_visible.chunks_list = false;
                window_visible.regions_list = false;
                window_visible.beings_list = false;
                window_visible.players_list = false;
                window_visible.portals_list = false;
                window_visible.portal_details = false;
                window_visible.terrgen_editor = false;
                window_visible.terrgen_values = false;
                window_visible.settings_editor = false;
                window_visible.tile_details = false;
                window_visible.chunk_details = false;
                window_visible.region_details = false;
                window_visible.tilemap_details = false;
                window_visible.being_details = false;
                window_visible.player_details = false;
                window_visible.registered_positions = false;
                window_visible.exempted_entity_details = false;
                window_visible.sprite_configs_list = false;
                window_visible.sprite_details = false;
                window_visible.gpos_maps = false;
                window_visible.hot_reload_menu = false;
                window_visible.river_debug = false;
            }
            ui.separator();
            if ui.button(egui::RichText::new("🔍 States Inspector").size(16.0)).clicked() {
                window_visible.states = !window_visible.states;
            }
            if ui.button(egui::RichText::new("▢▢  Chunking").size(16.0)).clicked() {
                window_visible.chunks_list = !window_visible.chunks_list;
            }
            if ui.button(egui::RichText::new("⬜ Regions").size(16.0)).clicked() {
                window_visible.regions_list = !window_visible.regions_list;
            }
            if ui.button(egui::RichText::new("GPos Maps").size(16.0)).clicked() {
                window_visible.gpos_maps = !window_visible.gpos_maps;
            }
            if ui.button(egui::RichText::new("👥 Beings list").size(16.0)).clicked() {
                window_visible.beings_list = !window_visible.beings_list;
            }
            if ui.button(egui::RichText::new("🧑 Players list").size(16.0)).clicked() {
                window_visible.players_list = !window_visible.players_list;
            }
            if ui.button(egui::RichText::new("🌀 Portals").size(16.0)).clicked() {
                window_visible.portals_list = !window_visible.portals_list;
            }
            if ui.button(egui::RichText::new("� Sprite Configs").size(16.0)).clicked() {
                window_visible.sprite_configs_list = !window_visible.sprite_configs_list;
            }
            if ui.button(egui::RichText::new("🌍 Terrain generation editor").size(16.0)).clicked() {
                window_visible.terrgen_editor = !window_visible.terrgen_editor;
            }
            if ui.button(egui::RichText::new("🧮 Terrain noise values map").size(16.0)).clicked() {
                window_visible.terrgen_values = !window_visible.terrgen_values;
            }
            if ui.button(egui::RichText::new("🌐 Global generation settings").size(16.0)).clicked() {
                window_visible.settings_editor = !window_visible.settings_editor;
            }
            if ui.button(egui::RichText::new("📍 Important Tile Positions").size(16.0)).clicked() {
                window_visible.registered_positions = !window_visible.registered_positions;
            }
            if ui.button(egui::RichText::new("♻ Hot Reload").size(16.0)).clicked() {
                window_visible.hot_reload_menu = !window_visible.hot_reload_menu;
            }
            ui.separator();
            ui.label("F11: Toggle this menu");
        });
    window_visible.main_menu = open;
}

#[allow(unused_parens)]
pub fn hot_reload_window(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selection: ResMut<HotReloadSelection>,
    mut request: ResMut<HotReloadRequest>,
    mut force_all_chunks_despawn_writer: MessageWriter<ForceAllChunksDespawn>,
) {
    if !window_visible.hot_reload_menu {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen_rect = ctx.content_rect();
    let mut open = window_visible.hot_reload_menu;
    egui::Window::new("Hot Reload")
        .default_pos([screen_rect.left() + 24.0, screen_rect.top() + 24.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Hot Reload Sets");
            ui.separator();
            ui.checkbox(&mut selection.global_gen_settings, "Global gen settings");
            ui.checkbox(&mut selection.probes_and_filters, "Probes and filters");
            ui.checkbox(&mut selection.terrain_oplists_and_noises, "Terrain oplists and noises");
            ui.checkbox(&mut selection.tiles, "Tiles");
            ui.checkbox(&mut selection.sprite_configs_and_animations, "Sprite configs + animations");
            ui.checkbox(&mut selection.beings_inst_templates, "Being templates");
            ui.checkbox(&mut selection.races, "Races");
            ui.checkbox(&mut selection.sexes, "Sexes");
            if ui.button("Clear selections").clicked() {
                selection.global_gen_settings = false;
                selection.probes_and_filters = false;
                selection.terrain_oplists_and_noises = false;
                selection.tiles = false;
                selection.sprite_configs_and_animations = false;
                selection.beings_inst_templates = false;
                selection.races = false;
                selection.sexes = false;
            }
            ui.separator();
            if ui.button("Hot reload").clicked() {
                request.requested = true;
                force_all_chunks_despawn_writer.write(ForceAllChunksDespawn);
            }
        });

    if open && keys.just_pressed(KeyCode::KeyR) {
        request.requested = true;
        force_all_chunks_despawn_writer.write(ForceAllChunksDespawn);
    }

    window_visible.hot_reload_menu = open;
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
    let mut open = window_visible.settings_editor;

    egui::Window::new("Global gen settings editor")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Global gen settings editor");

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
                ui.label("Tectonic Frequency:");
                ui.add(egui::Slider::new(&mut gen_settings.tectonic_frequency, 0.01..=0.20));
            });

            ui.horizontal(|ui| {
                ui.label("Structure Build Timeout (s):");
                ui.add(egui::Slider::new(&mut gen_settings.structure_build_timeout_secs, 0.1..=60.0));
            });
        });
    window_visible.settings_editor = open;
}
