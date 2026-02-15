#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions, Deserialize, Serialize)]
#[reflect(Default, InspectorOptions)]
pub struct VoronoiTextureOverlayMat {
    #[texture(1)]
    #[sampler(2)]
    #[serde(skip)]
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
        "shader_wgsl/voronoi_shuffle.wgsl".into()
    }
}

impl Eq for VoronoiTextureOverlayMat {}
