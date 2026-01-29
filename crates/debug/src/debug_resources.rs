
use bevy::prelude::*;

#[derive(Resource)]
pub struct DubugWindowsVisibility{
    pub states: bool,
    pub main_menu: bool,
    pub chunks_list: bool,
    pub regions_list: bool,
    pub beings_list: bool,
    pub portals_list: bool,
    pub portal_details: bool,
    pub terrgen_editor: bool,
    pub settings_editor: bool,
    pub tile_details: bool,
    pub chunk_details: bool,
    pub tilemap_details: bool,
    pub being_details: bool,
    pub registered_positions: bool,
}

impl Default for DubugWindowsVisibility {
    fn default() -> Self {
        Self {
            states: false,
            main_menu: true,
            chunks_list: false,
            regions_list: false,
            beings_list: false,
            portals_list: false,
            portal_details: false,
            terrgen_editor: false,
            settings_editor: false,
            tile_details: false,
            chunk_details: false,
            tilemap_details: false,
            being_details: false,
            registered_positions: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct DebugSelectedEntities {
    pub selected_regions: std::collections::HashSet<Entity>,
    pub selected_chunks: std::collections::HashSet<Entity>,
    pub selected_portals: std::collections::HashSet<Entity>,
    pub selected_operationlist: Option<Entity>,
    pub selected_noise: Option<Entity>,
    pub selected_tile: Option<Entity>,
    pub selected_being: Option<Entity>,
}
