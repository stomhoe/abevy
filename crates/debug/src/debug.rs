#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy::ecs::schedule::common_conditions::on_message;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use ::being_shared::{BeingNavDebugLine, DebuggingBeingNav};
use ::being_shared::WallPhaserOnSpawn;
use tilemap_shared::DirectionalLight2dOverride;

    use crate::{
        being_details_inspector::*, beings_list_window::*, chunk_details_inspector::*,
        being_tile_click_picker::*,
        being_click_remover::*,
        being_nav_log_window::*,
        daylight_window::*,
        debug_chunking_window::*, debug_fonts::*, debug_resources::*,
        debug_systems::*, debug_window_systems::*,
        dimension_changer_window::*,
        gpos_maps_window::*,
        faction_details_inspector::*,
        nav_maps_window::*,
        macrochunks_grid_window::*,
        player_details_inspector::*, players_list_window::*, portals_list_window::*, region_details_inspector::*,
        regions_list_window::*, registered_positions_window::*, sprite_cfgs_details_inspector::*,
        sprite_cfgs_list_window::*, terrgen_editor_window::*, terrgen_values_window::*,
        inlandness_visualizer_window::*,
        tile_indices_map_window::*,
        world_tile_click_picker_window::*,
        tile_click_remover::*,
        tile_details_inspector::*,
        tilemap_details_inspector::*,
    };

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    let debug_enabled = |cfg: Res<DebugUiConfig>| cfg.enable_debug_menus;

    app.add_plugins((FpsCounterPlugin))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            load_debug_ui_config,
        )
        .add_systems(
            Update,
            (
                setup_debug_fonts,
                debug_toggle_hot_reload_window,
                debug_toggle_main_menu,
                capture_world_tile_click_selection,
                capture_world_tile_click_removal,
                capture_world_being_click_selection,
                capture_world_being_click_removal,
                capture_world_being_nav_selection,
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            Update,
            (
                receive_debug_increase_speed_request
                .run_if(on_message::<ac_input::LocalDebugIncreaseSpeedRequest>),
                receive_debug_decrease_speed_request
                .run_if(on_message::<ac_input::LocalDebugDecreaseSpeedRequest>),
            ).run_if(in_state(ServerState::Running)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                main_menu_window,
                states_window,
                all_states_window,
                collect_being_nav_debug_messages.run_if(on_message::<BeingNavDebugLine>),
                debug_chunking_window,
                macrochunks_grid_window,
                regions_list_window,
                beings_list_window,
                players_list_window,
                portals_list_window,
                sprites_list_window,
                terrgen_editor_window,
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                inlandness_visualizer_window,
                daylight_window,
                dimension_changer_window,
                terrgen_settings_editor_window,
                hot_reload_window,
                registered_positions_window,
                gpos_maps_window_system,
                tile_indices_map_window,
                nav_maps_window,
                world_tile_click_picker_window,
                nav_log_window,
                terrgen_debug_window_system
                    .run_if(|visible: Res<DubugWindowsVisibility>| visible.terrgen_values),
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                tile_details_inspector,
                chunk_details_inspector,
                region_details_inspector,
                tilemap_details_inspector,
                being_details_inspector,
                faction_details_inspector,
                player_details_inspector,
                sprite_details_inspector,
            )
                .run_if(debug_enabled),
        )
        .init_resource::<DubugWindowsVisibility>()
        .init_resource::<DebugSelectedEntities>()
        .init_resource::<DebugChunkingUiState>()
        .init_resource::<DebugNoiseWorkshopState>()
        .init_resource::<DebugFontsInitialized>()
        .init_resource::<DebugUiConfig>()
        .init_resource::<DirectionalLight2dOverride>()
        .init_resource::<WallPhaserOnSpawn>()
        .init_resource::<WorldTileClickInspectorState>()
        .init_resource::<TileClickRemoverState>()
        .init_resource::<BeingClickRemoverState>()
        .init_resource::<BeingTileClickInspectorState>()
        .init_resource::<DebugBeingNavUiState>()
        .init_resource::<DebuggingBeingNav>()
        .init_resource::<common::common_states::HotReloadSelection>();
}
