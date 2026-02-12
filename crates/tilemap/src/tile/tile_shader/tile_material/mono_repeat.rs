#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::ser::SerializeStruct;
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct MonoRepeatTextureOverlayMat {
    #[texture(1)]
    #[sampler(2)]
    pub texture_overlay: Handle<Image>,
    #[uniform(3)]
    pub mask_color: Vec4,
    #[uniform(4)]#[inspector(min = 1e-5, max = 1e-3)]
    pub scale: f32,
}

impl MonoRepeatTextureOverlayMat {
    pub fn new(texture_overlay: Handle<Image>, mask_color: Vec4, scale: f32) -> Self {
        Self { texture_overlay, mask_color: mask_color / 255.0, scale }
    }
}
impl Serialize for MonoRepeatTextureOverlayMat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut state = serializer.serialize_struct("MonoRepeatTextureOverlayMat", 3)?;
        state.serialize_field("mask_color", &self.mask_color)?;
        state.serialize_field("scale", &self.scale)?;
        state.end()
    }
}
impl<'de> Deserialize<'de> for MonoRepeatTextureOverlayMat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct MonoRepeatTextureOverlayMatData {
            mask_color: Vec4,
            scale: f32,
        }
        let data = MonoRepeatTextureOverlayMatData::deserialize(deserializer)?;
        Ok(MonoRepeatTextureOverlayMat {
            texture_overlay: Handle::default(),
            mask_color: data.mask_color,
            scale: data.scale,
        })
    }
}
//https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/struct.InspectorOptions.html
impl PartialEq for MonoRepeatTextureOverlayMat {
    fn eq(&self, other: &Self) -> bool {
        self.texture_overlay == other.texture_overlay
            && self.mask_color == other.mask_color
            && self.scale.to_bits() == other.scale.to_bits()
    }
}
impl Eq for MonoRepeatTextureOverlayMat {}

impl Default for MonoRepeatTextureOverlayMat {
    fn default() -> Self {
        Self {
            texture_overlay: Handle::default(),
            mask_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            scale: 1e-5,
        }
    }
}
impl MaterialTilemap for MonoRepeatTextureOverlayMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/textured_tile.wgsl".into()
    }
}
