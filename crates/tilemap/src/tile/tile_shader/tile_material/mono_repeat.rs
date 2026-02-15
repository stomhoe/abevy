#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct MonoRepeatTextureOverlayMat {
    #[texture(1)]
    #[sampler(2)]
    #[serde(skip)]
    pub texture_overlay: Handle<Image>,
    #[uniform(3)]
    pub mask_color: Vec4,
    #[uniform(4)]#[inspector(min = 1e-5, max = 1e-3)]
    pub scale: f32,
    #[uniform(5)]
    pub tint_color: Vec4,
}

impl MonoRepeatTextureOverlayMat {
    pub fn new(texture_overlay: Handle<Image>, mask_color: Vec4, scale: f32, tint_color: Vec4) -> Self {
        Self { texture_overlay, mask_color: mask_color / 255.0, scale, tint_color }
    }
}

//https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/struct.InspectorOptions.html
impl PartialEq for MonoRepeatTextureOverlayMat {
    fn eq(&self, other: &Self) -> bool {
        self.texture_overlay == other.texture_overlay
            && self.mask_color == other.mask_color
            && self.scale.to_bits() == other.scale.to_bits()
            && self.tint_color == other.tint_color
    }
}
impl Eq for MonoRepeatTextureOverlayMat {}

impl Default for MonoRepeatTextureOverlayMat {
    fn default() -> Self {
        Self {
            texture_overlay: Handle::default(),
            mask_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            scale: 1e-5,
            tint_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
        }
    }
}
impl MaterialTilemap for MonoRepeatTextureOverlayMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/textured_tile.wgsl".into()
    }
}
