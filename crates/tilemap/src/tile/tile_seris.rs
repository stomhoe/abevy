use bevy::prelude::*;
use serde::Deserialize;

use item_shared::GeneratedItemsSeri;
use tilemap_shared::*;

fn default_f32_one() -> f32 { 1. }
fn default_u32_u32_one() -> (u32, u32) { (1, 1) }

#[derive(Deserialize, Asset, TypePath, Default, )]
pub struct TileSeri {
    pub id: String,
    pub name: String,
    pub z: f32,
    pub img_paths: Vec<(String, String)>,
    #[serde(default)]
    pub tags: HashSet<String>,
    pub y_sort: Option<f32>,
    /// persisted only when state gets altered from starting state
    #[serde(default)]
    pub persisted: bool,
    #[serde(default)]
    pub shader: String,
    #[serde(default)]
    pub terrbl_params: TerrblParamsSeri,
    #[serde(default)]
    pub is_spritetile: bool,
    pub color: Option<[u8; 4]>,
    #[serde(default)]
    pub color_map: String,
    #[serde(default)]
    pub spawns: Vec<String>,
    #[serde(default)]
    pub spawns_children: Vec<String>,
    #[serde(default)]
    pub randflipx: bool,
    #[serde(default)]
    pub randflipy: bool,
    #[serde(default)]
    pub randflipd: bool,
    #[serde(default)]
    pub min_distances: HashMap<String, u64>,
    #[serde(default)]
    pub portal: PortalSeri,
    #[serde(default)]
    pub offset: (f32, f32),

    #[serde(default)]
    pub interaction_zones: HashMap<String, InteractionZoneSeri>,

    #[serde(default)]
    pub offsets_for_portal_arrivals: Vec<(f32, (i8, i8))>,

    #[serde(default)]
    pub delete_other_tiles: DeleteOtherTilesSeri,
    #[serde(default)]
    pub terrgen_offset: (i32, i32),

    #[serde(default = "default_u32_u32_one")]
    pub size_in_tiles: (u32, u32),
    /// Optional per-cell collision mask, row-major, '1' blocks movement, '0' is passable.
    #[serde(default)]
    pub colmask: Vec<String>,

    pub adj_retex: Option<AdjRetexConfigSeri>,
    #[serde(default = "default_f32_one")]
    pub walk_speed: f32,
    /// to be used by other systems to factor in their own walkspeed on top if a certain tag is present on this tile
    #[serde(default)]
    pub walk_speed_tags: HashSet<String>,
    #[serde(default)]
    pub step_sfx: TileStepSfxSeri,

    /// When true, this tile spawns a projectile-stopping collider.
    #[serde(default)]
    pub blocks_projectiles: bool,

    #[serde(default)]
    pub items_dropped_on_death: GeneratedItemsSeri,
    #[serde(default)]
    pub hp: f32,
}
