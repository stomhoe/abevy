use ::sprite_shared::*;
use bevy::ecs::entity::{EntityHashMap, MapEntities};
#[allow(unused_imports, )]
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;

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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AdjRetexConfig(
    pub Vec<(Vec<(DiagonalCardinalDirection, HashId)>, (HashId, Option<TileFlip>))>,
);


impl AdjRetexConfig {
    pub fn new(seri: AdjRetexConfigSeri) -> Self {
        let mut parsed_rules = Vec::with_capacity(seri.0.len());
        for (rule_i, (adj_state_seri, (out_hash_seri, tile_flip))) in seri.0.into_iter().enumerate() {
            let mut parsed_adj_state: Vec<(DiagonalCardinalDirection, HashId)> = Vec::with_capacity(adj_state_seri.len());
            let mut invalid_rule = false;
            for (dir_seri, hash_seri) in adj_state_seri.into_iter() {
                let Some(dir) = DiagonalCardinalDirection::parse(&dir_seri) else {
                    warn!(
                        target: "tilemap",
                        "Invalid adj-retex direction '{}' in rule {}, skipping full rule",
                        dir_seri,
                        rule_i
                    );
                    invalid_rule = true;
                    break;
                };
                parsed_adj_state.push((dir, HashId::from(hash_seri)));
            }
            if invalid_rule {
                continue;
            }
            parsed_rules.push((parsed_adj_state, (HashId::from(out_hash_seri), tile_flip)));
        }
        Self(parsed_rules)
    }

    /// Uses first-match priority: rules are evaluated in order and the first rule whose requirements are all present wins.
    pub fn get_tex_in_curr_adjacency_state(&self, tile_adjacency_state: &[(DiagonalCardinalDirection, HashId)]) -> Option<(HashId, Option<TileFlip>)> {
        let state_set: HashSet<(DiagonalCardinalDirection, HashId)> = tile_adjacency_state.iter().copied().collect();
        for (reqs, (hash_id, flip)) in self.0.iter() {
            if reqs.iter().all(|req| state_set.contains(req)) {
                return Some((*hash_id, *flip));
            }
        }
        None
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

#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct KeepDistanceFrom(#[entities] pub Vec<Entity>);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct BlocksProjectiles;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Asset, TypePath)]
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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
