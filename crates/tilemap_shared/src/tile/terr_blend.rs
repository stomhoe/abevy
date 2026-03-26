use bevy::prelude::*;
use bevy::asset::AssetPath;
use bevy::{render::render_resource::AsBindGroup, shader::ShaderRef};
use bevy_ecs_tilemap::prelude::MaterialTilemap;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TerrblParamsSeri {
    #[serde(default)]
    pub texture_path: String,
    #[serde(default)]
    pub priority: f32,
    #[serde(default = "default_terrbl_scale")]
    pub scale: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub wavy_strength: f32,
    #[serde(default)]
    pub time_offset: f32,
    #[serde(default = "default_true")]
    pub blend_enabled: bool,
    #[serde(default = "default_tint")]
    pub tint: [u8; 4],
    #[serde(default = "default_tint_mask_target_sentinel")]
    pub tint_mask_target: [u8; 4],
}

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct TerrBlendParams {
    pub texture_path: AssetPath<'static>,
    #[serde(skip, default)]
    pub texture_handle: Handle<Image>,
    pub priority: f32,
    pub tint: Vec4,
    pub tint_mask_target: Vec4,
    pub has_tint: bool,
    pub has_tint_mask_target: bool,
    pub scale: f32,
    pub speed: f32,
    pub wavy_strength: f32,
    pub time_offset: f32,
    pub blend_enabled: bool,
}

impl TerrblParamsSeri {
    pub fn to_runtime(&self) -> Result<TerrBlendParams, bevy::asset::ParseAssetPathError> {
        let texture_path = AssetPath::try_parse(&self.texture_path)?
            .into_owned();

        let has_tint = self.tint != [255, 255, 255, 255];
        let has_tint_mask_target = self.tint_mask_target != [255, 0, 255, 0];
        Ok(TerrBlendParams {
            texture_path: texture_path.into(),
            texture_handle: Handle::default(),
            priority: self.priority,
            tint: if has_tint {
                Vec4::new(
                    self.tint[0] as f32 / 255.0,
                    self.tint[1] as f32 / 255.0,
                    self.tint[2] as f32 / 255.0,
                    self.tint[3] as f32 / 255.0,
                )
            } else {
                Vec4::ONE
            },
            tint_mask_target: if has_tint_mask_target {
                Vec4::new(
                    self.tint_mask_target[0] as f32 / 255.0,
                    self.tint_mask_target[1] as f32 / 255.0,
                    self.tint_mask_target[2] as f32 / 255.0,
                    self.tint_mask_target[3] as f32 / 255.0,
                )            } else {
                Vec4::ZERO
            },
            has_tint,
            has_tint_mask_target,
            scale: self.scale,
            speed: self.speed,
            wavy_strength: self.wavy_strength,
            time_offset: self.time_offset,
            blend_enabled: self.blend_enabled,
        })
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath, InspectorOptions, Deserialize, Serialize)]
pub struct TerrBlendMat {
    #[texture(1)]
    #[serde(skip)]
    pub tile_indices_map: Handle<Image>,
    #[texture(2)]
    #[serde(skip)]
    pub tile_flags_map: Handle<Image>,
    #[texture(3)]
    #[serde(skip)]
    pub tile_params_map: Handle<Image>,
    #[texture(14)]
    #[serde(skip)]
    pub tile_tint_map: Handle<Image>,
    #[uniform(4)]
    pub map_size_tiles: Vec2,
    #[uniform(5)]
    pub time: f32,
    #[texture(6)]
    #[serde(skip)]
    pub overlay_tex_0: Handle<Image>,
    #[texture(7)]
    #[serde(skip)]
    pub overlay_tex_1: Handle<Image>,
    #[texture(8)]
    #[serde(skip)]
    pub overlay_tex_2: Handle<Image>,
    #[texture(9)]
    #[serde(skip)]
    pub overlay_tex_3: Handle<Image>,
    #[texture(10)]
    #[serde(skip)]
    pub overlay_tex_4: Handle<Image>,
    #[texture(11)]
    #[serde(skip)]
    pub overlay_tex_5: Handle<Image>,
    #[texture(12)]
    #[serde(skip)]
    pub overlay_tex_6: Handle<Image>,
    #[texture(13)]
    #[serde(skip)]
    pub overlay_tex_7: Handle<Image>,
}
impl PartialEq for TerrBlendMat {
    fn eq(&self, other: &Self) -> bool {
        self.tile_indices_map == other.tile_indices_map
            && self.tile_flags_map == other.tile_flags_map
            && self.tile_params_map == other.tile_params_map
            && self.tile_tint_map == other.tile_tint_map
            && self.map_size_tiles == other.map_size_tiles
            && self.time.to_bits() == other.time.to_bits()
            && self.overlay_tex_0 == other.overlay_tex_0
            && self.overlay_tex_1 == other.overlay_tex_1
            && self.overlay_tex_2 == other.overlay_tex_2
            && self.overlay_tex_3 == other.overlay_tex_3
            && self.overlay_tex_4 == other.overlay_tex_4
            && self.overlay_tex_5 == other.overlay_tex_5
            && self.overlay_tex_6 == other.overlay_tex_6
            && self.overlay_tex_7 == other.overlay_tex_7
    }
}
impl Eq for TerrBlendMat {}

impl Default for TerrBlendMat {
    fn default() -> Self {
        Self {
            tile_indices_map: Handle::default(),
            tile_flags_map: Handle::default(),
            tile_params_map: Handle::default(),
            tile_tint_map: Handle::default(),
            map_size_tiles: Vec2::ONE,
            time: 0.0,
            overlay_tex_0: Handle::default(),
            overlay_tex_1: Handle::default(),
            overlay_tex_2: Handle::default(),
            overlay_tex_3: Handle::default(),
            overlay_tex_4: Handle::default(),
            overlay_tex_5: Handle::default(),
            overlay_tex_6: Handle::default(),
            overlay_tex_7: Handle::default(),
        }
    }
}
impl MaterialTilemap for TerrBlendMat {
    fn fragment_shader() -> ShaderRef {
        "shader_wgsl/terrbl.wgsl".into()
    }
}

fn default_terrbl_scale() -> f32 { 1e-5 }
fn default_true() -> bool { true }
fn default_tint() -> [u8; 4] { [255, 255, 255, 255] }
fn default_tint_mask_target_sentinel() -> [u8; 4] { [255, 0, 255, 0] }
