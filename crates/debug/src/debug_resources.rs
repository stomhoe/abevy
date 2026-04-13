
use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
pub use crate::debug_seris::*;
use common::common_components::HashId;
use common::common_states::HotReloadSelection;
use std::collections::HashMap;

use tilemap_shared::GlobalTilePos;

#[derive(Resource, Debug, Clone)]
pub struct DubugWindowsVisibility{
    pub states: bool,
    pub all_states: bool,
    pub main_menu: bool,
    pub chunks_list: bool,
    pub macrochunks_grid: bool,
    pub regions_list: bool,
    pub beings_list: bool,
    pub players_list: bool,
    pub portals_list: bool,
    pub terrgen_editor: bool,
    pub terrgen_values: bool,
    pub terrain_visualizer: bool,
    pub settings_editor: bool,
    pub tile_details: bool,
    pub chunk_details: bool,
    pub region_details: bool,
    pub tilemap_details: bool,
    pub being_details: bool,
    pub being_nav_log: bool,
    pub being_tile_click_picker: bool,
    pub faction_details: bool,
    pub player_details: bool,
    pub registered_positions: bool,
    pub exempted_entity_details: bool,
    pub sprite_configs_list: bool,
    pub sprite_details: bool,
    pub gpos_maps: bool,
    pub tile_indices_map: bool,
    pub world_tile_click_picker: bool,
    pub hot_reload_window_open_on_start: bool,
    pub river_debug: bool,
}

