use bevy::prelude::*;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy_ecs_tilemap::tiles::TileColor;
use being_shared::{WallPhaserOnSpawn, InvulnerableOnSpawn};
use common::common_components::{HashId, SettingsEntity};
use common::common_states::HotReloadSelection;
use serde::{Deserialize, Serialize};
use tilemap_shared::GlobalTilePos;
use crate::debug_seris::*;

#[derive(Resource, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DubugWindowsVisibility {
    pub states: bool,
    pub all_states: bool,
    pub main_menu: bool,
    pub dimension_changer: bool,
    pub chunks_list: bool,
    pub macrochunks_grid: bool,
    pub regions_list: bool,
    pub beings_list: bool,
    pub players_list: bool,
    pub portals_list: bool,
    pub terrgen_values: bool,
    pub inlandness_visualizer: bool,
    pub settings_editor: bool,
    pub daylight: bool,
    pub tile_details: bool,
    pub chunk_details: bool,
    pub region_details: bool,
    pub tilemap_details: bool,
    pub being_details: bool,
    pub faction_details: bool,
    pub player_details: bool,
    pub registered_positions: bool,
    pub exempted_entity_details: bool,
    pub sprite_configs_list: bool,
    pub sprite_details: bool,
    pub gpos_maps: bool,
    pub tile_indices_map: bool,
    pub nav_maps: bool,
    pub click_picker: bool,
    pub tile_click_remover: bool,
    pub being_click_remover: bool,
    pub hot_reload_window_open_on_start: bool,
    pub river_debug: bool,
}

