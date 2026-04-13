use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::{ClientState, ServerState};
use ac_input::ac_input_actions::{
    DebugToggleHotReloadWindowAction, DebugToggleMainMenuAction, HotReloadAction,
};
use common::common_states::*;
use game_common::game_common_states::*;
use tilemap_shared::GlobalGenSettings;
use tilemap::regioning::regioning_resources::StructureGenerationSettings;

use crate::being_tile_click_picker::set_being_click_picker_active;
use crate::debug_resources::{BeingTileClickInspectorState, DubugWindowsVisibility};

fn render_state_row<T: States + std::fmt::Debug>(ui: &mut egui::Ui, label: &str, state: &State<T>) {
    ui.horizontal(|ui| {
        ui.monospace(label);
        ui.label(format!("{:?}", state.get()));
    });
}

fn render_optional_state_row<T: States + std::fmt::Debug>(ui: &mut egui::Ui, label: &str, state: Option<Res<State<T>>>) {
    ui.horizontal(|ui| {
        ui.monospace(label);
        if let Some(state) = state {
            ui.label(format!("{:?}", state.into_inner().get()));
        } else {
            ui.label("Not available");
        }
    });
}

#[allow(unused_parens)]
pub fn debug_toggle_hot_reload_window(
    hot_reload_toggle_events: Single<&ActionEvents, With<Action<DebugToggleHotReloadWindowAction>>>,
    main_menu_toggle: Single<&Action<DebugToggleMainMenuAction>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if hot_reload_toggle_events.contains(ActionEvents::START) && !***main_menu_toggle {
        window_visible.hot_reload_window_open_on_start = !window_visible.hot_reload_window_open_on_start;
    }
}

#[allow(unused_parens)]
pub fn debug_toggle_main_menu(
    main_menu_toggle_events: Single<&ActionEvents, With<Action<DebugToggleMainMenuAction>>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if main_menu_toggle_events.contains(ActionEvents::START) {
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
pub fn all_states_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    app_state: Res<State<AppState>>,
    pre_game_state: Res<State<PreGameState>>,
    game_phase: Res<State<GamePhase>>,
    asset_loading: Res<State<AssetLoading>>,
    asset_hot_reload_state: Res<State<AssetHotReloadState>>,
    client_state: Res<State<ClientState>>,
    server_state: Res<State<ServerState>>,
    game_setup_screen: Option<Res<State<GameSetupScreen>>>,
    simulation_state: Option<Res<State<SimulationState>>>,
) {
    if !window_visible.all_states {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.right() - 340.0;
    let mut open = window_visible.all_states;

    egui::Window::new("All States Inspector")
        .default_pos([default_x, 10.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Core");
                render_state_row(ui, "AppState", &app_state);
                render_state_row(ui, "PreGameState", &pre_game_state);
                render_state_row(ui, "GamePhase", &game_phase);

                ui.separator();
                ui.heading("Network / Loading");
                render_state_row(ui, "ClientState", &client_state);
                render_state_row(ui, "ServerState", &server_state);
                render_state_row(ui, "AssetLoading", &asset_loading);
                render_state_row(ui, "AssetHotReloadState", &asset_hot_reload_state);

                ui.separator();
                ui.heading("Sub States");
                render_optional_state_row(ui, "GameSetupScreen", game_setup_screen);
                render_optional_state_row(ui, "SimulationState", simulation_state);
            });
        });
    window_visible.all_states = open;
}

