
use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Resource, Debug, Clone)]
pub struct DubugWindowsVisibility{
    pub states: bool,
    pub main_menu: bool,
    pub chunks_list: bool,
    pub regions_list: bool,
    pub beings_list: bool,
    pub players_list: bool,
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
    pub player_details: bool,
    pub registered_positions: bool,
    pub exempted_entity_details: bool,
    pub sprite_configs_list: bool,
    pub sprite_details: bool,
    pub gpos_maps: bool,
    pub hot_reload_menu: bool,
    pub river_debug: bool,
    pub river_sample_values: bool,
}

impl Default for DubugWindowsVisibility {
    fn default() -> Self {
        Self {
            states: false,
            main_menu: true,
            chunks_list: false,
            regions_list: false,
            beings_list: false,
            players_list: false,
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
            player_details: false,
            registered_positions: false,
            exempted_entity_details: false,
            sprite_configs_list: false,
            sprite_details: false,
            gpos_maps: false,
            hot_reload_menu: false,
            river_debug: false,
            river_sample_values: false,
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
    pub selected_player: Option<Entity>,
    pub selected_exempted_entity: Option<Entity>,
    pub selected_sprite: Option<Entity>,
    pub selected_tilemap: Option<Entity>,
    pub selected_river_debug_region: Option<Entity>,
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

impl Default for DebugUiConfig {
    fn default() -> Self {
        Self {
            enable_debug_menus: true,
            hot_reload_defaults: common::common_states::HotReloadSelection::default(),
            windows_open_on_start: DubugWindowsVisibility::default(),
        }
    }
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct DebugUiConfigSeri {
    pub id: String,
    #[serde(default = "default_enable_debug_menus")]
    pub enable_debug_menus: bool,
    #[serde(default)]
    pub hot_reload_defaults: DebugHotReloadDefaultsSeri,
    #[serde(default)]
    pub windows_open_on_start: DebugWindowsOpenOnStartSeri,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct DebugHotReloadDefaultsSeri {
    #[serde(default)]
    pub tiles: bool,
    #[serde(default)]
    pub sprite_configs_and_animations: bool,
    #[serde(default)]
    pub terrain_oplists_and_noises: bool,
    #[serde(default)]
    pub probes_and_filters: bool,
    #[serde(default)]
    pub global_gen_settings: bool,
    #[serde(default)]
    pub beings_inst_templates: bool,
    #[serde(default)]
    pub races: bool,
    #[serde(default)]
    pub sexes: bool,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct DebugWindowsOpenOnStartSeri {
    #[serde(default)]
    pub main_menu: bool,
    #[serde(default)]
    pub hot_reload_menu: bool,
    #[serde(default)]
    pub terrgen_editor: bool,
    #[serde(default)]
    pub terrgen_values: bool,
    #[serde(default)]
    pub settings_editor: bool,
}

fn default_enable_debug_menus() -> bool { true }

impl DebugUiConfigSeri {
    pub fn to_config(&self) -> DebugUiConfig {
        DebugUiConfig {
            enable_debug_menus: self.enable_debug_menus,
            hot_reload_defaults: common::common_states::HotReloadSelection {
                tiles: self.hot_reload_defaults.tiles,
                sprite_configs_and_animations: self.hot_reload_defaults.sprite_configs_and_animations,
                terrain_oplists_and_noises: self.hot_reload_defaults.terrain_oplists_and_noises,
                probes_and_filters: self.hot_reload_defaults.probes_and_filters,
                global_gen_settings: self.hot_reload_defaults.global_gen_settings,
                beings_inst_templates: self.hot_reload_defaults.beings_inst_templates,
                races: self.hot_reload_defaults.races,
                sexes: self.hot_reload_defaults.sexes,
            },
            windows_open_on_start: {
                let mut v = DubugWindowsVisibility::default();
                v.main_menu = self.windows_open_on_start.main_menu;
                v.hot_reload_menu = self.windows_open_on_start.hot_reload_menu;
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
