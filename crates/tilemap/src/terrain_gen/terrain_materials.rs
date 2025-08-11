#[allow(unused_imports)] use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy_ecs_tilemap::prelude::MaterialTilemap;

#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct MonoRepeatTextureOverlayMat {
    #[texture(1)]
    #[sampler(2)]
    pub texture_overlay: Handle<Image>,
    #[uniform(3)]
    pub mask_color: Vec4,
    #[uniform(4)]
    pub scale: f32,

}
impl Default for MonoRepeatTextureOverlayMat {
    fn default() -> Self {
        Self { 
            texture_overlay: Handle::default(),
            mask_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            scale: 0.00001,
        }
    }
}

impl MaterialTilemap for MonoRepeatTextureOverlayMat {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/textured_tile.wgsl".into()
    }
}


#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
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
        "shaders/textured_tile_dual.wgsl".into()
    }
}


#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct MyMaterial {
    #[uniform(0)]
    brightness: f32,
    #[uniform(0)]
    _padding: Vec3,
}

impl MaterialTilemap for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_shader.wgsl".into()
    }
}