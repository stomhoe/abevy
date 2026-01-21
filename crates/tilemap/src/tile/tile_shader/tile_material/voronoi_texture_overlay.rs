#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::ser::SerializeStruct;
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)] 
pub struct VoronoiTextureOverlayMat {
    #[texture(1)]
    #[sampler(2)]
    pub texture_overlay: Handle<Image>,

    #[uniform(3)]
    pub mask_color: Vec4,

    #[uniform(4)]#[inspector(min = 1e-5, max = 1e2)]
    pub scale: f32,

    #[uniform(5)]#[inspector(min = 1e-5, max = 1e2)]
    pub voronoi_scale: f32,

    #[uniform(6)]#[inspector(min = 0.0, max = 1.0)]
    pub voronoi_scale_random: f32,

    #[uniform(7)]#[inspector(min = 0.0, max = 6.28319)]
    pub voronoi_rotation: f32,
}
impl Serialize for VoronoiTextureOverlayMat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut state = serializer.serialize_struct("VoronoiTextureOverlayMat", 5)?;
        state.serialize_field("mask_color", &self.mask_color)?;
        state.serialize_field("scale", &self.scale)?;
        state.serialize_field("voronoi_scale", &self.voronoi_scale)?;
        state.serialize_field("voronoi_scale_random", &self.voronoi_scale_random)?;
        state.serialize_field("voronoi_rotation", &self.voronoi_rotation)?;
        state.end()
    }
}
impl<'de> Deserialize<'de> for VoronoiTextureOverlayMat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct VoronoiTextureOverlayMatData {
            mask_color: Vec4,
            scale: f32,
            voronoi_scale: f32,
            voronoi_scale_random: f32,
            voronoi_rotation: f32,
        }
        let data = VoronoiTextureOverlayMatData::deserialize(deserializer)?;
        Ok(VoronoiTextureOverlayMat {
            texture_overlay: Handle::default(),
            mask_color: data.mask_color,
            scale: data.scale,
            voronoi_scale: data.voronoi_scale,
            voronoi_scale_random: data.voronoi_scale_random,
            voronoi_rotation: data.voronoi_rotation,
        })
    }
}
impl VoronoiTextureOverlayMat {
    pub fn new(texture_overlay: Handle<Image>, mask_color: Vec4, base_scale: f32, voronoi_scale: f32, voronoi_scale_random: f32, voronoi_rotation: f32) -> Self {
        Self { texture_overlay, mask_color: mask_color / 255.0, scale: base_scale, voronoi_scale, voronoi_scale_random, voronoi_rotation }
    }
}

impl Default for VoronoiTextureOverlayMat {
    fn default() -> Self {
        Self {
            texture_overlay: Handle::default(),
            mask_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            scale: 1e1,
            voronoi_scale: 2.,
            voronoi_scale_random: 1e-2,
            voronoi_rotation: 3.14*2.0,
        }
    }
}
impl PartialEq for VoronoiTextureOverlayMat {
    fn eq(&self, other: &Self) -> bool {
        self.texture_overlay == other.texture_overlay
            && self.mask_color == other.mask_color
            && self.scale.to_bits() == other.scale.to_bits()
            && self.voronoi_scale.to_bits() == other.voronoi_scale.to_bits()
    }
}
impl MaterialTilemap for VoronoiTextureOverlayMat {
    fn fragment_shader() -> ShaderRef {
        "shader/voronoi_shuffle.wgsl".into()
    }
}

impl Eq for VoronoiTextureOverlayMat {}
