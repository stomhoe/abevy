use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_lit::prelude::*;
use serde::{Deserialize, Serialize};

use item_shared::GeneratedItemsSeri;
use tilemap_shared::*;

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct PointLightSeri {
    pub kind: String,
    pub color: [u8; 4],
    pub intensity: f32,
    pub radius: f32,
    pub cast_shadows: bool,
    pub offset: (f32, f32),
    pub height: f32,
}

impl Default for PointLightSeri {
    fn default() -> Self {
        Self {
            kind: String::new(),
            color: [255, 255, 255, 255],
            intensity: 1.0,
            radius: 100.0,
            cast_shadows: true,
            offset: (0.0, 0.0),
            height: 0.0,
        }
    }
}

impl PointLightSeri {
    pub fn sentinel() -> Self {
        Self::default()
    }

    pub fn is_sentinel(&self) -> bool {
        self.kind.trim().is_empty() || self.kind.trim() == "unset"
    }

    pub fn to_light(&self) -> (PointLight2d, Transform) {
        (
            PointLight2d {
                color: Color::srgba_u8(self.color[0], self.color[1], self.color[2], self.color[3]),
                intensity: self.intensity,
                inner_radius: 0.0,
                outer_radius: self.radius,
                falloff: 1.0,
                cast_shadows: self.cast_shadows,
                ..Default::default()
            },
            Transform::from_xyz(self.offset.0, self.offset.1, self.height),
        )
    }
}

#[derive(Component, Deserialize, Clone, Debug, Serialize)]
#[serde(default)]
pub struct LightOccluderSeri {
    pub enabled: bool,
    pub color: [u8; 4],
    pub offset: (f32, f32),
    pub rotation: f32,
    pub use_sprite: bool,
    pub shape_size: (f32, f32),
    pub radius: Option<f32>,
    pub shape: String,
}

impl Default for LightOccluderSeri {
    fn default() -> Self {
        Self {
            enabled: false,
            color: [0, 0, 0, 255],
            offset: (0.0, 0.0),
            rotation: f32::NAN,
            use_sprite: false,
            shape_size: (32.0, 32.0),
            radius: None,
            shape: String::new(),
        }
    }
}

impl LightOccluderSeri {
    pub fn sentinel() -> Self {
        Self::default()
    }

    pub fn is_sentinel(&self) -> bool {
        !self.enabled
    }

    pub fn shape_height(&self) -> f32 {
        self.shape_size.1.max(1.0)
    }

    pub fn to_shape_mask_image(&self) -> Image {
        let width = self.shape_size.0.max(1.0).ceil() as u32;
        let height = self.shape_size.1.max(1.0).ceil() as u32;
        let mut data = vec![0_u8; width as usize * height as usize * 4];
        let cap_radius = self
            .radius
            .unwrap_or_else(|| width.min(height) as f32 * 0.5)
            .clamp(0.5, width.min(height) as f32 * 0.5);
        let shear = self.rotation.is_finite().then(|| self.rotation.to_radians().tan()).unwrap_or(0.0);

        match self.shape.trim().to_ascii_lowercase().as_str() {
            "circle" => paint_circle_mask(&mut data, width, height),
            "capsule" if shear != 0.0 => paint_sheared_capsule_mask(&mut data, width, height, cap_radius, shear),
            "capsule" => paint_capsule_mask(&mut data, width, height, cap_radius),
            _ if shear != 0.0 => paint_sheared_rectangle_mask(&mut data, width, height, shear),
            _ => paint_rectangle_mask(&mut data),
        }

        Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        )
    }
}

fn paint_rectangle_mask(data: &mut [u8]) {
    for pixel in data.chunks_exact_mut(4) {
        pixel.fill(255);
    }
}

fn paint_circle_mask(data: &mut [u8], width: u32, height: u32) {
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let radius = width.min(height) as f32 * 0.5;
    let radius_squared = radius * radius;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;
            if dx * dx + dy * dy <= radius_squared {
                paint_opaque_pixel(data, width, x, y);
            }
        }
    }
}

fn paint_sheared_rectangle_mask(data: &mut [u8], width: u32, height: u32, shear: f32) {
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let half_width = width as f32 * 0.5;
    let half_height = height as f32 * 0.5;

    for y in 0..height {
        let dy = y as f32 + 0.5 - center_y;
        let x_offset = shear * dy;
        for x in 0..width {
            let dx = x as f32 + 0.5 - center_x - x_offset;
            if dx.abs() <= half_width && dy.abs() <= half_height {
                paint_opaque_pixel(data, width, x, y);
            }
        }
    }
}

fn paint_capsule_mask(data: &mut [u8], width: u32, height: u32, radius: f32) {
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let straight_half = (height as f32 * 0.5 - radius).max(0.0);
    let radius_squared = radius * radius;

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 + 0.5 - center_x).abs();
            let dy = (y as f32 + 0.5 - center_y).abs();

            let should_paint = if dy <= straight_half {
                dx <= radius
            } else {
                let circle_dy = dy - straight_half;
                dx * dx + circle_dy * circle_dy <= radius_squared
            };

            if should_paint {
                paint_opaque_pixel(data, width, x, y);
            }
        }
    }
}

