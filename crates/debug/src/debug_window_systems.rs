use bevy_inspector_egui::bevy_egui::{EguiContexts, egui};
use bevy::prelude::*;
use common::common_states::*;
use game_common::game_common_states::*;

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
pub fn states_window(
    mut contexts: EguiContexts,
    window_visible: Res<DubugWindowsVisibility>,
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
            ui.vertical(|ui| {
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
        });
}