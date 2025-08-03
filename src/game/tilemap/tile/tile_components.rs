use bevy::math::U8Vec4;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};
use crate::common::common_components::MyZ;
use crate::game::tilemap::terrain_gen::terrgen_resources::WorldGenSettings;
use crate::game::tilemap::{chunking_components::ChunkPos, tile::tile_constants::* };
use bevy_ecs_tilemap::tiles::*;

use bevy_ecs_tilemap::tiles::*;
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(MyZ)]
pub struct Tile;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TileposHashRand(pub f32);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(TileposHashRand)]
pub struct FlipAlongX;


#[derive(Component, Debug, Default, )]
pub struct Tree;

#[derive(Component, Debug,  Deserialize, Serialize, Copy, Clone)]
pub struct ShaderRef(pub Entity);

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, )]//HACER ENTIDAD PROPIA
pub enum TileShader{
    TexRepeat(RepeatingTexture),
    TwoTexRepeat(RepeatingTexture, RepeatingTexture),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, )]
pub struct RepeatingTexture{ img: Handle<Image>, scale: u32, mask_color: U8Vec4, }
impl RepeatingTexture {
    #[allow(dead_code, )]
    pub fn new<S: Into<String>>(asset_server: &AssetServer, path: S, scale: u32, mask_color: U8Vec4) -> Self {
        Self { img: asset_server.load(path.into()), scale, mask_color }
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

    pub fn hash_value(&self, settings: &WorldGenSettings) -> u64 {
        let mut hasher = DefaultHasher::new(); self.0.hash(&mut hasher);
        settings.seed.hash(&mut hasher); hasher.finish()
    }
    pub fn normalized_hash_value(&self, settings: &WorldGenSettings) -> f32 {
        self.hash_value(settings) as f32 / u64::MAX as f32
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

#[derive(Debug, Clone, Component)]
pub struct TileWeightedSampler { alias: Vec<usize>, prob: Vec<f32>, entities: Vec<Entity>, }
impl TileWeightedSampler {
    pub fn new(weights: &[(Entity, f32)]) -> Self {
        let n = weights.len();
        let mut prob = vec![0.0; n];
        let mut alias = vec![0; n];
        let mut entities = Vec::with_capacity(n);
        if n == 0 {
            return Self { alias, prob, entities };
        }
        let mut norm_weights: Vec<f32> = weights.iter().map(|(_, w)| *w).collect();
        let sum: f32 = norm_weights.iter().sum();
        if sum > 0.0 {
            for w in &mut norm_weights {
                *w *= n as f32 / sum;
            }
        }
        let mut small = Vec::new();
        let mut large = Vec::new();
        for (i, &w) in norm_weights.iter().enumerate() {
            entities.push(weights[i].0);
            if w < 1.0 {
                small.push(i);
            } else {
                large.push(i);
            }
        }
        while let (Some(s), Some(l)) = (small.pop(), large.pop()) {
            prob[s] = norm_weights[s];
            alias[s] = l;
            norm_weights[l] = (norm_weights[l] + norm_weights[s]) - 1.0;
            if norm_weights[l] < 1.0 {
                small.push(l);
            } else {
                large.push(l);
            }
        }
        for &i in large.iter().chain(small.iter()) {
            prob[i] = 1.0;
            alias[i] = i;
        }
        Self { alias, prob, entities }
    }

    pub fn sample(&self, settings: &WorldGenSettings, pos: GlobalTilePos) -> Option<Entity> {
        if self.entities.is_empty() {
            return None;
        } 

        let n = self.entities.len();
        let hash = pos.hash_value(settings);

        let idx = (hash % n as u64) as usize;
        // Use upper 32 bits for float
        let float_bits = ((hash >> 32) as u32) | 1; // avoid 0
        let u = (float_bits as f32) / (u32::MAX as f32);

        if u < self.prob[idx] {
            Some(self.entities[idx])
        } else {
            Some(self.entities[self.alias[idx]])
        }
    }
}

