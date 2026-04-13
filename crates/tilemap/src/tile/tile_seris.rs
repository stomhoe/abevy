use bevy::prelude::*;
use serde::Deserialize;

use item_shared::GeneratedItemsSeri;
use tilemap_shared::*;

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
}