impl Default for DubugWindowsVisibility {
    fn default() -> Self {
        Self {
            states: false,
            all_states: false,
            main_menu: true,
            chunks_list: false,
            macrochunks_grid: false,
            regions_list: false,
            beings_list: false,
            players_list: false,
            portals_list: false,
            terrgen_editor: false,
            terrgen_values: false,
            terrain_visualizer: false,
            settings_editor: false,
            tile_details: false,
            chunk_details: false,
            region_details: false,
            tilemap_details: false,
            being_details: false,
            being_nav_log: false,
            being_tile_click_picker: false,
            faction_details: false,
            player_details: false,
            registered_positions: false,
            exempted_entity_details: false,
            sprite_configs_list: false,
            sprite_details: false,
            gpos_maps: false,
            tile_indices_map: false,
            world_tile_click_picker: false,
            hot_reload_window_open_on_start: false,
            river_debug: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct DebugBeingNavUiState {
    pub track_new_being: bool,
    pub last_clicked_dim: Option<HashId>,
    pub last_clicked_gpos: Option<GlobalTilePos>,
    pub last_selected_being: Option<Entity>,
}

#[derive(Resource)]
pub struct DebugSelectedEntities {
    pub selected_regions: EntityHashSet,
    pub selected_chunks: EntityHashSet,
    pub selected_macrochunk: Option<Entity>,
    pub selected_portals: EntityHashSet,
    pub selected_operationlist: Option<Entity>,
    pub selected_noise: Option<Entity>,
    pub selected_tile: Option<Entity>,
    pub selected_being: Option<Entity>,
    pub selected_being_interaction_zone: Option<HashId>,
    pub selected_being_bodypart: Option<Entity>,
    pub show_full_being_components: bool,
    pub selected_faction: Option<Entity>,
    pub show_full_faction_components: bool,
    pub selected_player: Option<Entity>,
    pub selected_exempted_entity: Option<Entity>,
    pub selected_sprite: Option<Entity>,
    pub selected_tilemap: Option<Entity>,
    pub selected_river_debug_region: Option<Entity>,
    pub river_samples_show_river_overlay: bool,
    pub river_samples_show_sources: bool,
    pub river_samples_show_mouths: bool,
    pub river_samples_show_camera_tile: bool,
    pub river_samples_show_region_bounds: bool,
    pub river_samples_show_failed_centers: bool,
    pub river_samples_show_none_points: bool,
}
impl Default for DebugSelectedEntities {
    fn default() -> Self {
        Self {
            selected_regions: EntityHashSet::default(),
            selected_chunks: EntityHashSet::default(),
            selected_macrochunk: None,
            selected_portals: EntityHashSet::default(),
            selected_operationlist: None,
            selected_noise: None,
            selected_tile: None,
            selected_being: None,
            selected_being_interaction_zone: None,
            selected_being_bodypart: None,
            show_full_being_components: false,
            selected_faction: None,
            show_full_faction_components: false,
            selected_player: None,
            selected_exempted_entity: None,
            selected_sprite: None,
            selected_tilemap: None,
            selected_river_debug_region: None,
            river_samples_show_river_overlay: true,
            river_samples_show_sources: true,
            river_samples_show_mouths: true,
            river_samples_show_camera_tile: true,
            river_samples_show_region_bounds: true,
            river_samples_show_failed_centers: true,
            river_samples_show_none_points: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct WorldTileClickInspectorState {
    pub enabled: bool,
    pub clicked_dim: Option<HashId>,
    pub clicked_gpos: Option<tilemap_shared::GlobalTilePos>,
    pub entities_at_gpos: Vec<Entity>,
    pub click_generation: u64,
    pub last_opened_click_generation: u64,
}

#[derive(Resource, Debug)]
pub struct BeingTileClickInspectorState {
    pub last_clicked_dim: Option<HashId>,
    pub last_clicked_gpos: Option<tilemap_shared::GlobalTilePos>,
    pub last_selected_being: Option<Entity>,
    pub inactivity_timer: Timer,
}

impl Default for BeingTileClickInspectorState {
    fn default() -> Self {
        Self {
            last_clicked_dim: None,
            last_clicked_gpos: None,
            last_selected_being: None,
            inactivity_timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

impl BeingTileClickInspectorState {
    pub fn clear_selection(&mut self) {
        self.last_clicked_dim = None;
        self.last_clicked_gpos = None;
        self.last_selected_being = None;
    }

    pub fn reset_inactivity_timer(&mut self) {
        self.inactivity_timer.reset();
    }
}

#[derive(Resource)]
pub struct DebugChunkingUiState {
    pub follow_camera_chunk: bool,
    pub follow_camera_region: bool,
    pub follow_camera_macrochunk: bool,
    pub open_tilemap_type: Option<String>,
    pub chunk_details_open_nonce: u64,
    pub selected_macrochunk_chunk: Option<tilemap_shared::ChunkPos>,
}

impl Default for DebugChunkingUiState {
    fn default() -> Self {
        Self {
            follow_camera_chunk: true,
            follow_camera_region: true,
            follow_camera_macrochunk: true,
            open_tilemap_type: None,
            chunk_details_open_nonce: 0,
            selected_macrochunk_chunk: None,
        }
    }
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
    pub original_noises: HashMap<Entity, tilemap::terrain::terrgen_components::FnlNoiseComp>,
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

#[derive(Resource, Debug, Clone)]
pub struct DebugUiConfig {
    pub enable_debug_menus: bool,
    pub hot_reload_defaults: common::common_states::HotReloadSelection,
    pub windows_open_on_start: DubugWindowsVisibility,
}

pub fn load_debug_ui_config(
    mut cfg: ResMut<DebugUiConfig>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selection: ResMut<HotReloadSelection>,
) {
    let defs = load_debug_ui_config_seri_defs();
    let Some(def) = defs.first() else {
        return;
    };

    *cfg = def.to_config();
    *selection = cfg.hot_reload_defaults.clone();
    *window_visible = cfg.windows_open_on_start.clone();
}

impl Default for DebugUiConfig {
    fn default() -> Self {
        Self {
            enable_debug_menus: true,
            hot_reload_defaults: common::common_states::HotReloadSelection::default(),
            windows_open_on_start: DubugWindowsVisibility::default(),
        }
    }
}

impl DebugUiConfigSeri {
    pub fn to_config(&self) -> DebugUiConfig {
        DebugUiConfig {
            enable_debug_menus: self.enable_debug_menus,
            hot_reload_defaults: common::common_states::HotReloadSelection {
                tiles: self.hot_reload_defaults.tiles,
                sprite_configs_and_animations: self.hot_reload_defaults.sprite_configs_and_animations,
                terrain_oplists_and_noises: self.hot_reload_defaults.terrain_oplists_and_noises,
                probes_and_filters: self.hot_reload_defaults.probes_and_filters,
                terrgen_settings: self.hot_reload_defaults.terrgen_settings,
                beings_inst_templates: self.hot_reload_defaults.beings_inst_templates,
                races: self.hot_reload_defaults.races,
                sexes: self.hot_reload_defaults.sexes,
            },
            windows_open_on_start: {
                let mut v = DubugWindowsVisibility::default();
                v.main_menu = self.windows_open_on_start.main_menu;
                v.hot_reload_window_open_on_start = self.windows_open_on_start.hot_reload_window_open_on_start;
                v.all_states = self.windows_open_on_start.all_states;
                v.terrgen_editor = self.windows_open_on_start.terrgen_editor;
                v.terrgen_values = self.windows_open_on_start.terrgen_values;
                v.settings_editor = self.windows_open_on_start.settings_editor;
                v
            },
        }
    }
}

pub fn load_debug_ui_config_seri_defs() -> Vec<DebugUiConfigSeri> {
    let db = match common::def_db::DefDatabase::<DebugUiConfigSeri>::load_from_assets_dir_with_type(
        stringify!(DebugUiConfigSeri),
        &["debug_ui.settings.ron"],
        |_| "debug_ui",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(
                target: "debug",
                "Failed loading DebugUiConfigSeri defs: {err:#}"
            );
            return Vec::new();
        }
    };
    db.into_records().into_iter().map(|r| r.value).collect()
}