#[derive(Resource, Debug, Default)]
pub struct DebugBeingLocationEditorState {
    pub last_selected_being_entity: Option<Entity>,
    pub gpos_x_text: String,
    pub gpos_y_text: String,
    pub teleport_error: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BeingVitalsAdjustState {
    pub current_hp: f32,
    pub blood: f32,
    pub hp_dragging: bool,
    pub blood_dragging: bool,
    pub hp_pending_target: Option<f32>,
    pub blood_pending_target: Option<f32>,
}

#[derive(Resource, Debug, Default)]
pub struct DebugBeingVitalsAdjustState {
    pub states: EntityHashMap<BeingVitalsAdjustState>,
}

impl Default for DubugWindowsVisibility {
    fn default() -> Self {
        Self {
            states: false,
            all_states: false,
            main_menu: false,
            dimension_changer: false,
            chunks_list: false,
            macrochunks_grid: false,
            regions_list: false,
            beings_list: false,
            players_list: false,
            portals_list: false,
            terrgen_values: false,
            inlandness_visualizer: false,
            settings_editor: false,
            daylight: false,
            tile_details: false,
            chunk_details: false,
            region_details: false,
            tilemap_details: false,
            being_details: false,
            faction_details: false,
            player_details: false,
            registered_positions: false,
            exempted_entity_details: false,
            sprite_configs_list: false,
            sprite_details: false,
            gpos_maps: false,
            tile_indices_map: false,
            nav_maps: false,
            click_picker: false,
            tile_click_remover: false,
            being_click_remover: false,
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
    pub selected_tiles: EntityHashSet,
    pub selected_being: Option<Entity>,
    pub selected_beings: EntityHashSet,
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
    pub river_samples_show_sources: bool,
    pub river_samples_show_mouths: bool,
    pub river_samples_show_region_bounds: bool,
    pub river_samples_show_failed_centers: bool,
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
            selected_tiles: EntityHashSet::default(),
            selected_being: None,
            selected_beings: EntityHashSet::default(),
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
            river_samples_show_sources: true,
            river_samples_show_mouths: true,
            river_samples_show_region_bounds: true,
            river_samples_show_failed_centers: true,
        }
    }
}

#[derive(Resource)]
pub struct ClickInspectorState {
    pub enabled: bool,
    pub picking_enabled: bool,
    pub picker_side: usize,
    pub mult_being_windows: bool,
    pub mult_tile_windows: bool,
    pub auto_open_being_details: bool,
    pub auto_open_tile_details: bool,
    pub clicked_dim: Option<HashId>,
    pub clicked_gpos: Option<tilemap_shared::GlobalTilePos>,
    pub highlighted_center_tile: Option<Entity>,
    pub highlighted_center_tile_original_color: Option<TileColor>,
}

impl ClickInspectorState {
    pub fn clear_picker_selection(&mut self) {
        self.clicked_dim = None;
        self.clicked_gpos = None;
    }
}

impl Default for ClickInspectorState {
    fn default() -> Self {
        Self {
            enabled: false,
            picking_enabled: true,
            picker_side: 5,
            mult_being_windows: false,
            mult_tile_windows: false,
            auto_open_being_details: true,
            auto_open_tile_details: true,
            clicked_dim: None,
            clicked_gpos: None,
            highlighted_center_tile: None,
            highlighted_center_tile_original_color: None,
        }
    }
}

#[derive(Resource, Debug)]
pub struct TileClickRemoverState {
    pub despawn_last_tile: bool,
    pub inactivity_timer: Timer,
}

impl Default for TileClickRemoverState {
    fn default() -> Self {
        Self {
            despawn_last_tile: false,
            inactivity_timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

impl TileClickRemoverState {
    pub fn reset_inactivity_timer(&mut self) {
        self.inactivity_timer.reset();
    }
}

#[derive(Resource, Debug)]
pub struct BeingClickRemoverState {
    pub inactivity_timer: Timer,
}

impl Default for BeingClickRemoverState {
    fn default() -> Self {
        Self {
            inactivity_timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

impl BeingClickRemoverState {
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

#[derive(Component, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DebugUiConfig {
    pub enable_debug_menus: bool,
    pub wall_phaser: bool,
    pub invulnerable: bool,
    pub client_debug: bool,
    pub hot_reload_defaults: common::common_states::HotReloadSelection,
    pub windows_open_on_start: DubugWindowsVisibility,
}

pub fn load_debug_ui_config(
    mut cmd: Commands,
    settings_entity_query: Query<Entity, With<SettingsEntity>>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selection: ResMut<HotReloadSelection>,
    mut wall_phaser_on_spawn: ResMut<WallPhaserOnSpawn>,
    mut invulnerable_on_spawn: ResMut<InvulnerableOnSpawn>,
) {
    let defs = load_debug_ui_config_seri_defs();
    let Some(def) = defs.first() else {
        return;
    };

    let cfg = def.to_config();
    if let Ok(settings_entity) = settings_entity_query.single() {
        cmd.entity(settings_entity).insert(cfg.clone());
    }

    *selection = cfg.hot_reload_defaults.clone();
    *window_visible = cfg.windows_open_on_start.clone();
    wall_phaser_on_spawn.0 = cfg.wall_phaser;
    invulnerable_on_spawn.0 = cfg.invulnerable;
}

impl Default for DebugUiConfig {
    fn default() -> Self {
        Self {
            enable_debug_menus: true,
            wall_phaser: false,
            invulnerable: false,
            client_debug: false,
            hot_reload_defaults: common::common_states::HotReloadSelection::default(),
            windows_open_on_start: DubugWindowsVisibility::default(),
        }
    }
}

impl DebugUiConfigSeri {
    pub fn to_config(&self) -> DebugUiConfig {
        DebugUiConfig {
            enable_debug_menus: self.enable_debug_menus,
            wall_phaser: self.wall_phaser,
            invulnerable: self.invulnerable,
            client_debug: self.client_debug,
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
                v.terrgen_values = self.windows_open_on_start.terrgen_values;
                v.settings_editor = self.windows_open_on_start.settings_editor;
                v.daylight = self.windows_open_on_start.daylight;
                v.nav_maps = self.windows_open_on_start.nav_maps;
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
