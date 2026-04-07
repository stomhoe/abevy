#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy::ecs::schedule::common_conditions::on_message;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use game_common::AcClientSystems;
use movement::MovementSystems;

    use crate::{
        being_details_inspector::*, beings_list_window::*, chunk_details_inspector::*,
        debug_chunking_window::*, debug_fonts::*, debug_messages::*, debug_resources::*,
        debug_systems::*, debug_window_systems::*,
        gpos_maps_window::*,
        faction_details_inspector::*,
        macrochunks_grid_window::*,
        player_details_inspector::*, players_list_window::*, portals_list_window::*, region_details_inspector::*,
        regions_list_window::*, registered_positions_window::*, sprite_cfgs_details_inspector::*,
        sprite_cfgs_list_window::*, terrgen_editor_window::*, terrgen_values_window::*,
        tile_indices_map_window::*,
        world_tile_click_picker_window::*,
        tile_details_inspector::*,
        tilemap_details_inspector::*,
    };

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    let debug_enabled = |cfg: Res<DebugUiConfig>| cfg.enable_debug_menus;

    app.add_plugins((FpsCounterPlugin))
        .add_systems(Update, (apply_debug_ui_config_once,))
        .add_systems(
            Update,
            (
                setup_debug_fonts,
                debug_toggle_hot_reload_window,
                debug_toggle_main_menu,
                capture_world_tile_click_selection,
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            FixedUpdate,
            (
                debug_increase_speed,
                receive_speed_update_applied_from_server
                    .in_set(AcClientSystems)
                    .run_if(in_state(ClientState::Connected))
                    .run_if(on_message::<BeingDebugSpeedApplied>)
                    .before(disable_movement_while_speed_debug_update_pending),
                (disable_movement_while_speed_debug_update_pending)
                    .in_set(AcClientSystems)
                    .before(MovementSystems)
                    .run_if(in_state(ClientState::Connected)),
                receive_increase_speed_from_client
                    .after(debug_increase_speed)
                    .run_if(in_state(ServerState::Running))
                    .run_if(on_message::<FromClient<UpdateBeingSpeed>>),
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                main_menu_window,
                states_window,
                all_states_window,
                debug_chunking_window,
                macrochunks_grid_window,
                regions_list_window,
                beings_list_window,
                players_list_window,
                portals_list_window,
                sprites_list_window,
                terrgen_editor_window,
                terrgen_settings_editor_window,
                hot_reload_window,
                registered_positions_window,
                gpos_maps_window_system,
                tile_indices_map_window,
                world_tile_click_picker_window,
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
        .init_resource::<PendingSpeedDebugUpdates>()
        .init_resource::<DebugNoiseWorkshopState>()
        .init_resource::<DebugFontsInitialized>()
        .init_resource::<DebugUiConfig>()
        .init_resource::<WorldTileClickInspectorState>()
        .init_resource::<common::common_states::HotReloadSelection>()
        .init_resource::<common::common_states::HotReloadRequest>()
        .add_mapped_client_message::<UpdateBeingSpeed>(Channel::Ordered)
        .add_mapped_server_message::<BeingDebugSpeedApplied>(Channel::Ordered);
}
