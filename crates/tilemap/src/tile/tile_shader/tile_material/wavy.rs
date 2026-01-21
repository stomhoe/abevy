#[allow(unused_imports)] use bevy::prelude::*;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::ser::SerializeStruct;
use bevy_inspector_egui::prelude::*;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)] 
pub struct WavyMat {
    #[uniform(1)]#[inspector(min = 1e-5, max = 1e2)]
    pub scale: f32,
    #[uniform(2)]
    pub time: f32,
    #[uniform(3)]//#[inspector(min = 0.0, max = 10.0)]
    pub speed: Vec2,
    #[uniform(4)]#[inspector(min = 0.0, max = 1.0)]
    pub amplitude: f32,
    #[uniform(5)]
    pub wave_color: Vec4,
    #[uniform(6)]#[inspector(min = 0.01, max = 10.0)]
    pub cell_scale: f32,
    #[uniform(7)]#[inspector(min = 0.0, max = 1.0)]
    pub seam_strength: f32,
    #[uniform(8)]#[inspector(min = 0.0, max = 2.0)]
    pub highlight_strength: f32,
    #[uniform(9)]#[inspector(min = 0.0, max = 5.0)]
    pub warp_strength: f32,
    #[uniform(10)]#[inspector(min = 0.0, max = 5.0)]
    pub flow_speed: f32,
} 

impl WavyMat {
    pub fn new(scale: f32, time: f32, speed: Vec2, amplitude: f32, wave_color: Vec4, cell_scale: f32, seam_strength: f32, highlight_strength: f32, warp_strength: f32, flow_speed: f32) -> Self {
        Self { scale, time, speed, amplitude, wave_color: wave_color / 255.0, cell_scale, seam_strength, highlight_strength, warp_strength, flow_speed }
    }
}

impl Serialize for WavyMat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut state = serializer.serialize_struct("WavyMat", 10)?;
        state.serialize_field("scale", &self.scale)?;
        state.serialize_field("time", &self.time)?;
        state.serialize_field("speed", &self.speed)?;
        state.serialize_field("amplitude", &self.amplitude)?;
        state.serialize_field("wave_color", &self.wave_color)?;
        state.serialize_field("cell_scale", &self.cell_scale)?;
        state.serialize_field("seam_strength", &self.seam_strength)?;
        state.serialize_field("highlight_strength", &self.highlight_strength)?;
        state.serialize_field("warp_strength", &self.warp_strength)?;
        state.serialize_field("flow_speed", &self.flow_speed)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for WavyMat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct WavyMatData {
            scale: f32,
            time: f32,
            speed: Vec2,
            amplitude: f32,
            wave_color: Vec4,
            cell_scale: f32,
            seam_strength: f32,
            highlight_strength: f32,
            warp_strength: f32,
            flow_speed: f32,
        }
        let data = WavyMatData::deserialize(deserializer)?;
        Ok(WavyMat {
            scale: data.scale,
            time: data.time,
            speed: data.speed,
            amplitude: data.amplitude,
            wave_color: data.wave_color,
            cell_scale: data.cell_scale,
            seam_strength: data.seam_strength,
            highlight_strength: data.highlight_strength,
            warp_strength: data.warp_strength,
            flow_speed: data.flow_speed,
        })
    }
}

impl PartialEq for WavyMat {
    fn eq(&self, other: &Self) -> bool {
        self.scale.to_bits() == other.scale.to_bits()
            && self.time.to_bits() == other.time.to_bits()
            && self.speed == other.speed
            && self.amplitude.to_bits() == other.amplitude.to_bits()
            && self.wave_color == other.wave_color
            && self.cell_scale.to_bits() == other.cell_scale.to_bits()
            && self.seam_strength.to_bits() == other.seam_strength.to_bits()
            && self.highlight_strength.to_bits() == other.highlight_strength.to_bits()
            && self.warp_strength.to_bits() == other.warp_strength.to_bits()
            && self.flow_speed.to_bits() == other.flow_speed.to_bits()
    }
}
impl Eq for WavyMat {}

impl Default for WavyMat {
    fn default() -> Self {
        Self { 
            scale: 1e-5,
            time: 0.0,
            speed: Vec2::ONE,
            amplitude: 0.5,
            wave_color: Vec4::new(0.0, 0.5, 1.0, 0.6),
            cell_scale: 6.0,
            seam_strength: 0.6,
            highlight_strength: 0.6,
            warp_strength: 0.9,
            flow_speed: 1.0,
        }
    }
}

impl MaterialTilemap for WavyMat {
    fn fragment_shader() -> ShaderRef {
        "shader/wavy.wgsl".into()
    }
}
