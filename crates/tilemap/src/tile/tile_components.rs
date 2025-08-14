use bevy::math::U8Vec4;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::*, common_states::*};
use game_common::game_common_components::MyZ;

use std::hash::{DefaultHasher, Hash, Hasher};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

use crate::terrain_gen::terrgen_components::OplistSize;
use crate::{chunking_components::ChunkPos, terrain_gen::terrgen_resources::GlobalGenSettings, };
use crate::{tile::{tile_materials::*}, };


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(MyZ, EntityPrefix::new("Tile"), AssetScoped,)]
pub struct Tile;
impl Tile {
    pub const PIXELS: UVec2 = UVec2 { x: 64, y: 64 };
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct TileposHashRand(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
#[require(TileposHashRand)]
pub struct FlipAlongX;

#[derive(Component, Debug,  Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct TileShaderRef(pub Entity);

#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect, )]
#[require(EntityPrefix::new("TileShader"), AssetScoped,)]
pub enum TileShader{
    TexRepeat(MonoRepeatTextureOverlayMat),
    TwoTexRepeat(TwoOverlaysExample),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}



#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct GlobalTilePos(pub IVec2);

impl GlobalTilePos {
    pub fn new(x: i32, y: i32) -> Self {GlobalTilePos(IVec2::new(x, y))}
    
    pub fn get_pos_within_chunk(self, chunk_pos: ChunkPos, oplist_size: OplistSize) -> TilePos {
        let pos_within_chunk = (self.0 - chunk_pos.to_tilepos().0) / oplist_size.inner().as_ivec2();
        TilePos::from(pos_within_chunk.as_uvec2())
    }
    pub fn x(&self) -> i32 { self.0.x } pub fn y(&self) -> i32 { self.0.y }

    pub fn hash_value(&self, settings: &GlobalGenSettings, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Mix coordinates with a unique constant and the seed
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
    pub fn hash_for_weight_maps(&self, settings: &GlobalGenSettings,) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Use a different mixing and constants
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        49.hash(&mut hasher); 
        hasher.finish()
    }
    pub fn normalized_hash_value(&self, settings: &GlobalGenSettings, seed: u64) -> f32 {
        self.hash_value(settings, seed) as f32 / u64::MAX as f32
    }
    pub const TYPE_NAME: &'static str = "G-TilePos";
}

impl std::fmt::Display for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_NAME, self.0.x, self.0.y)
    }
}
impl std::fmt::Debug for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_NAME, self.0.x, self.0.y)
    }
}
impl From<Vec2> for GlobalTilePos {
    fn from(pixelpos: Vec2) -> Self {
        GlobalTilePos(pixelpos.div_euclid(Tile::PIXELS.as_vec2()).as_ivec2())
    }
}
impl From<IVec2> for GlobalTilePos {
    fn from(ivec2: IVec2) -> Self {
        GlobalTilePos(ivec2)
    }
}
impl std::ops::Add for GlobalTilePos {type Output = Self; fn add(self, other: Self) -> Self {GlobalTilePos(self.0 + other.0)}}
impl std::ops::Sub for GlobalTilePos {type Output = Self; fn sub(self, other: Self) -> Self {GlobalTilePos(self.0 - other.0)}}


#[derive(Debug, Clone, Component, Default)]
#[require(EntityPrefix::new("HashPosEntWSampler"), Replicated, SessionScoped,)]
pub struct HashPosEntiWeightedSampler {
    #[entities]entities: Vec<Entity>, weights: Vec<f32>,
    cumulative_weights: Vec<f32>, total_weight: f32,
}

impl HashPosEntiWeightedSampler {
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

    pub fn sample(&self, settings: &GlobalGenSettings, pos: GlobalTilePos) -> Option<Entity> {
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

impl Serialize for HashPosEntiWeightedSampler {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.entities, &self.weights).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for HashPosEntiWeightedSampler {
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
        Ok(HashPosEntiWeightedSampler {
            entities,
            weights,
            cumulative_weights,
            total_weight,
        })
    }
}


#[derive(Debug, Clone, Component, Default)]
#[require(EntityPrefix::new("HashPosWSampler"), Replicated)]
pub struct HashPosWeightedSampler<T: Clone + Serialize> {
    choices_and_weights: Vec<(T, f32)>,
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}

impl<T: Clone + Serialize> HashPosWeightedSampler<T> {
    pub fn new_from_map(weights_map: &HashMap<T, f32>) -> Self {
        let mut choices_and_weights = Vec::with_capacity(weights_map.len());
        for (choice, weight) in weights_map.iter() {
            choices_and_weights.push((choice.clone(), *weight));
        }
        let mut cumulative_weights = Vec::with_capacity(choices_and_weights.len());
        let mut acc = 0.0;
        for &(_, w) in &choices_and_weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Self {
            choices_and_weights,
            cumulative_weights,
            total_weight,
        }
    }

    pub fn sample(&self, settings: &GlobalGenSettings, pos: GlobalTilePos) -> Option<T> {
        if self.choices_and_weights.is_empty() {
            return None;
        }
        let hash_used_to_sample = pos.hash_for_weight_maps(settings);
        let mut rng_val = (hash_used_to_sample as f64 / u64::MAX as f64) as f32;
        if rng_val >= 1.0 { rng_val = 0.999_999; }
        let target = rng_val * self.total_weight;

        match self.cumulative_weights.binary_search_by(|w| w.partial_cmp(&target).unwrap()) {
            Ok(idx) | Err(idx) => {
                self.choices_and_weights.get(idx).map(|(choice, _)| choice.clone())
            }
        }
    }
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> Serialize for HashPosWeightedSampler<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.choices_and_weights).serialize(serializer)
    }
}
impl<'de, T> Deserialize<'de> for HashPosWeightedSampler<T>
where
    T: Clone + Serialize + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let choices_and_weights: Vec<(T, f32)> = Deserialize::deserialize(deserializer)?;
        let mut cumulative_weights = Vec::with_capacity(choices_and_weights.len());
        let mut acc = 0.0;
        for &(_, w) in &choices_and_weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Ok(HashPosWeightedSampler {
            choices_and_weights,
            cumulative_weights,
            total_weight,
        })
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct TileIdsHandles { ids: Vec<HashId>, handles: Vec<Handle<Image>>,}

impl TileIdsHandles {
    pub fn from_paths(asset_server: &AssetServer, img_paths: HashMap<String, String>, ) -> Result<Self, BevyError> {

        if img_paths.is_empty() {
            return Err(BevyError::from("TileImgsMap cannot be created with an empty image paths map"));
        }
        let mut ids = Vec::new();
        let mut handles = Vec::new();
        for (key, path) in img_paths {
            let image_holder = ImageHolder::new(asset_server, &path)?;
            ids.push(HashId::from(key));
            handles.push(image_holder.0);
        }
        Ok(Self { ids, handles, })
    }

    pub fn first_handle(&self) -> Handle<Image> {
        self.handles.first().cloned().unwrap_or_else(|| Handle::default())
    }

    pub fn clone_handles(&self) -> Vec<Handle<Image>> { self.handles.clone() }

    pub fn iter(&self) -> impl Iterator<Item = (HashId, &Handle<Image>)> {
        self.ids.iter().cloned().zip(self.handles.iter())
    }
}
