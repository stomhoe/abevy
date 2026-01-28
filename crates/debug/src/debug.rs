#[allow(unused_imports)] use bevy::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiPrimaryContextPass, };

use crate::{debug_resources::*, debug_window_systems::*, debug_entity_lists::*, debug_systems::*, debug_fonts::*};

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        FpsCounterPlugin
    ))
    .add_systems(Update, setup_debug_fonts)
    .add_systems(Update, (
        debug_increase_speed,
        debug_toggle_states_window,
        debug_toggle_main_menu,
    ))
    .add_systems(EguiPrimaryContextPass, (
        main_menu_window,
        states_window,
        chunks_list_window,
        regions_list_window,
        beings_list_window,
        terrgen_editor_window,
        global_gen_settings_editor_window,
    ))
    .add_systems(EguiPrimaryContextPass, (
        tile_details_inspector,
        chunk_details_inspector,
        tilemap_details_inspector,
        being_details_inspector,
    ))
    .init_resource::<DubugWindowsVisibility>()
    .init_resource::<DebugSelectedEntities>()
    .init_resource::<FontsInitialized>()
    
    ;
}





