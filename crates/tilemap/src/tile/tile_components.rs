use ::sprite_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::entity::MapEntities;
#[allow(unused_imports, )]
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::{HashedTagsVec, TagSet};

use game_common::game_common_components::*;

use ::tilemap_shared::*;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::terrain_gen::{terrgen_messages::OpFilter};
use crate::tile::tile_resources::{AdjRetexConfigSeri, PortalSeri, TileImagePaths};



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

#[derive(Component, Debug, Default, Clone)]
pub struct DebugValue{
    pub name: String,
    pub value: f32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AdjRetexConfig(
    pub Vec<(Vec<(DiagonalCardinalDirection, HashId)>, HashId)>,
);


impl AdjRetexConfig {
    pub fn new(seri: AdjRetexConfigSeri) -> Self {
        let mut parsed_rules: Vec<(Vec<(DiagonalCardinalDirection, HashId)>, HashId)> = Vec::with_capacity(seri.0.len());
        for (rule_i, (adj_state_seri, out_hash_seri)) in seri.0.into_iter().enumerate() {
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
            parsed_rules.push((parsed_adj_state, HashId::from(out_hash_seri)));
        }
        Self(parsed_rules)
    }

    /// Uses first-match priority: rules are evaluated in order and the first rule whose requirements are all present wins.
    pub fn get_tex_in_curr_adjacency_state(&self, tile_adjacency_state: &[(DiagonalCardinalDirection, HashId)]) -> Option<HashId> {
        let state_set: HashSet<(DiagonalCardinalDirection, HashId)> = tile_adjacency_state.iter().copied().collect();
        for (reqs, hash_id) in self.0.iter() {
            if reqs.iter().all(|req| state_set.contains(req)) {
                return Some(*hash_id);
            }
        }
        None
    }
}

//TODO HACER Q LAS TILES CAMBIEN AUTOMATICAMENTE DE TINTE SEGUN VALOR DE NOISES RELEVANTES COMO HUMEDAD O LO Q SEA
//SE PUEDE MODIFICAR EL SHADER PARA Q TOME OTRO VEC3 DE COLOR MÁS COMO PARÁMETRO Y SE LE MULTIPLIQUE AL PIXEL DE LA TEXTURA SAMPLEADO

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct SeekingPortalOtherEnd;

#[derive(Component, Debug, Clone, )]
pub struct PortalRecipe {
    #[entities]
    pub dest_dimension: Entity,
    #[entities]
    pub oe_portal_tile: Entity,
    pub tags: TagSet,
    pub op_i: i16,
    pub min_val: f32,
    pub max_val: f32,
    pub one_way: bool,
}
impl PortalRecipe {
    pub fn to_op_filter(&self, start_pos: GlobalTilePos, oe_rootoplist: Entity) -> OpFilter {
        OpFilter {
            tags: HashedTagsVec::from(&self.tags),
            start_oplist: oe_rootoplist,
            op_i: self.op_i,
            min_val: self.min_val,
            max_val: self.max_val,
            search_start_pos: start_pos,
        }
    }
}
impl Default for PortalRecipe {
    fn default() -> Self {
        Self {
            dest_dimension: Entity::PLACEHOLDER,
            oe_portal_tile: Entity::PLACEHOLDER,
            tags: TagSet::default(),
            op_i: -1,
            min_val: 0.0,
            max_val: 0.0,
            one_way: false,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, )]
pub struct PortalTo {

    pub dest_portal: Entity,
}
impl PortalTo {
    pub fn new(dest_portal: Entity) -> Self {
        Self { dest_portal }
    }
}

pub fn tile_pos_hash_rand(initial_pos: InitialPos, settings: &GlobalGenSettings) -> f32 {
    let mut hasher = DefaultHasher::new();
    initial_pos.hash(&mut hasher);
    settings.seed.hash(&mut hasher);
    (hasher.finish() as f64 / u64::MAX as f64).abs() as f32
}

#[derive(Component, Deserialize, Serialize, Default, Debug,)]
pub struct FlipHorizontallyBasedOnHash;

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


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct BlocksProjectiles;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct DeleteOtherTiles {
    pub spared_z: HashSet<AcZ>,
    pub spared_tags: TagSet,
    pub extra_radius: u32,
    /// use this only if both delete each other and they don't spare each other. the one with higher priority doesn't get deleted
    pub priority: u32,
}


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
#[require(Replicated, AssetScoped, Prefix::trunc("EguiPortalsZeroHolder"), Transform, Visibility)]
pub struct PortalsZeroEguiHolder;
