#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Deserialize, };

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, Component, Default, Deserialize, Serialize)]
#[reflect(Default)]
pub struct TwoOverlaysExample {
    #[texture(2)]
    #[sampler(3)]
    #[serde(skip)]
    pub texture_overlay: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    #[serde(skip)]
    pub texture_overlay_2: Handle<Image>,
}

impl MaterialTilemap for TwoOverlaysExample {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/textured_tile_dual.wgsl".into()
    }
}
impl PartialEq for TwoOverlaysExample {
    fn eq(&self, other: &Self) -> bool {
        self.texture_overlay == other.texture_overlay
            && self.texture_overlay_2 == other.texture_overlay_2
    }
}
impl Eq for TwoOverlaysExample {}