#[allow(unused_parens)]
pub fn main_menu_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut being_click_picker_state: ResMut<BeingTileClickInspectorState>,
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
                window_visible.all_states = false;
                window_visible.chunks_list = false;
                window_visible.macrochunks_grid = false;
                window_visible.regions_list = false;
                window_visible.beings_list = false;
                window_visible.players_list = false;
                window_visible.portals_list = false;
                window_visible.terrgen_editor = false;
                window_visible.terrgen_values = false;
                window_visible.terrain_visualizer = false;
                window_visible.settings_editor = false;
                window_visible.tile_details = false;
                window_visible.chunk_details = false;
                window_visible.region_details = false;
                window_visible.tilemap_details = false;
                window_visible.being_details = false;
                window_visible.faction_details = false;
                window_visible.player_details = false;
                window_visible.registered_positions = false;
                window_visible.exempted_entity_details = false;
                window_visible.sprite_configs_list = false;
                window_visible.sprite_details = false;
                window_visible.gpos_maps = false;
                window_visible.tile_indices_map = false;
                window_visible.world_tile_click_picker = false;
                window_visible.being_tile_click_picker = false;
                window_visible.being_nav_log = false;
                window_visible.hot_reload_window_open_on_start = false;
                window_visible.river_debug = false;
            }
            ui.separator();
            if ui.button(egui::RichText::new("🔍 States Inspector").size(16.0)).clicked() {
                window_visible.states = !window_visible.states;
            }
            if ui.button(egui::RichText::new("📚 All States").size(16.0)).clicked() {
                window_visible.all_states = !window_visible.all_states;
            }
            if ui.button(egui::RichText::new("▢▢  Chunking").size(16.0)).clicked() {
                window_visible.chunks_list = !window_visible.chunks_list;
            }
            if ui.button(egui::RichText::new("MacroChunks Grid").size(16.0)).clicked() {
                window_visible.macrochunks_grid = !window_visible.macrochunks_grid;
            }
            if ui.button(egui::RichText::new("⬜ Regions").size(16.0)).clicked() {
                window_visible.regions_list = !window_visible.regions_list;
            }
            if ui.button(egui::RichText::new("GPos Maps").size(16.0)).clicked() {
                window_visible.gpos_maps = !window_visible.gpos_maps;
            }
            if ui.button(egui::RichText::new("Tile Index Map").size(16.0)).clicked() {
                window_visible.tile_indices_map = !window_visible.tile_indices_map;
            }
            if ui.button(egui::RichText::new("🖱️ TileGpos Click Picker").size(16.0)).clicked() {
                window_visible.world_tile_click_picker = !window_visible.world_tile_click_picker;
            }
            let being_click_picker_label = if window_visible.being_tile_click_picker {
                "🖱️ Stop Being Click Picker"
            } else {
                "🖱️ Being Click Picker"
            };
            let mut being_click_picker_button = egui::Button::new(being_click_picker_label);
            if window_visible.being_tile_click_picker {
                being_click_picker_button = being_click_picker_button.fill(egui::Color32::from_rgb(80, 120, 60));
            }
            if ui.add(being_click_picker_button).clicked() {
                set_being_click_picker_active(
                    !window_visible.being_tile_click_picker,
                    &mut being_click_picker_state,
                    &mut window_visible,
                );
            }
            if ui.button(egui::RichText::new("NavLog").size(16.0)).clicked() {
                window_visible.being_nav_log = true;
            }
            if ui.button(egui::RichText::new("👥 Beings list").size(16.0)).clicked() {
                window_visible.beings_list = !window_visible.beings_list;
            }
            if ui.button(egui::RichText::new("🏰 Faction Details").size(16.0)).clicked() {
                window_visible.faction_details = !window_visible.faction_details;
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
            if ui.button(egui::RichText::new("▦ Terrain noise values map").size(16.0)).clicked() {
                window_visible.terrgen_values = !window_visible.terrgen_values;
            }
            if ui.button(egui::RichText::new("⛰ Terrain visualizer").size(16.0)).clicked() {
                window_visible.terrain_visualizer = !window_visible.terrain_visualizer;
            }
            if ui.button(egui::RichText::new("🌐 Global generation settings").size(16.0)).clicked() {
                window_visible.settings_editor = !window_visible.settings_editor;
            }
            if ui.button(egui::RichText::new("📍 Important Tile Positions").size(16.0)).clicked() {
                window_visible.registered_positions = !window_visible.registered_positions;
            }
            if ui.button(egui::RichText::new("♻ Hot Reload").size(16.0)).clicked() {
                window_visible.hot_reload_window_open_on_start = !window_visible.hot_reload_window_open_on_start;
            }
            ui.separator();
            ui.label("F11: Toggle this menu");
        });
    window_visible.main_menu = open;
}

#[allow(unused_parens)]
pub fn hot_reload_window(
    mut cmd: Commands,
    hot_reload_action_events: Single<&ActionEvents, With<Action<HotReloadAction>>>,
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selection: ResMut<HotReloadSelection>,
) {
    if !window_visible.hot_reload_window_open_on_start {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen_rect = ctx.content_rect();
    let mut open = window_visible.hot_reload_window_open_on_start;
    egui::Window::new("Hot Reload")
        .default_pos([screen_rect.left() + 24.0, screen_rect.top() + 24.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Hot Reload Sets");
            ui.separator();
            ui.checkbox(&mut selection.terrgen_settings, "Terrgen settings");
            ui.checkbox(&mut selection.probes_and_filters, "Probes and filters");
            ui.checkbox(&mut selection.terrain_oplists_and_noises, "Terrain oplists and noises");
            ui.checkbox(&mut selection.tiles, "Tiles");
            ui.checkbox(&mut selection.sprite_configs_and_animations, "Sprite configs + animations");
            ui.checkbox(&mut selection.beings_inst_templates, "Being templates");
            ui.checkbox(&mut selection.races, "Races");
            ui.checkbox(&mut selection.sexes, "Sexes");
            if ui.button("Clear selections").clicked() {
                selection.terrgen_settings = false;
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
                cmd.trigger(HotReloadRequest);
            }
        });

    if open && hot_reload_action_events.contains(ActionEvents::START) {
        cmd.trigger(HotReloadRequest);
    }

    window_visible.hot_reload_window_open_on_start = open;
}

#[allow(unused_parens)]
pub fn terrgen_settings_editor_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut gen_settings: Query<&mut GlobalGenSettings>,
    mut structure_settings: Query<&mut StructureGenerationSettings>,
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
    let Ok(mut structure_settings) = structure_settings.single_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 10.0;
    let default_y = screen_rect.top() + 10.0;
    let mut open = window_visible.settings_editor;

    egui::Window::new("Terrgen settings editor")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Terrgen settings editor");

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
                ui.add(egui::Slider::new(&mut structure_settings.structure_build_timeout_secs, 0.1..=60.0));
            });
        });
    window_visible.settings_editor = open;
}
