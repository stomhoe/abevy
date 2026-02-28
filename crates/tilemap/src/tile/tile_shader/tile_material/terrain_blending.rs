#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct TerrainBlendingMat {
    #[texture(1)]
    #[sampler(2)]
    #[serde(skip)]
    pub texture_a: Handle<Image>,

    #[texture(3)]
    #[sampler(4)]
    #[serde(skip)]
    pub texture_b: Handle<Image>,

    #[uniform(5)]
    pub mask_color: Vec4,

    #[uniform(6)]#[inspector(min = 1e-5, max = 1e2)]
    pub scale_a: f32,

    #[uniform(7)]#[inspector(min = 1e-5, max = 1e2)]
    pub scale_b: f32,

    #[uniform(8)]#[inspector(min = 0.0, max = 1.0)]
    pub blend_sharpness: f32,

    #[uniform(9)]#[inspector(min = 0.0, max = 1.0)]
    pub noise_strength: f32,

    #[uniform(10)]#[inspector(min = 0.0, max = 0.1)]
    pub jitter_strength: f32,
}

impl TerrainBlendingMat {
    pub fn new(mask_color: Vec4, scale_a: f32, scale_b: f32, blend_sharpness: f32, noise_strength: f32, jitter_strength: f32) -> Self {
        Self {
            texture_a: Handle::default(),
            texture_b: Handle::default(),
            mask_color: mask_color / 255.0,
            scale_a,
            scale_b,
            blend_sharpness,
            noise_strength,
            jitter_strength,
        }
    }
}



impl Default for TerrainBlendingMat {
    fn default() -> Self {
        Self {
            texture_a: Handle::default(),
            texture_b: Handle::default(),
            mask_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            scale_a: 8.0,
            scale_b: 8.0,
            blend_sharpness: 0.0,
            noise_strength: 0.28,
            jitter_strength: 0.015,
        }
    }
}

impl PartialEq for TerrainBlendingMat {
    fn eq(&self, other: &Self) -> bool {
        self.texture_a == other.texture_a
            && self.texture_b == other.texture_b
            && self.mask_color == other.mask_color
            && self.scale_a.to_bits() == other.scale_a.to_bits()
            && self.scale_b.to_bits() == other.scale_b.to_bits()
            && self.blend_sharpness.to_bits() == other.blend_sharpness.to_bits()
            && self.noise_strength.to_bits() == other.noise_strength.to_bits()
            && self.jitter_strength.to_bits() == other.jitter_strength.to_bits()
    }
}

impl Eq for TerrainBlendingMat {}

impl MaterialTilemap for TerrainBlendingMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/terrain_blending.wgsl".into()
    }
}
