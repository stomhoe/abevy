use ::sprite_shared::*;
use bevy::ecs::entity::{EntityHashMap, MapEntities};
#[allow(unused_imports, )]
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use common::log_targets::TILE_INIT;

use game_common::{define_weightedsampler, game_common_components::*, game_common_samplers::GlobalTilePosWeightedSampler};

use ::tilemap_shared::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::tile::tile_resources::*;



#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct KeepDisabled;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
//Don't add RequiredComponents here because it is forced onto clones and when removed it despawns the new entity
pub struct Tile;
impl Tile {
    pub const MIN_ID_LENGTH: u8 = 1;
}
pub type TileStrId = StrId;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct TileChildSprite;

define_weightedsampler!(TileStepSfx, Vec<String>, "TileStepSfx");

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct TileStepSfxConfig {
    pub prevent_repeat: bool,
}
impl Default for TileStepSfxConfig {
    fn default() -> Self {
        Self {
            prevent_repeat: true,
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct DebugValue{
    pub name: String,
    pub value: f32,
}

pub type VisibleResult = (HashId, Option<TileFlip>);
pub type ModuloResult = Option<u32>;

/// Bit positions for 8-direction adjacency masks.
/// These are used by adjacency-based retexturing (autotiling).
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdjMask(pub u16);
impl AdjMask {
    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn insert(&mut self, bit: AdjMask) {
        self.0 |= bit.0;
    }
    pub fn contains_all(&self, other: AdjMask) -> bool {
        (self.0 & other.0) == other.0
    }
    pub fn count_bits(&self) -> usize {
        self.0.count_ones() as usize
    }
}

pub trait DiagonalCardinalDirectionAdjMaskExt {
    fn adj_mask_bit(self) -> AdjMask;
}
impl DiagonalCardinalDirectionAdjMaskExt for DiagonalCardinalDirection {
    #[inline]
    fn adj_mask_bit(self) -> AdjMask {
        AdjMask(match self {
            DiagonalCardinalDirection::North => 1 << 0,
            DiagonalCardinalDirection::East => 1 << 1,
            DiagonalCardinalDirection::South => 1 << 2,
            DiagonalCardinalDirection::West => 1 << 3,
            DiagonalCardinalDirection::NorthEast => 1 << 4,
            DiagonalCardinalDirection::SouthEast => 1 << 5,
            DiagonalCardinalDirection::SouthWest => 1 << 6,
            DiagonalCardinalDirection::NorthWest => 1 << 7,
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AdjRetexRule {
    /// HashId of the neighbor tile type this rule is matching against.
    pub connect_to: HashId,
    /// Required adjacency mask (8-direction bitmask).
    pub required_mask: AdjMask,
    pub out: VisibleResult,
    pub match_mode: AdjRetexRuleMatchMode,
    pub mod_res_i: ModuloResult,
    pub mod_res_j: ModuloResult,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AdjRetexConfig(
    pub Vec<AdjRetexRule>,
);

impl AdjRetexConfig {
    pub fn new(seri: AdjRetexConfigSeri) -> Self {
        let mut parsed_rules = Vec::with_capacity(seri.0.len());
        for (rule_i, rule_seri) in seri.0.into_iter().enumerate() {
            let adj_state_seri = rule_seri.adj_state;
            let mod_res_i = (rule_seri.modulo_i != u32::MAX).then_some(rule_seri.modulo_i);
            let mod_res_j = (rule_seri.modulo_j != u32::MAX).then_some(rule_seri.modulo_j);
            let out_hash_seri = rule_seri.out_id;
            let tile_flip = rule_seri.tile_flip;
            let match_mode = rule_seri.match_mode;
            let mut required_mask = AdjMask::empty();
            let mut connect_to_str: Option<String> = None;
            let mut invalid_rule = false;
            for (id_seri, dir_seri, ) in adj_state_seri.into_iter() {
                let id_trim = id_seri.trim();
                if id_trim.is_empty() {
                    warn!(
                        target: TILE_INIT,
                        "Invalid adj-retex neighbor id '{}' in rule {}, skipping full rule",
                        id_seri,
                        rule_i
                    );
                    invalid_rule = true;
                    break;
                }
                if let Some(existing) = &connect_to_str {
                    if existing != id_trim {
                        warn!(
                            target: TILE_INIT,
                            "Adj-retex rule {} mixes multiple neighbor ids ('{}', '{}'), skipping full rule",
                            rule_i,
                            existing,
                            id_trim
                        );
                        invalid_rule = true;
                        break;
                    }
                } else {
                    connect_to_str = Some(id_trim.to_string());
                }
                let Some(dir) = DiagonalCardinalDirection::parse(&dir_seri) else {
                    warn!(
                        target: TILE_INIT,
                        "Invalid adj-retex direction '{}' in rule {}, skipping full rule",
                        dir_seri,
                        rule_i
                    );
                    invalid_rule = true;
                    break;
                };
                required_mask.insert(dir.adj_mask_bit());
            }
            if invalid_rule {
                continue;
            }
            let Some(connect_to_str) = connect_to_str else {
                continue;
            };
            parsed_rules.push(AdjRetexRule {
                connect_to: HashId::from(connect_to_str),
                required_mask,
                out: (HashId::from(out_hash_seri), tile_flip),
                match_mode,
                mod_res_i,
                mod_res_j,
            });
        }
        Self(parsed_rules)
    }

    /// Mixed-mode rules:
    /// - `ExactState`: requires exact equality between current adjacency set and rule requirements.
    /// - `BestMatching`: among matching subset-rules, picks the one with highest requirement count.
    pub fn get_tex_in_curr_adjacency_state(&self, adj_masks_by_hid: &HashMap<HashId, AdjMask>) -> Option<VisibleResult> {
        let mut best_match: Option<(usize, VisibleResult)> = None;
        for rule in self.0.iter() {
            let current_mask = adj_masks_by_hid.get(&rule.connect_to).copied().unwrap_or_default();
            match rule.match_mode {
                AdjRetexRuleMatchMode::ExactState => {
                    if current_mask == rule.required_mask {
                        return Some(rule.out);
                    }
                }
                AdjRetexRuleMatchMode::BestMatching => {
                    if current_mask.contains_all(rule.required_mask) {
                        let reqs_len = rule.required_mask.count_bits();
                        let should_replace = match best_match {
                            Some((best_len, ..)) => reqs_len > best_len,
                            None => true,
                        };
                        if should_replace {
                            best_match = Some((reqs_len, rule.out));
                        }
                    }
                }
            }
        }
        best_match.map(|(_, visible_result)| visible_result)
    }
}

//TODO HACER Q LAS TILES CAMBIEN AUTOMATICAMENTE DE TINTE SEGUN VALOR DE NOISES RELEVANTES COMO HUMEDAD O LO Q SEA
//SE PUEDE MODIFICAR EL SHADER PARA Q TOME OTRO VEC3 DE COLOR MÁS COMO PARÁMETRO Y SE LE MULTIPLIQUE AL PIXEL DE LA TEXTURA SAMPLEADO



#[derive(Component, Debug, Clone, Deserialize, Serialize, MapEntities)]
pub struct PortalRecipe {
    #[entities]
    pub dest_dimension: Entity,
    #[entities]
    pub oe_portal_tile: Entity,
    #[entities]
    pub terrprobe_ent: Entity,
    pub one_way: bool,
    pub sampler: GlobalTilePosWeightedSampler,
}

#[derive(Component, Debug, Clone, )]
pub struct PortalTo {
    pub dest_portal: Entity,
    pub offset_pos_destinations: GlobalTilePosWeightedSampler
}
impl PortalTo {
    pub fn new(dest_portal: Entity, offset_pos_destinations: GlobalTilePosWeightedSampler) -> Self {
        Self { dest_portal, offset_pos_destinations }
    }
}

pub fn tile_pos_hash_rand(initial_pos: InitialPos, settings: &GlobalGenSettings) -> f32 {
    let mut hasher = DefaultHasher::new();
    initial_pos.hash(&mut hasher);
    settings.seed.hash(&mut hasher);
    (hasher.finish() as f64 / u64::MAX as f64).abs() as f32
}

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlipHorizontallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlipVerticallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlipDiagonallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct RotateCardinallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct TransformBasedCardRotation;

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Debug,)]
pub struct InitialPos(pub GlobalTilePos);





#[derive(Component, Debug, Clone, Default)]
/// Holds the mapping between tile image HashIds and the image handles they are mapped to
pub struct TileHashIdsHandles {
    ids: Vec<HashId>,
    handles: Vec<Handle<Image>>,
}
impl TileHashIdsHandles {
    pub fn from_paths(
        asset_server: &AssetServer,
        img_paths: TileImagePaths,
    ) -> Result<Self, BevyError> {
        if img_paths.is_empty() {
            return Err(BevyError::from(
                "TileImgsMap cannot be created with an empty image paths map",
            ));
        }
        let mut ids = Vec::with_capacity(img_paths.len());
        let mut handles = Vec::with_capacity(img_paths.len());
        for (key, path) in img_paths {
            let Ok(image_holder) = ImageHolder::new(asset_server, path.clone()) else {
                error!(
                    "Failed to find image file for key {} at path: {}",
                    key, path
                );
                continue;
            };
            ids.push(HashId::from(key));
            handles.push(image_holder.0);
        }
        if ids.is_empty() {
            return Err(BevyError::from("No valid entries"));
        }
        Ok(Self { ids, handles })
    }
    pub fn len(&self) -> usize {
        self.handles.len()
    }
    pub fn first_handle(&self) -> Handle<Image> {
        self.handles
            .first()
            .cloned()
            .unwrap_or_else(|| Handle::default())
    }
    /// NO HACER take() porque lo necesitan multiples isntancias de tiles
    pub fn handles(&self) -> &Vec<Handle<Image>> {
        &self.handles
    }
    pub fn iter(&self) -> impl Iterator<Item = (HashId, &Handle<Image>)> {
        self.ids.iter().cloned().zip(self.handles.iter())
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
/// applied on tile's start gpos if placed via terrgen
pub struct OffsetForTerrgenPlacement(pub GlobalTilePos);

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct MinDistancesMap(pub EntityHashMap<u64>);

impl MinDistancesMap {
    #[allow(unused_parens)]
    pub fn check_min_distances(
        &self,
        my_pos: (DimensionRef, GlobalTilePos),
        new: (EntityZeroRef, DimensionRef, GlobalTilePos),
    ) -> bool {
        self.0.get(&new.0.0).map_or(true, |&min_dist| {
            my_pos.0 != new.1 || my_pos.1.distance_squared(&new.2) > (min_dist * min_dist)
        })
    }
}

#[derive(Component, Debug, Default, Clone, )]
pub struct KeepDistanceFrom(#[entities] pub Vec<Entity>);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct BlocksProjectiles;

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct TerrBlendParams {
    pub texture_path: Option<ImagePathHolder>,
    #[serde(skip, default)]
    pub texture_handle: Handle<Image>,
    pub mask_color: Vec4,
    pub scale: f32,
    pub speed: f32,
    pub wavy_strength: f32,
    pub time_offset: f32,
    pub blend_enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TerrblParamsSeri {
    #[serde(default)]
    pub texture_path: String,
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
}
impl Default for TerrblParamsSeri {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            scale: default_terrbl_scale(),
            speed: 0.0,
            wavy_strength: 0.0,
            time_offset: 0.0,
            blend_enabled: default_true(),
        }
    }
}
impl TerrblParamsSeri {
    pub fn to_terrbl_params(&self) -> TerrBlendParams {
        let texture_path = if self.texture_path.trim().is_empty() {
            None
        } else {
            let Ok(path_holder) = ImagePathHolder::new(self.texture_path.clone()) else {
                error!(
                    target: TILE_INIT,
                    "Invalid terrbl texture path '{}', falling back to no overlay",
                    self.texture_path
                );
                return TerrBlendParams {
                    texture_path: None,
                    texture_handle: Handle::default(),
                    mask_color: Vec4::new(255.0, 0.0, 0.0, 255.0),
                    scale: self.scale,
                    speed: self.speed,
                    wavy_strength: self.wavy_strength,
                    time_offset: self.time_offset,
                    blend_enabled: self.blend_enabled,
                };
            };
            Some(path_holder)
        };
        TerrBlendParams {
            texture_path,
            texture_handle: Handle::default(),
            mask_color: Vec4::new(255.0, 0.0, 0.0, 255.0),
            scale: self.scale,
            speed: self.speed,
            wavy_strength: self.wavy_strength,
            time_offset: self.time_offset,
            blend_enabled: self.blend_enabled,
        }
    }
}
fn default_terrbl_scale() -> f32 {
    1e-5
}
fn default_true() -> bool {
    true
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct DeleteOtherTiles {
    pub spared_z: HashSet<AcZ>,
    pub targeted_z: HashSet<AcZ>,
    pub spared_tags: TagSet,
    pub targeted_tags: TagSet,
    pub extra_radius: u32,
    /// use this only if both delete each other and they don't spare each other. the one with higher priority doesn't get deleted
    pub priority: f32,
}

impl DeleteOtherTiles {
    pub fn is_empty(&self) -> bool {
        self.spared_z.is_empty()
        && self.targeted_z.is_empty()
        && self.spared_tags.is_empty()
        && self.targeted_tags.is_empty()
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DeleteOtherTilesSeri {
    #[serde(default)]
    pub spared_z: Vec<f32>,
    #[serde(default)]
    pub targeted_z: Vec<f32>,
    #[serde(default)]
    pub spared_tags: Vec<String>,
    #[serde(default)]
    pub targeted_tags: Vec<String>,
    #[serde(default)]
    pub extra_radius: u32,
    #[serde(default)]
    pub priority: f32,
}
impl DeleteOtherTilesSeri {
    pub fn to_delete_other_tiles(&self) -> DeleteOtherTiles {
        let mut spared_z = HashSet::default();
        for &z in &self.spared_z {
            spared_z.insert(AcZ::new(z));
        }
        let mut targeted_z = HashSet::default();
        for &z in &self.targeted_z {
            targeted_z.insert(AcZ::new(z));
        }
        let mut spared_tags = TagSet::default();
        for tag in &self.spared_tags {
            spared_tags.insert(Tag::trunc(tag));
        }
        let mut targeted_tags = TagSet::default();
        for tag in &self.targeted_tags {
            targeted_tags.insert(Tag::trunc(tag));
        }
        DeleteOtherTiles {
            spared_z,
            targeted_z,
            spared_tags,
            targeted_tags,
            extra_radius: self.extra_radius,
            priority: self.priority,
        }
    }
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(Replicated, AssetScoped, Prefix::trunc("EguiPortalsZeroHolder"), Transform, Visibility)]
pub struct PortalsZeroEguiHolder;
