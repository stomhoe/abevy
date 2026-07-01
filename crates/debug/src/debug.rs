#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::SettingsEntity;
use common::common_states::AssetLoading;
use ::being_shared::*;
use tilemap_shared::DirectionalLight2dOverride;
use crate::debug_messages::*;
use crate::debug_speed_systems::*;
use debug_shared::*;

    use crate::{
        being_details_inspector::*, beings_list_window::*, chunk_details_inspector::*,
        being_click_remover::*,
        daylight_window::*,
        debug_chunking_window::*, debug_fonts::*,
        debug_systems::*, debug_window_systems::*,
        dimension_changer_window::*,
        gpos_maps_window::*,
        faction_details_inspector::*,
        nav_maps_window::*,
        macrochunks_grid_window::*,
        player_details_inspector::*, players_list_window::*, portals_list_window::*, region_details_inspector::*,
        regions_list_window::*, registered_positions_window::*, sprite_cfgs_details_inspector::*,
        sprite_cfgs_list_window::*, terrgen_values_window::*,
        inlandness_visualizer_window::*,
        tile_indices_map_window::*,
        click_picker_window::*,
        tile_click_remover::*,
        tile_details_inspector::*,
        tilemap_details_inspector::*,
    };

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    let debug_enabled = |cfg: Query<&DebugUiConfig, With<SettingsEntity>>| {
        cfg.single().map_or(false, |cfg| cfg.enable_debug_menus)
    };

    app.add_mapped_client_event::<ClientDebugSetSpeedRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_client_event::<ClientDebugSetBeingDimensionRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_client_event::<ClientDebugTeleportBeingRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_client_event::<ClientDebugSetBeingCurrentHpRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_client_event::<ClientDebugSetBeingCurrentBloodRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_client_event::<ClientDebugKillBeingRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_client_event::<ClientDebugReviveBeingRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_observer(receive_client_debug_set_speed_request)
        .add_observer(receive_client_debug_set_being_dimension_request)
        .add_observer(receive_client_debug_teleport_being_request)
        .add_observer(receive_client_debug_set_being_current_hp_request)
        .add_observer(receive_client_debug_set_being_current_blood_request)
        .add_observer(receive_client_debug_kill_being_request)
        .add_observer(receive_client_debug_revive_being_request)
        .add_observer(apply_local_debug_teleport_being_request)
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
                capture_world_being_click_removal,
                debug_numpad_speed_shortcuts,
            )
                .run_if(debug_enabled),
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
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                beings_list_window,
                players_list_window,
                portals_list_window,
                sprites_list_window,
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
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                gpos_maps_window_system,
                tile_indices_map_window,
                nav_maps_window,
                click_picker_window,
                terrgen_debug_window_system.run_if(|visible: Res<DubugWindowsVisibility>| visible.terrgen_values),
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                chunk_details_inspector,
                region_details_inspector,
                tilemap_details_inspector,
                tile_details_inspector,
            )
                .run_if(debug_enabled),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
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
        .init_resource::<DebugFontsInitialized>()
        .replicate::<DebugUiConfig>()
        .init_resource::<DirectionalLight2dOverride>()
        .init_resource::<WallPhaserOnSpawn>()
        .init_resource::<ClickInspectorState>()
        .init_resource::<TileClickRemoverState>()
        .init_resource::<BeingClickRemoverState>()
        .init_resource::<DebugBeingNavUiState>()
        .init_resource::<DebugBeingLocationEditorState>()
        .init_resource::<DebugBeingVitalsAdjustState>()
        .init_resource::<common::common_states::HotReloadSelection>();
}