fn paint_sheared_capsule_mask(data: &mut [u8], width: u32, height: u32, radius: f32, shear: f32) {
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let straight_half = (height as f32 * 0.5 - radius).max(0.0);
    let radius_squared = radius * radius;

    for y in 0..height {
        let dy = y as f32 + 0.5 - center_y;
        let x_offset = shear * dy;
        for x in 0..width {
            let dx = x as f32 + 0.5 - center_x - x_offset;
            let adx = dx.abs();
            let ady = dy.abs();

            let should_paint = if ady <= straight_half {
                adx <= radius
            } else {
                let circle_dy = ady - straight_half;
                dx * dx + circle_dy * circle_dy <= radius_squared
            };

            if should_paint {
                paint_opaque_pixel(data, width, x, y);
            }
        }
    }
}

fn paint_opaque_pixel(data: &mut [u8], width: u32, x: u32, y: u32) {
    let index = ((y * width + x) * 4) as usize;
    data[index] = 255;
    data[index + 1] = 255;
    data[index + 2] = 255;
    data[index + 3] = 255;
}

#[derive(Deserialize, Asset, TypePath)]
#[serde(default)]
pub struct TileSeri {
    pub id: String,
    pub name: String,
    #[serde(default = "tile_z_default")]
    pub z: f32,
    pub img_paths: Vec<(String, String)>,
    pub tags: HashSet<String>,
    /// For spritetiles, non-finite values (`nan`, `inf`, `-inf`) disable Y sorting.
    pub y_sort: f32,
    /// persisted only when state gets altered from starting state
    pub persisted: bool,
    pub shader: String,
    pub terrbl_params: TerrblParamsSeri,
    pub is_spritetile: bool,
    pub color: Option<[u8; 4]>,
    pub color_map: String,
    pub spawns: Vec<String>,
    pub spawns_children: Vec<String>,
    pub randflipx: bool,
    pub randflipy: bool,
    pub randflipd: bool,
    pub size_variation: NormalDistSeri,
    pub hori_variation: NormalDistSeri,
    pub vert_variation: NormalDistSeri,
    pub min_distances: HashMap<String, u64>,
    pub portal: PortalSeri,
    pub offset: (f32, f32),

    pub interaction_zones: HashMap<String, InteractionZoneSeri>,

    pub offsets_for_portal_arrivals: Vec<(f32, (i8, i8))>,

    pub delete_other_tiles: DeleteOtherTilesSeri,
    pub terrgen_offset: (i32, i32),

    pub size_in_tiles: (u32, u32),
    /// Optional per-cell collision mask, row-major, '1' blocks movement, '0' is passable.
    pub colmask: Vec<String>,

    pub adj_retex: Option<AdjRetexConfigSeri>,
    pub walk_speed: f32,
    /// to be used by other systems to factor in their own walkspeed on top if a certain tag is present on this tile
    pub walk_speed_tags: HashSet<String>,
    pub step_sfx: TileStepSfxSeri,

    /// When true, this tile spawns a projectile-stopping collider.
    pub blocks_projectiles: bool,

    pub point_light: PointLightSeri,
    pub light_occluder: LightOccluderSeri,

    pub items_dropped_on_death: GeneratedItemsSeri,
    pub hp: f32,
}

fn tile_z_default() -> f32 {
    f32::NAN
}

impl Default for TileSeri {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            z: tile_z_default(),
            img_paths: Vec::new(),
            tags: HashSet::default(),
            y_sort: 0.0,
            persisted: false,
            shader: String::new(),
            terrbl_params: TerrblParamsSeri::default(),
            is_spritetile: false,
            color: None,
            color_map: String::new(),
            spawns: Vec::new(),
            spawns_children: Vec::new(),
            randflipx: false,
            randflipy: false,
            randflipd: false,
            size_variation: NormalDistSeri::default(),
            hori_variation: NormalDistSeri::default(),
            vert_variation: NormalDistSeri::default(),
            min_distances: HashMap::default(),
            portal: PortalSeri::default(),
            offset: (0.0, 0.0),
            interaction_zones: HashMap::default(),
            offsets_for_portal_arrivals: Vec::new(),
            delete_other_tiles: DeleteOtherTilesSeri::default(),
            terrgen_offset: (0, 0),
            size_in_tiles: (1, 1),
            colmask: Vec::new(),
            adj_retex: None,
            walk_speed: 1.0,
            walk_speed_tags: HashSet::default(),
            step_sfx: TileStepSfxSeri::default(),
            blocks_projectiles: false,
            point_light: PointLightSeri::default(),
            light_occluder: LightOccluderSeri::default(),
            items_dropped_on_death: GeneratedItemsSeri::default(),
            hp: 0.0,
        }
    }
}

impl TileSeri {
    pub fn sprite_tile_y_sort_origin(&self) -> Option<f32> {
        if self.is_spritetile && self.y_sort.is_finite() {
            Some(self.offset.1 + self.y_sort - 10.0)
        } else {
            None
        }
    }

    pub fn point_light(&self) -> Option<(PointLight2d, Transform)> {
        if self.point_light.is_sentinel() {
            return None;
        }

        Some(self.point_light.to_light())
    }
}
