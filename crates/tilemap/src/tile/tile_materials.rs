#[allow(unused_imports)] use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use common::common_components::{ ImageHolder, };
use bevy_inspector_egui::prelude::*;


#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)] 
pub struct VoronoiTextureOverlayMat {
    #[texture(1)]
    #[sampler(2)]
    pub texture_overlay: Handle<Image>,

    #[uniform(3)]
    pub mask_color: Vec4,

    #[uniform(4)]
    #[inspector(min = 1e-5, max = 1e2)]
    pub scale: f32,

    #[uniform(5)]
    #[inspector(min = 1e-5, max = 1e2)]
    pub voronoi_scale: f32,

    #[uniform(6)]
    #[inspector(min = 0.0, max = 1.0)]
    pub voronoi_scale_random: f32,

    #[uniform(7)]
    #[inspector(min = 0.0, max = 6.28319)]
    pub voronoi_rotation: f32,
}
impl VoronoiTextureOverlayMat {
    pub fn new(texture_overlay: ImageHolder, mask_color: Vec4, base_scale: f32, voronoi_scale: f32, voronoi_scale_random: f32, voronoi_rotation: f32) -> Self {
        Self { texture_overlay: texture_overlay.0, mask_color: mask_color / 255.0, scale: base_scale, voronoi_scale, voronoi_scale_random, voronoi_rotation }
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
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shader/voronoi.wgsl".into()
    }
}

impl Eq for VoronoiTextureOverlayMat {}

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
    pub fn new(texture_overlay: ImageHolder, mask_color: Vec4, scale: f32) -> Self {
        Self { texture_overlay: texture_overlay.0, mask_color: mask_color / 255.0, scale }
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
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shader/textured_tile.wgsl".into()
    }
}


#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, Component, Default)]
#[reflect(Default)] 
pub struct TwoOverlaysExample {
    #[texture(2)]
    #[sampler(3)]
    pub texture_overlay: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub texture_overlay_2: Handle<Image>,
}

impl MaterialTilemap for TwoOverlaysExample {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shader/textured_tile_dual.wgsl".into()
    }
}
impl PartialEq for TwoOverlaysExample {
    fn eq(&self, other: &Self) -> bool {
        self.texture_overlay == other.texture_overlay
            && self.texture_overlay_2 == other.texture_overlay_2
    }
}
impl Eq for TwoOverlaysExample {}


