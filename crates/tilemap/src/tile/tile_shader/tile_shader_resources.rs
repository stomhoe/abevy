#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

pub use crate::tile::tile_shader::tile_shader_seris::*;

use crate::tile::tile_shader::tile_shader_components::TileShader;

common::define_entity_map_systems!(
    TileShader,
    ShaderTerrblSeri, "seri.tilemap.tile_shader.terrbl", "terrbl.ron"
);
