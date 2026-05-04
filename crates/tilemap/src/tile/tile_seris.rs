use bevy::prelude::*;
use bevy_firefly::prelude::*;
use serde::Deserialize;

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

    pub fn to_light(&self) -> (PointLight2d, LightHeight) {
        (
            PointLight2d {
                color: Color::srgba_u8(self.color[0], self.color[1], self.color[2], self.color[3]),
                intensity: self.intensity,
                radius: self.radius,
                cast_shadows: self.cast_shadows,
                offset: Vec3::new(self.offset.0, self.offset.1, 0.0),
                ..Default::default()
            },
            LightHeight(self.height),
        )
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LightOccluderSeri {
    pub opacity: f32,
    pub color: [u8; 4],
    pub z_sorting: bool,
    pub offset: (f32, f32),
    pub shape_size: (f32, f32),
    pub shape: String,
}

impl Default for LightOccluderSeri {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            color: [0, 0, 0, 255],
            z_sorting: true,
            offset: (0.0, 0.0),
            shape_size: (32.0, 32.0),
            shape: String::new(),
        }
    }
}

impl LightOccluderSeri {
    pub fn sentinel() -> Self {
        Self::default()
    }

    pub fn is_sentinel(&self) -> bool {
        self.shape.trim().is_empty() || self.shape.trim() == "unset"
    }

    pub fn to_occluder(&self) -> Occluder2d {
        let (width, height) = self.shape_size;
        let occluder = match self.shape.trim().to_ascii_lowercase().as_str() {
            "rectangle" | "" | "unset" => Occluder2d::rectangle(width, height),
            "circle" => {
                let radius = width.min(height) * 0.5;
                Occluder2d::circle(radius)
            }
            "capsule" => {
                let length = height;
                let radius = (width * 0.5).min(height * 0.25);
                Occluder2d::vertical_capsule(length, radius)
            }
            other => {
                warn!("Unknown light occluder shape '{}' on tile '{}', falling back to rectangle", other, self.shape);
                Occluder2d::rectangle(width, height)
            }
        };

        occluder
            .with_opacity(self.opacity)
            .with_color(Color::srgba_u8(self.color[0], self.color[1], self.color[2], self.color[3]))
            .with_z_sorting(self.z_sorting)
            .with_offset(Vec3::new(self.offset.0, self.offset.1, 0.0))
    }
}

#[derive(Deserialize, Asset, TypePath)]
#[serde(default)]
pub struct TileSeri {
    pub id: String,
    pub name: String,
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

impl Default for TileSeri {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            z: 0.0,
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

    pub fn light_occluder(&self) -> Option<Occluder2d> {
        if self.light_occluder.is_sentinel() {
            return None;
        }

        Some(self.light_occluder.to_occluder())
    }

    pub fn point_light(&self) -> Option<(PointLight2d, LightHeight)> {
        if self.point_light.is_sentinel() {
            return None;
        }

        Some(self.point_light.to_light())
    }
}
