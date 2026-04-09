#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

pub use crate::tile::tile_shader::tile_shader_seris::*;

use crate::tile::tile_shader::tile_shader_components::TileShader;

common::define_entity_map_systems!(
    main_component: TileShader,
    with_filters: (),
    abbreviation: TileShader,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: TileShader,
    id_type: common::common_components::StrId,
    assets: [(ShaderTerrblSeri, "seri.tilemap.tile_shader.terrbl", "terrbl.ron")],
);
