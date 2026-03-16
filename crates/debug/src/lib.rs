
pub use debug::*;
pub mod debug;
mod debug_window_systems;
mod debug_chunking_window;
mod macrochunks_grid_window;
mod tile_details_inspector;
mod chunk_details_inspector;
mod region_details_inspector;
mod portals_details_inspector;
mod tilemap_details_inspector;
mod being_details_inspector;
mod player_details_inspector;
mod exempted_entity_details_inspector;
mod regions_list_window;
mod beings_list_window;
mod players_list_window;
mod portals_list_window;
mod terrgen_editor_window;
mod terrgen_values_window;
mod registered_positions_window;
mod sprite_cfgs_list_window;
mod sprite_cfgs_details_inspector;
mod debug_resources;
mod debug_seris;
mod debug_systems;
mod debug_fonts;
mod debug_messages;
mod gpos_maps_window;
mod world_tile_click_picker_window;
mod debug_ui_helpers;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        debug::*,
        debug_window_systems::*,
        debug_chunking_window::*,
        macrochunks_grid_window::*,
        tile_details_inspector::*,
        chunk_details_inspector::*,
        region_details_inspector::*,
        portals_details_inspector::*,
        tilemap_details_inspector::*,
        being_details_inspector::*,
        player_details_inspector::*,
        exempted_entity_details_inspector::*,
        regions_list_window::*,
        beings_list_window::*,
        players_list_window::*,
        portals_list_window::*,
        terrgen_editor_window::*,
        terrgen_values_window::*,
        registered_positions_window::*,
        sprite_cfgs_list_window::*,
        sprite_cfgs_details_inspector::*,
        debug_resources::*,
        debug_seris::*,
        debug_systems::*,
        debug_fonts::*,
        debug_messages::*,
        gpos_maps_window::*,
        world_tile_click_picker_window::*,
    };
}
