use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, };
pub use crate::tile::tile_shader::tile_shader_seris::*;

use crate::tile::tile_shader::tile_shader_components::TileShader;

common::define_entity_map_systems!(
    TileShader,
    ShaderRepeatTexSeri, "seri.tilemap.tile_shader.repeat_tex", "monorepeat.ron",
    //PlaceholderSeri, "ron/tilemap/tiling/shader/voroshu", "voroshu.ron",
    ShaderWavySeri, "seri.tilemap.tile_shader.wavy", "wavy.ron"
);
