
use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use std::collections::HashMap;

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
    pub terrgen_values: bool,
    pub settings_editor: bool,
    pub tile_details: bool,
    pub chunk_details: bool,
    pub region_details: bool,
    pub tilemap_details: bool,
    pub being_details: bool,
    pub registered_positions: bool,
    pub exempted_entity_details: bool,
    pub sprite_configs_list: bool,
    pub sprite_details: bool,
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
            terrgen_values: false,
            settings_editor: false,
            tile_details: false,
            chunk_details: false,
            region_details: false,
            tilemap_details: false,
            being_details: false,
            registered_positions: false,
            exempted_entity_details: false,
            sprite_configs_list: false,
            sprite_details: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct DebugSelectedEntities {
    pub selected_regions: EntityHashSet,
    pub selected_chunks: EntityHashSet,
    pub selected_portals: EntityHashSet,
    pub selected_operationlist: Option<Entity>,
    pub selected_noise: Option<Entity>,
    pub selected_tile: Option<Entity>,
    pub selected_being: Option<Entity>,
    pub selected_exempted_entity: Option<Entity>,
    pub selected_sprite: Option<Entity>,
    pub selected_tilemap: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct DebugChunkingUiState {
    pub follow_camera_chunk: bool,
    pub open_tilemap_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseCombineOp {
    Add,
    Subtract,
    Multiply,
    Average,
    Max,
    Min,
}

#[derive(Resource, Debug)]
pub struct DebugNoiseWorkshopState {
    pub selected_noises: Vec<Entity>,
    pub per_noise_subtract: HashMap<Entity, f32>,
    pub original_noises: HashMap<Entity, tilemap::terrain_gen::terrgen_components::FnlNoiseComp>,
    pub combine_op: NoiseCombineOp,
    pub threshold_enabled: bool,
    pub threshold: f32,
    pub preview_size_px: f32,
    pub preview_samples: usize,
    pub preview_zoom: f32,
}

impl Default for DebugNoiseWorkshopState {
    fn default() -> Self {
        Self {
            selected_noises: Vec::new(),
            per_noise_subtract: HashMap::new(),
            original_noises: HashMap::new(),
            combine_op: NoiseCombineOp::Average,
            threshold_enabled: false,
            threshold: 0.5,
            preview_size_px: 420.0,
            preview_samples: 64,
            preview_zoom: 1.0,
        }
    }
}
