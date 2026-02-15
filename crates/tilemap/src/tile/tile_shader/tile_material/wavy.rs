#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct WavyMat {
    // Overlay texture + sampler (bind as texture and sampler)
    #[texture(0)]
    #[sampler(1)]
    #[serde(skip)]
    pub texture_overlay: Handle<Image>,

    #[uniform(2)]
    pub mask_color: Vec4,

    #[uniform(3)]#[inspector(min = 1e-5, max = 1e2)]
    pub scale: f32,
    #[uniform(4)]
    pub time: f32,
    #[uniform(5)]
    pub speed: f32,
    #[uniform(6)]
    pub debug_mode: f32,
}

impl WavyMat {
    pub fn new(mask_color: Vec4, scale: f32, time: f32, speed: f32, debug_mode: f32) -> Self {
        Self { texture_overlay: Handle::default(), mask_color, scale, time, speed, debug_mode }
    }
}


impl PartialEq for WavyMat {
    fn eq(&self, other: &Self) -> bool {
        self.mask_color == other.mask_color
            && self.scale.to_bits() == other.scale.to_bits()
            && self.time.to_bits() == other.time.to_bits()
            && self.speed.to_bits() == other.speed.to_bits()
            && self.debug_mode.to_bits() == other.debug_mode.to_bits()
    }
}
impl Eq for WavyMat {}

impl Default for WavyMat {
    fn default() -> Self {
        Self {
            texture_overlay: Handle::default(),
            mask_color: Vec4::new(0.0, 0.5, 1.0, 0.6),
            scale: 1e-5,
            time: 0.0,
            speed: 1.0,
            debug_mode: 0.0,
        }
    }
}

impl MaterialTilemap for WavyMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/wavy.wgsl".into()
    }
}
