#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct TerrBlendMat {
    #[texture(1)]
    #[serde(skip)]
    pub tile_indices_map: Handle<Image>,
    #[texture(2)]
    #[serde(skip)]
    pub tile_flags_map: Handle<Image>,
    #[texture(3)]
    #[serde(skip)]
    pub tile_params_map: Handle<Image>,
    #[uniform(4)]
    pub map_size_tiles: Vec2,
    #[uniform(5)]
    pub time: f32,
}

impl TerrBlendMat {
    pub fn new() -> Self {
        Self {
            tile_indices_map: Handle::default(),
            tile_flags_map: Handle::default(),
            tile_params_map: Handle::default(),
            map_size_tiles: Vec2::ONE,
            time: 0.0,
        }
    }
}

//https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/struct.InspectorOptions.html
impl PartialEq for TerrBlendMat {
    fn eq(&self, other: &Self) -> bool {
        self.tile_indices_map == other.tile_indices_map
            && self.tile_flags_map == other.tile_flags_map
            && self.tile_params_map == other.tile_params_map
            && self.map_size_tiles == other.map_size_tiles
            && self.time.to_bits() == other.time.to_bits()
    }
}
impl Eq for TerrBlendMat {}

impl Default for TerrBlendMat {
    fn default() -> Self {
        Self {
            tile_indices_map: Handle::default(),
            tile_flags_map: Handle::default(),
            tile_params_map: Handle::default(),
            map_size_tiles: Vec2::ONE,
            time: 0.0,
        }
    }
}
impl MaterialTilemap for TerrBlendMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/terrbl.wgsl".into()
    }
}
