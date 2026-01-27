
use bevy::prelude::*;

#[derive(Resource)]
pub struct DubugWindowsVisibility{
    pub states: bool,
    pub main_menu: bool,
    pub chunks_list: bool,
    pub regions_list: bool,
    pub beings_list: bool,
    pub terrgen_editor: bool,
}

impl Default for DubugWindowsVisibility {
    fn default() -> Self {
        Self {
            states: false,
            main_menu: true,
            chunks_list: false,
            regions_list: false,
            beings_list: false,
            terrgen_editor: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct DebugSelectedEntities {
    pub selected_regions: std::collections::HashSet<Entity>,
    pub selected_chunks: std::collections::HashSet<Entity>,
    pub selected_operationlist: Option<Entity>,
    pub selected_noise: Option<Entity>,
}
