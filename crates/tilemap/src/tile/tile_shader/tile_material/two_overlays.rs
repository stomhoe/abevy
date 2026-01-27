#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Serializer, Deserialize, Deserializer};

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


impl Serialize for TwoOverlaysExample {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // Don't serialize handles, just use default
        serializer.serialize_unit_struct("TwoOverlaysExample")
    }
}

impl<'de> Deserialize<'de> for TwoOverlaysExample {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        // Ignore input, always return default
        let _ = <()>::deserialize(deserializer)?;
        Ok(TwoOverlaysExample {
            texture_overlay: Handle::default(),
            texture_overlay_2: Handle::default(),
        })
    }
}
