#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath, InspectorOptions, Deserialize, Serialize)]
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
    #[texture(14)]
    #[serde(skip)]
    pub tile_tint_map: Handle<Image>,
    #[uniform(4)]
    pub map_size_tiles: Vec2,
    #[uniform(5)]
    pub time: f32,
    #[texture(6)]
    #[serde(skip)]
    pub overlay_tex_0: Handle<Image>,
    #[texture(7)]
    #[serde(skip)]
    pub overlay_tex_1: Handle<Image>,
    #[texture(8)]
    #[serde(skip)]
    pub overlay_tex_2: Handle<Image>,
    #[texture(9)]
    #[serde(skip)]
    pub overlay_tex_3: Handle<Image>,
    #[texture(10)]
    #[serde(skip)]
    pub overlay_tex_4: Handle<Image>,
    #[texture(11)]
    #[serde(skip)]
    pub overlay_tex_5: Handle<Image>,
    #[texture(12)]
    #[serde(skip)]
    pub overlay_tex_6: Handle<Image>,
    #[texture(13)]
    #[serde(skip)]
    pub overlay_tex_7: Handle<Image>,
}
impl PartialEq for TerrBlendMat {
    fn eq(&self, other: &Self) -> bool {
        self.tile_indices_map == other.tile_indices_map
            && self.tile_flags_map == other.tile_flags_map
            && self.tile_params_map == other.tile_params_map
            && self.tile_tint_map == other.tile_tint_map
            && self.map_size_tiles == other.map_size_tiles
            && self.time.to_bits() == other.time.to_bits()
            && self.overlay_tex_0 == other.overlay_tex_0
            && self.overlay_tex_1 == other.overlay_tex_1
            && self.overlay_tex_2 == other.overlay_tex_2
            && self.overlay_tex_3 == other.overlay_tex_3
            && self.overlay_tex_4 == other.overlay_tex_4
            && self.overlay_tex_5 == other.overlay_tex_5
            && self.overlay_tex_6 == other.overlay_tex_6
            && self.overlay_tex_7 == other.overlay_tex_7
    }
}
impl Eq for TerrBlendMat {}

impl Default for TerrBlendMat {
    fn default() -> Self {
        Self {
            tile_indices_map: Handle::default(),
            tile_flags_map: Handle::default(),
            tile_params_map: Handle::default(),
            tile_tint_map: Handle::default(),
            map_size_tiles: Vec2::ONE,
            time: 0.0,
            overlay_tex_0: Handle::default(),
            overlay_tex_1: Handle::default(),
            overlay_tex_2: Handle::default(),
            overlay_tex_3: Handle::default(),
            overlay_tex_4: Handle::default(),
            overlay_tex_5: Handle::default(),
            overlay_tex_6: Handle::default(),
            overlay_tex_7: Handle::default(),
        }
    }
}
impl MaterialTilemap for TerrBlendMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/terrbl.wgsl".into()
    }
}
