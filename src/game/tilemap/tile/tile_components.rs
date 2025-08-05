use bevy::math::U8Vec4;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::common::common_components::{EntityPrefix, MyZ};
use crate::game::game_components::ImagePathHolder;
use crate::game::tilemap::terrain_gen::terrgen_resources::WorldGenSettings;
use crate::game::tilemap::{chunking_components::ChunkPos, tile::tile_utils::* };
use bevy_ecs_tilemap::tiles::*;

use bevy_ecs_tilemap::tiles::*;
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;
use serde::{Serialize, Serializer, Deserialize, Deserializer};



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(MyZ, EntityPrefix::new("Tile"), )]
pub struct Tile;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TileposHashRand(pub f32);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(TileposHashRand)]
pub struct FlipAlongX;


#[derive(Component, Debug, Default, )]
pub struct Tree;

#[derive(Component, Debug,  Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TileShaderRef(pub Entity);

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, )]
#[require(EntityPrefix::new("TileShader"), )]
pub enum TileShader{
    TexRepeat(RepeatingTexture),
    TwoTexRepeat(RepeatingTexture, RepeatingTexture),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, )]
pub struct RepeatingTexture{ img: Handle<Image>, scale: u32, mask_color: U8Vec4, }
impl RepeatingTexture {
    #[allow(dead_code, )]
    pub fn new(asset_server: &AssetServer, path: ImagePathHolder, scale: u32, mask_color: U8Vec4) -> Self {
        Self { img: asset_server.load(path), scale, mask_color }
    }
    pub fn new_w_red_mask<S: Into<String>>(asset_server: &AssetServer, path: S, scale: u32) -> Self {
        Self { img: asset_server.load(path.into()), scale, mask_color: U8Vec4::new(255, 0, 0, 255) }
    }
    pub fn cloned_handle(&self) -> Handle<Image> { self.img.clone() }
    #[allow(non_snake_case)]
    pub fn scale_div_1kM(&self) -> f32 { self.scale as f32 / 1_000_000_000.0 }
    pub fn mask_color(&self) -> Vec4 { self.mask_color.as_vec4()/255.0 }
}

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy)]
pub struct GlobalTilePos(pub IVec2);

impl GlobalTilePos {
    pub fn get_pos_within_chunk(self, chunk_pos: ChunkPos) -> TilePos {
        let pos_within_chunk = self - chunk_pos.to_tilepos();
        TilePos::new(pos_within_chunk.x() as u32, pos_within_chunk.y() as u32)
    }
    pub fn x(&self) -> i32 { self.0.x } 
    pub fn y(&self) -> i32 { self.0.y }

    pub fn hash_value(&self, settings: &WorldGenSettings, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Mix coordinates with a unique constant and the seed
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
    pub fn hash_for_weight_maps(&self, settings: &WorldGenSettings,) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Use a different mixing and constants
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        49.hash(&mut hasher); 
        hasher.finish()
    }
    pub fn normalized_hash_value(&self, settings: &WorldGenSettings, seed: u64) -> f32 {
        self.hash_value(settings, seed) as f32 / u64::MAX as f32
    }
    pub const TYPE_DEBUG_NAME: &'static str = "GlobalTilePos";
}

impl std::fmt::Display for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_DEBUG_NAME, self.0.x, self.0.y)
    }
}
impl std::fmt::Debug for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_DEBUG_NAME, self.0.x, self.0.y)
    }
}
impl From<Vec2> for GlobalTilePos {
    fn from(pixelpos: Vec2) -> Self {
        GlobalTilePos(pixelpos.div_euclid(TILE_SIZE_PXS.as_vec2()).as_ivec2())
    }
}
impl std::ops::Add for GlobalTilePos {type Output = Self; fn add(self, other: Self) -> Self {GlobalTilePos(self.0 + other.0)}}
impl std::ops::Sub for GlobalTilePos {type Output = Self; fn sub(self, other: Self) -> Self {GlobalTilePos(self.0 - other.0)}}

#[derive(Debug, Clone, Component, Default)]
#[require(EntityPrefix::new("TileWeightedSampler"), Replicated)]
pub struct TileWeightedSampler {
    #[entities]
    entities: Vec<Entity>,
    weights: Vec<f32>,
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}

impl TileWeightedSampler {
    pub fn new(weights: &[(Entity, f32)]) -> Self {
        let mut cumulative_weights = Vec::with_capacity(weights.len());
        let mut acc = 0.0;
        for &(_, w) in weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Self {
            entities: weights.iter().map(|(e, _)| *e).collect(),
            weights: weights.iter().map(|(_, w)| *w).collect(),
            cumulative_weights,
            total_weight,
        }
    }

    pub fn sample(&self, settings: &WorldGenSettings, pos: GlobalTilePos) -> Option<Entity> {
        if self.entities.is_empty() {
            return None;
        }
        let hash_used_to_sample = pos.hash_for_weight_maps(settings);
        let mut rng_val = (hash_used_to_sample as f64 / u64::MAX as f64) as f32;
        if rng_val >= 1.0 { rng_val = 0.999_999; }
        let target = rng_val * self.total_weight;

        match self.cumulative_weights.binary_search_by(|w| w.partial_cmp(&target).unwrap()) {
            Ok(idx) | Err(idx) => {
                self.entities.get(idx).map(|e| *e)
            }
        }
    }
}

// Manual Serialize/Deserialize to skip cumulative_weights and total_weight

impl Serialize for TileWeightedSampler {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.entities, &self.weights).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TileWeightedSampler {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (entities, weights): (Vec<Entity>, Vec<f32>) = Deserialize::deserialize(deserializer)?;
        // Recompute cumulative_weights and total_weight
        let mut cumulative_weights = Vec::with_capacity(weights.len());
        let mut acc = 0.0;
        for &w in &weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Ok(TileWeightedSampler {
            entities,
            weights,
            cumulative_weights,
            total_weight,
        })
    }
}
