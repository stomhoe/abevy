#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;

use crate::{
    being_details_inspector::*, beings_list_window::*, chunk_details_inspector::*,
    debug_chunking_window::*, debug_fonts::*, debug_messages::*, debug_resources::*,
    debug_systems::*, debug_window_systems::*, exempted_entity_details_inspector::*,
    player_details_inspector::*, players_list_window::*, portals_details_inspector::*, portals_list_window::*, region_details_inspector::*,
    regions_list_window::*, registered_positions_window::*, sprite_cfgs_details_inspector::*,
    sprite_cfgs_list_window::*, terrgen_editor_window::*, terrgen_values_window::*,
    tile_details_inspector::*,
    tilemap_details_inspector::*,
};

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((FpsCounterPlugin))
        .add_systems(
            Update,
            (
                setup_debug_fonts,
                debug_toggle_hot_reload_window,
                debug_toggle_main_menu,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                debug_increase_speed,
                receive_increase_speed_from_client.run_if(in_state(ServerState::Running)),
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                main_menu_window,
                states_window,
                debug_chunking_window,
                regions_list_window,
                beings_list_window,
                players_list_window,
                portals_list_window,
                sprites_list_window,
                terrgen_editor_window,
                global_gen_settings_editor_window,
                hot_reload_window,
                registered_positions_window,
                terrgen_debug_window_system
                    .run_if(|visible: Res<DubugWindowsVisibility>| visible.terrgen_values),
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                tile_details_inspector,
                chunk_details_inspector,
                region_details_inspector,
                portals_details_inspector,
                tilemap_details_inspector,
                being_details_inspector,
                player_details_inspector,
                exempted_entity_details_inspector,
                sprite_details_inspector,
            ),
        )
        .init_resource::<DubugWindowsVisibility>()
        .init_resource::<DebugSelectedEntities>()
        .init_resource::<DebugChunkingUiState>()
        .init_resource::<DebugNoiseWorkshopState>()
        .init_resource::<DebugFontsInitialized>()
        .init_resource::<common::common_states::HotReloadSelection>()
        .init_resource::<common::common_states::HotReloadRequest>()
        .add_mapped_client_message::<UpdateBeingSpeed>(Channel::Ordered);
}
