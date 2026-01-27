#[allow(unused_imports)] use bevy::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiPrimaryContextPass, };

use crate::{debug_resources::*, debug_window_systems::*, debug_entity_lists::*, debug_systems::*, };

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        FpsCounterPlugin
    ))
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
    ))
    .init_resource::<DubugWindowsVisibility>()
    .init_resource::<DebugSelectedEntities>()
    
    ;
}




