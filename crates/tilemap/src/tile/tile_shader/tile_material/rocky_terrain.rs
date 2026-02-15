#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct RockyTerrainMat {
    #[uniform(1)]#[inspector(min = 0.0, max = 1.0)]
    pub roughness: f32,

    #[uniform(2)]#[inspector(min = 1e-5, max = 1e2)]
    pub scale: f32,

    #[uniform(3)]#[inspector(min = 0.0, max = 2.0)]
    pub height_scale: f32,

    #[uniform(4)]
    pub color_base: Vec4,

    #[uniform(5)]
    pub color_shadow: Vec4,
}

impl RockyTerrainMat {
    pub fn new(roughness: f32, scale: f32, height_scale: f32, color_base: Vec4, color_shadow: Vec4) -> Self {
        Self {
            roughness,
            scale,
            height_scale,
            color_base: color_base / 255.0,
            color_shadow: color_shadow / 255.0,
        }
    }
}



impl Default for RockyTerrainMat {
    fn default() -> Self {
        Self {
            roughness: 0.6,
            scale: 0.5,
            height_scale: 1.2,
            color_base: Vec4::new(0.6, 0.6, 0.6, 1.0),
            color_shadow: Vec4::new(0.3, 0.3, 0.3, 1.0),
        }
    }
}

impl PartialEq for RockyTerrainMat {
    fn eq(&self, other: &Self) -> bool {
        self.roughness.to_bits() == other.roughness.to_bits()
            && self.scale.to_bits() == other.scale.to_bits()
            && self.height_scale.to_bits() == other.height_scale.to_bits()
            && self.color_base == other.color_base
            && self.color_shadow == other.color_shadow
    }
}

impl Eq for RockyTerrainMat {}

impl MaterialTilemap for RockyTerrainMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/rocky_terrain.wgsl".into()
    }
}
