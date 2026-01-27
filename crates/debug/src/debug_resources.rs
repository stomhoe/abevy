
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct DubugWindowsVisibility{
    pub states: bool,
    pub main_menu: bool,
    pub chunks_list: bool,
    pub regions_list: bool,
    pub beings_list: bool,
}

#[derive(Resource, Default)]
pub struct DebugSelectedEntities {
    pub selected_regions: std::collections::HashSet<Entity>,
    pub selected_chunks: std::collections::HashSet<Entity>,
}
