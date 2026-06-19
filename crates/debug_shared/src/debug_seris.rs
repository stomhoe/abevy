use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct DebugUiConfigSeri {
    pub id: String,
    #[serde(default = "default_enable_debug_menus")]
    pub enable_debug_menus: bool,
    #[serde(default)]
    pub wall_phaser: bool,
    #[serde(default)]
    pub invulnerable: bool,
    #[serde(default)]
    pub client_debug: bool,
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
    pub terrgen_settings: bool,
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
    pub hot_reload_window_open_on_start: bool,
    #[serde(default)]
    pub all_states: bool,
    #[serde(default)]
    pub terrgen_editor: bool,
    #[serde(default)]
    pub terrgen_values: bool,
    #[serde(default)]
    pub settings_editor: bool,
    #[serde(default)]
    pub daylight: bool,
    #[serde(default)]
    pub nav_maps: bool,
}

fn default_enable_debug_menus() -> bool { true }
