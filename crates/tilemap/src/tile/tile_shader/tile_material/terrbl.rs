#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct TerrBlendMat {
    #[texture(1)]
    #[sampler(2)]
    #[serde(skip)]
    pub texture_overlay: Handle<Image>,
    #[uniform(3)]
    pub mask_color: Vec4,
    #[uniform(4)]#[inspector(min = 1e-5, max = 1e-3)]
    pub scale: f32,
    #[uniform(5)]
    pub time: f32,
    #[uniform(6)]
    pub speed: f32,
    #[uniform(7)]
    pub wavy_strength: f32,
}

impl TerrBlendMat {
    pub fn new(
        texture_overlay: Handle<Image>,
        mask_color: Vec4,
        scale: f32,
        speed: f32,
        wavy_strength: f32,
    ) -> Self {
        Self {
            texture_overlay,
            mask_color: mask_color / 255.0,
            scale,
            time: 0.0,
            speed,
            wavy_strength,
        }
    }
}

//https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/struct.InspectorOptions.html
impl PartialEq for TerrBlendMat {
    fn eq(&self, other: &Self) -> bool {
        self.texture_overlay == other.texture_overlay
            && self.mask_color == other.mask_color
            && self.scale.to_bits() == other.scale.to_bits()
            && self.time.to_bits() == other.time.to_bits()
            && self.speed.to_bits() == other.speed.to_bits()
            && self.wavy_strength.to_bits() == other.wavy_strength.to_bits()
    }
}
impl Eq for TerrBlendMat {}

impl Default for TerrBlendMat {
    fn default() -> Self {
        Self {
            texture_overlay: Handle::default(),
            mask_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            scale: 1e-5,
            time: 0.0,
            speed: 0.0,
            wavy_strength: 0.0,
        }
    }
}
impl MaterialTilemap for TerrBlendMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/terrbl.wgsl".into()
    }
}
